use arc_swap::ArcSwap;
use rustnes_core::nes::nescart::NesCart;
use rustnes_core::nes::Nes;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

const EMU_THREADSTACK: usize = 8 * 1024 * 1024;
const EMU_FRAMERATE: f64 = 60.0;

pub enum EmuCommand {
    Reset,
    LoadCart(NesCart),
}

pub struct EmuHandle {
    pub cmd_tx: Sender<EmuCommand>,
}

pub struct FrameBuffer {
    pub width: u16,
    pub height: u16,
    current: ArcSwap<Vec<u8>>,
    generation: AtomicU64,
}

impl FrameBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            current: ArcSwap::from_pointee(vec![0u8; (width as usize) * (height as usize) * 4]),
            generation: AtomicU64::new(0),
        }
    }

    pub fn publish(&self, frame: &[u8]) {
        self.current.store(Arc::new(frame.to_vec()));
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn snapshot(&self) -> Arc<Vec<u8>> {
        self.current.load_full()
    }
}

pub fn spawn_emulator_thread(fb: Arc<FrameBuffer>) -> EmuHandle {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<EmuCommand>();

    let mut nes = Nes::new();
    nes.reset();

    let target = Duration::from_secs_f64(1.0 / EMU_FRAMERATE);
    let mut buffer = vec![0u8; 256 * 240 * 4];

    std::thread::Builder::new()
        .name("emuthread".into())
        .stack_size(EMU_THREADSTACK)
        .spawn(move || loop {
            let t0 = Instant::now();

            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    EmuCommand::Reset => nes.reset(),
                    EmuCommand::LoadCart(cart) => {
                        nes.load_cart(cart);
                        nes.reset();
                    }
                }
            }

            while !nes.is_frame_ready() {
                nes.step();
            }

            nes.output_frame(&mut buffer);
            fb.publish(&buffer);

            let elapsed = t0.elapsed();
            if elapsed < target {
                spin_sleep::sleep(target - elapsed);
            }
        })
        .unwrap();

    EmuHandle { cmd_tx }
}
