mod cpubus;
mod ppubus;

use crate::nes::{mapper::MapperLogic, NesRom, Ppu};
pub use cpubus::{CpuBus, WRAM_SIZE};
pub use ppubus::{PpuBus, VRAM_SIZE};

#[derive(Debug, Clone, Copy)]
pub struct BusData {
    data: u8,
    stable_mask: u8,
}

impl BusData {
    pub fn new(data: u8, stable_mask: u8) -> Self {
        Self { data, stable_mask }
    }

    pub fn composite(self, openbus: u8) -> u8 {
        (self.data & self.stable_mask) | (openbus & !self.stable_mask)
    }
}

#[derive(Debug)]
pub struct NesBus<'a> {
    pub nesrom: &'a mut NesRom,
    pub cpu_openbus: &'a mut u8,
    pub wram: &'a mut [u8; WRAM_SIZE],
    pub ppu_openbus: &'a mut u8,
    pub ppu: &'a mut Ppu,
    pub vram: &'a mut [u8; VRAM_SIZE],
}

impl<'a> NesBus<'a> {
    pub fn sync_with_cpu(&mut self) {
        let mut ppubus = PpuBus {
            openbus: &mut *self.ppu_openbus,
            vram: &mut *self.vram,
        };

        self.ppu.cpu_step(&mut ppubus);
        self.nesrom.cpu_step();
    }
}
