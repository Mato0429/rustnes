use arc_swap::ArcSwap;
use rustnes_core::nes::emufile::parse_emufile;
use rustnes_core::nes::nescart::mapper::{Mapper, MapperArgs};
use rustnes_core::nes::nescart::NesCart;
use rustnes_core::nes::Nes;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    current: ArcSwap<Vec<u8>>,
    generation: AtomicU64,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            current: ArcSwap::from_pointee(vec![0u8; (width * height * 3) as usize]),
            generation: AtomicU64::new(0),
        }
    }

    pub fn publish(&self, frame: Vec<u8>) {
        self.current.store(Arc::new(frame));
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn snapshot(&self) -> Arc<Vec<u8>> {
        self.current.load_full()
    }
}

pub fn spawn_emulator_thread(fb: Arc<FrameBuffer>) {
    std::thread::spawn(move || {
        let file = File::open("/workspaces/rustnes/assets/private/smb.nes").unwrap();
        let emufile = parse_emufile(BufReader::new(file)).unwrap();

        let mapper_args = MapperArgs {
            vertical_nt: emufile.vertical_nt,
            alternative_nt: emufile.alternative_nt,
            prgrom_size: emufile.prgrom.len() as u32,
            prgram_size: emufile.prgram_size,
            chrrom_size: emufile.chrrom.len() as u32,
            chrram_size: emufile.chrram_size,
        };

        let cart = NesCart {
            mapper: Mapper::new(emufile.mapper_id, emufile.submapper, mapper_args).unwrap(),
            prgrom: emufile.prgrom,
            prgram: vec![0u8; emufile.prgram_size as usize],
            chrrom: emufile.chrrom,
            chrram: vec![0u8; emufile.chrram_size as usize],
        };

        let mut nes = Nes::new();
        nes.load_cart(cart);
        nes.reset();

        let target = Duration::from_micros(16_667); // 60fps

        loop {
            let t0 = Instant::now();

            for _ in 0..10000 {
                nes.step();
            }

            let rgb: Vec<u8> = nes.display_buffer().into();
            fb.publish(rgb);

            let elapsed = t0.elapsed();
            if elapsed < target {
                std::thread::sleep(target - elapsed);
            }
        }
    });
}
