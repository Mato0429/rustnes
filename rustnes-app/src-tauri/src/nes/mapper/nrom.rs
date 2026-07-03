use crate::nes::mapper::{Mapper, MapperInfo, Mirroring};

#[derive(Debug, Clone, Copy)]
pub struct Nrom {
    mirroring: Mirroring,
}

impl Mapper for Nrom {
    fn new(mapper_info: MapperInfo) -> Self
    where
        Self: Sized,
    {
        Self {
            mirroring: mapper_info.hardwired_nt,
        }
    }

    fn irq_active(&self) -> bool {
        false
    }

    fn read_by_cpu(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn write_by_cpu(&mut self, addr: u16, data: u8) {
        todo!()
    }

    fn read_by_ppu(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn write_by_ppu(&mut self, addr: u16, data: u8) {
        todo!()
    }
}
