pub mod bus;
pub mod cpu;
pub mod loader;
pub mod mapper;
pub mod ppu;

pub use bus::{CpuBus, NesBus, PpuBus};
pub use cpu::Cpu;
pub use loader::{EnvInfo, NesRom};
pub use mapper::Mapper;
pub use ppu::Ppu;

use bus::{VRAM_SIZE, WRAM_SIZE};

macro_rules! nesbus {
    ($nes:expr) => {
        NesBus {
            nesrom: &mut $nes.nesrom,
            cpu_openbus: &mut $nes.cpu_openbus,
            wram: &mut $nes.wram,
            ppu_openbus: &mut $nes.ppu_openbus,
            ppu: &mut $nes.ppu,
            vram: &mut $nes.vram,
        }
    };
}

#[derive(Debug, Clone)]
pub struct Nes {
    pub nesrom: NesRom,
    pub cpu_openbus: u8,
    pub cpu: Cpu,
    pub wram: [u8; WRAM_SIZE],
    pub ppu_openbus: u8,
    pub ppu: Ppu,
    pub vram: [u8; VRAM_SIZE],
}

impl Nes {
    pub fn new(_env: EnvInfo) -> Self {
        let mut nes = Self {
            nesrom: NesRom::disconnected(),
            cpu_openbus: 0x00,
            cpu: Cpu::new(),
            wram: [0x00; WRAM_SIZE],
            ppu_openbus: 0x00,
            ppu: Ppu::new(),
            vram: [0x00; VRAM_SIZE],
        };

        nes.reset();
        nes
    }

    pub fn load(&mut self, nesrom: NesRom) {
        self.nesrom = nesrom;
    }

    pub fn reset(&mut self) {
        let mut nesbus = nesbus!(self);
        self.cpu.reset(&mut nesbus);
    }

    pub fn step(&mut self) {
        let mut nesbus = nesbus!(self);
        self.cpu.step(&mut nesbus);
    }
}
