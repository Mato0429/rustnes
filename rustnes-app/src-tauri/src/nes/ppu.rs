use crate::nes::bus::PpuBus;

pub const VRAM_SIZE: usize = 0x800;

#[derive(Debug, Clone, Copy)]
pub struct Ppu {}

impl Ppu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, bus: &mut PpuBus) {}

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        todo!()
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {}
}
