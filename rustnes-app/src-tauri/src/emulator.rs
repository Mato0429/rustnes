use rustnes_core::nes::nescart::NesCart;
use rustnes_core::nes::Nes;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const EMU_THREADSTACK: usize = 8 * 1024 * 1024;
const EMU_FRAMERATE: f64 = 60.0;

pub enum EmuCommand {
    Reset,
    Unload,
    Load(NesCart),
    Hotswap(NesCart),
}

pub struct EmuHandle {
    pub cmd_tx: Sender<EmuCommand>,
    pub fb: Arc<Mutex<Vec<u8>>>,
    pub jp1: Arc<AtomicU8>,
}

pub fn spawn_emulator_thread() -> EmuHandle {
    let target = Duration::from_secs_f64(1.0 / EMU_FRAMERATE);
    let mut nes = Nes::new();
    nes.reset();

    let frame = Arc::new(Mutex::new(vec![0u8; 256 * 240 * 4]));
    let joypad1 = Arc::new(AtomicU8::from(0));
    let fb = Arc::clone(&frame);
    let jp1 = Arc::clone(&joypad1);
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<EmuCommand>();

    std::thread::Builder::new()
        .name("emuthread".into())
        .stack_size(EMU_THREADSTACK)
        .spawn(move || loop {
            let t0 = Instant::now();

            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    EmuCommand::Reset => nes.reset(),
                    EmuCommand::Unload => nes.unload_cart(),
                    EmuCommand::Hotswap(cart) => nes.load_cart(cart),
                    EmuCommand::Load(cart) => {
                        nes.load_cart(cart);
                        nes.reset();
                    }
                }
            }

            nes.update_joypad1(joypad1.load(Ordering::SeqCst));
            nes.step_frame();
            {
                let mut lock = frame.lock().unwrap();
                nes.output_frame(&mut lock);
            }

            let elapsed = t0.elapsed();
            if elapsed < target {
                spin_sleep::sleep(target - elapsed);
            }

            /* if target > elapsed {
                print!("\rtime per frame: {:?}", elapsed);
            } else {
                print!("\r\x1b[31mtime per frame: {:?}\x1b[0m", elapsed);
            }*/
        })
        .unwrap();

    EmuHandle { cmd_tx, fb, jp1 }
}
