use enum_dispatch::enum_dispatch;

mod nrom;

use nrom::Nrom;

#[derive(Debug, Clone)]
pub struct MapperArgs {
    pub hardwired_nt: Mirroring,
    pub alternative_nt: bool,
    pub prgrom_size: u32,
    pub chrrom_size: u32,
    pub prgram_size: u32,
    pub chrram_size: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen0,
    SingleScreen1,
    FourScreen,
}

#[derive(Debug, Clone, Copy)]
pub enum MappedCpuRead {
    MapperChip(u8),
    PrgRom(u32),
    PrgRam(u32),
    Openbus,
}

#[derive(Debug, Clone, Copy)]
pub enum MappedCpuWrite {
    Other, // Handled by the mapper or ignored
    PrgRam(u32, u8),
}

#[derive(Debug, Clone, Copy)]
pub enum MappedPpuRead {
    MapperChip(u8),
    InternalVram(u16),
    ChrRom(u32),
    ChrRam(u32),
    Openbus,
}

#[derive(Debug, Clone, Copy)]
pub enum MappedPpuWrite {
    Other, //  Handled by the mapper or ignored
    InternalVram(u16, u8),
    ChrRam(u32, u8),
}

#[enum_dispatch]
pub trait MapperLogic {
    fn irq_active(&self) -> bool;
    fn cpu_step(&mut self);
    fn map_cpu_read(&mut self, addr: u16) -> MappedCpuRead;
    fn map_cpu_write(&mut self, addr: u16, data: u8) -> MappedCpuWrite;
    fn map_ppu_read(&mut self, addr: u16) -> MappedPpuRead;
    fn map_ppu_write(&mut self, addr: u16, data: u8) -> MappedPpuWrite;
}

#[enum_dispatch(MapperLogic)]
#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    Nrom,
}

impl Mapper {
    pub fn new(mapper_id: u32, submapper: u8, args: MapperArgs) -> Option<Self> {
        let mapper = match (mapper_id, submapper) {
            (0, _) => Mapper::Nrom(Nrom::new(args)),
            _ => return None,
        };

        Some(mapper)
    }
}
