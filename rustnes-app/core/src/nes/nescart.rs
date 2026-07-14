pub mod mapper;
pub mod mirroring;

use mapper::{MappedCpuRead, MappedCpuWrite, MappedPpuRead, MappedPpuWrite, Mapper, MapperLogic};

#[derive(Debug, Clone, Copy)]
pub enum PpuRead {
    Internal(u16),
    External(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum PpuWrite {
    Internal(u16, u8),
    External,
}

#[derive(Debug, Clone)]
pub struct NesCart {
    pub mapper: Mapper,
    pub prgrom: Vec<u8>,
    pub prgram: Vec<u8>,
    pub chrrom: Vec<u8>,
    pub chrram: Vec<u8>,
}

impl NesCart {
    pub fn empty() -> Self {
        Self {
            mapper: Mapper::empty(),
            prgrom: Vec::new(),
            prgram: Vec::new(),
            chrrom: Vec::new(),
            chrram: Vec::new(),
        }
    }

    pub fn irq_active(&self) -> bool {
        self.mapper.irq_active()
    }

    pub fn cpu_step(&mut self) {
        self.mapper.cpu_step();
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match self.mapper.map_cpu_read(addr) {
            MappedCpuRead::MapperChip(data) => data,
            MappedCpuRead::PrgRom(addr) => self.prgrom[addr as usize],
            MappedCpuRead::PrgRam(addr) => self.prgram[addr as usize],
            MappedCpuRead::Openbus => 0x00, // TODO: openbus
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match self.mapper.map_cpu_write(addr, data) {
            MappedCpuWrite::Other => (),
            MappedCpuWrite::PrgRam(addr, data) => self.prgram[addr as usize] = data,
        }
    }

    pub fn ppu_read(&mut self, addr: u16) -> PpuRead {
        match self.mapper.map_ppu_read(addr) {
            MappedPpuRead::MapperChip(data) => PpuRead::External(data),
            MappedPpuRead::InternalVram(addr) => PpuRead::Internal(addr),
            MappedPpuRead::ChrRom(addr) => PpuRead::External(self.chrrom[addr as usize]),
            MappedPpuRead::ChrRam(addr) => PpuRead::External(self.chrram[addr as usize]),
            MappedPpuRead::Openbus => PpuRead::External(0x00), // TODO: openbus
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> PpuWrite {
        match self.mapper.map_ppu_write(addr, data) {
            MappedPpuWrite::Other => PpuWrite::External,
            MappedPpuWrite::InternalVram(addr, data) => PpuWrite::Internal(addr, data),
            MappedPpuWrite::ChrRam(addr, data) => {
                self.chrram[addr as usize] = data;
                PpuWrite::External
            }
        }
    }
}
