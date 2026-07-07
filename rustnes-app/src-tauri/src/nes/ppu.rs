mod palette;

use crate::nes::{bus::BusData, PpuBus};
pub use palette::Palette;

#[derive(Debug, Clone, Copy)]
pub struct Ppu {}

impl Ppu {
    pub fn new() -> Self {
        todo!()
    }

    pub fn read_register(&mut self, addr: u8) -> BusData {
        todo!()
    }

    pub fn write_register(&mut self, addr: u8, data: u8) {}

    pub fn cpu_step(&mut self, bus: &mut PpuBus) {}

    pub fn tick(&mut self, bus: &mut PpuBus) {}
}
