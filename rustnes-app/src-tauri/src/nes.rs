pub mod bus;
pub mod cpu;
pub mod emufile;
pub mod mapper;
pub mod parser;
pub mod ppu;

pub use cpu::Cpu;
pub use mapper::{Mapper, MapperFactory};
pub use parser::parse_emufile;
pub use ppu::Ppu;

use bus::{cpubus::WRAM_SIZE, CpuBus};
use bus::{
    ppubus::{PALETTE_SIZE, VRAM_SIZE},
    PpuBus,
};

#[derive(Debug)]
pub struct Nes {
    mapper: Box<dyn Mapper>,
    cpu: Cpu,
    wram: [u8; WRAM_SIZE],
    ppu: Ppu,
    vram: [u8; VRAM_SIZE],
    palette: [u8; PALETTE_SIZE],
}

impl Nes {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        Self {
            mapper,
            cpu: Cpu::new(),
            wram: [0; WRAM_SIZE],
            ppu: Ppu::new(),
            vram: [0; VRAM_SIZE],
            palette: [0; PALETTE_SIZE],
        }
    }

    pub fn reset(&mut self) {
        self.cpu.assert_reset();
    }

    pub fn tick(&mut self) {
        let mut cpubus = CpuBus {
            mapper: &mut self.mapper,
            wram: &mut self.wram,
            ppu: &mut self.ppu,
        };

        self.cpu.tick(&mut cpubus);

        let mut ppubus = PpuBus {
            mapper: &mut self.mapper,
            vram: &mut self.vram,
            palette: &mut self.palette,
        };

        for _ in 0..3 {
            self.ppu.tick(&mut ppubus);
        }
    }
}
