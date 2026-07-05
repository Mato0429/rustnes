use crate::nes::emufile::RomInfo;

mod nrom;

pub use nrom::Nrom;

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen0,
    SingleScreen1,
    FourScreen,
}

#[derive(Debug, Clone, Copy)]
pub enum PpuReadHook {
    InternalVram(u16),
    ExternalVram(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum PpuWriteHook {
    InternalVram(u16, u8),
    ExternalVram,
}

pub trait Mapper: std::fmt::Debug {
    fn new(rom_info: RomInfo) -> Self
    where
        Self: Sized;
    fn irq_active(&self) -> bool;
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
    fn ppu_read(&mut self, addr: u16) -> PpuReadHook;
    fn ppu_write(&mut self, addr: u16, data: u8) -> PpuWriteHook;
}

pub struct MapperFactory;

impl MapperFactory {
    pub fn create(info: RomInfo) -> Option<Box<dyn Mapper>> {
        let mapper = match info.mapper_id {
            0 => Box::new(Nrom::new(info)),
            _ => return None,
        };

        Some(mapper)
    }
}
