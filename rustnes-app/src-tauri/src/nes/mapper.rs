mod nrom;

use nrom::Nrom;

use enum_dispatch::enum_dispatch;

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

#[enum_dispatch]
pub trait MapperLogic {
    fn irq_active(&self) -> bool;
    fn cpu_step(&mut self);
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
    fn ppu_read(&mut self, addr: u16) -> PpuReadHook;
    fn ppu_write(&mut self, addr: u16, data: u8) -> PpuWriteHook;
}

#[enum_dispatch(MapperLogic)]
#[derive(Debug, Clone)]
pub enum Mapper {
    Nrom,
}

#[derive(Debug, Clone)]
pub struct MapperCtx {
    pub mapper_id: u32,
    pub submapper: u8,
    pub hardwired_nt: Mirroring,
    pub alternative_nt: bool,
    pub prgrom: Vec<u8>,
    pub chrrom: Vec<u8>,
    pub prgram_size: u32,
    pub chrram_size: u32,
}

impl MapperCtx {
    pub fn create(self) -> Mapper {
        match self.mapper_id {
            0 => Mapper::Nrom(Nrom::new(self)),
            _ => todo!(),
        }
    }
}
