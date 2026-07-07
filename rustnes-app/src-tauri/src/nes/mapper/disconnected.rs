use super::{MapperLogic, PpuReadHook, PpuWriteHook};
use crate::nes::bus::BusData;

#[derive(Debug, Clone, Copy)]
pub struct Disconnected;

impl MapperLogic for Disconnected {
    fn irq_active(&self) -> bool {
        false
    }

    fn cpu_step(&mut self) {}

    fn cpu_read(&mut self, _addr: u16) -> BusData {
        BusData::new(0, 0x00)
    }

    fn cpu_write(&mut self, _addr: u16, _data: u8) {}

    fn ppu_read(&mut self, _addr: u16) -> PpuReadHook {
        PpuReadHook::ExternalVram(BusData::new(0, 0x00))
    }

    fn ppu_write(&mut self, _addr: u16, _data: u8) -> PpuWriteHook {
        PpuWriteHook::ExternalVram
    }
}
