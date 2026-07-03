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
pub struct MapperInfo {
    pub mapper_id: u32,
    pub submapper: u8,
    pub hardwired_nt: Mirroring,
    pub alternative_nt: bool,
    pub has_battery: bool,
    pub has_trainer: bool,
    pub prgrom_size: u32,
    pub chrrom_size: u32,
    pub prgram_size: u32,
    pub chrram_size: u32,
    pub nvprgram_size: u32,
    pub nvchrram_size: u32,
}

pub trait Mapper: std::fmt::Debug {
    fn new(mapper_info: MapperInfo) -> Self
    where
        Self: Sized;
    fn irq_active(&self) -> bool;
    fn read_by_cpu(&mut self, addr: u16) -> u8;
    fn write_by_cpu(&mut self, addr: u16, data: u8);
    fn read_by_ppu(&mut self, addr: u16) -> u8;
    fn write_by_ppu(&mut self, addr: u16, data: u8);
}

pub struct MapperFactory;

impl MapperFactory {
    pub fn create(mapper_info: MapperInfo) -> Option<Box<dyn Mapper>> {
        let mapper = match mapper_info.mapper_id {
            0 => Box::new(Nrom::new(mapper_info)),
            _ => return None,
        };

        Some(mapper)
    }
}
