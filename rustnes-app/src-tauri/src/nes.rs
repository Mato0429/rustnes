mod cpu;
mod mapper;
mod nesrom;
mod ppu;

pub use cpu::{Cpu, CpuBus, WRAM_SIZE};
pub use mapper::{Mapper, MapperFactory};
pub use nesrom::NesRom;
pub use ppu::{Ppu, PpuBus, VRAM_SIZE};

// TODO: Support console details
#[derive(Debug, Clone, Copy)]
pub enum ConsoleType {
    NesOrFamicom,
    VsSystem { ppu: u8, hw: u8 },
    PlayChoise10,
    Extended { console: u8 },
}

#[derive(Debug, Clone, Copy)]
pub enum NesTiming {
    NtscNes,
    PalNes,
    MultiRegion,
    Dendy,
}

#[derive(Debug, Clone, Copy)]
pub struct EnvInfo {
    pub console_type: ConsoleType,
    pub cpu_ppu_timing: NesTiming,
    pub other_roms: u8,
    pub expansion_device: u8, // TODO: Support expansion devices
}

#[derive(Debug)]
pub struct Nes {
    cart: NesRom,
    cpu: Cpu,
    cpu_openbus: u8,
    wram: [u8; WRAM_SIZE],

    ppu: Ppu,
    ppu_openbus: u8,
    vram: [u8; VRAM_SIZE],
}

impl Nes {
    pub fn new(cart: NesRom) -> Self {
        Self {
            cart,

            cpu: Cpu::new(),
            cpu_openbus: 0x00,
            wram: [0; WRAM_SIZE],

            ppu: Ppu::new(),
            ppu_openbus: 0x00,
            vram: [0; VRAM_SIZE],
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.reset();
    }

    pub fn tick(&mut self) {
        let mut cpubus = CpuBus {
            openbus: &mut self.cpu_openbus,
            wram: &mut self.wram,
            ppu: &mut self.ppu,
        };

        self.cpu.tick(&mut cpubus);
    }
}
