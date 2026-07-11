pub mod mapper;

use crate::nes::{emufile::NesRomInfo, nesrom::mapper::MapperArgs};
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
pub struct NesRom {
    mapper: Mapper,
    prgrom: Vec<u8>,
    prgram: Vec<u8>,
    chrrom: Vec<u8>,
    chrram: Vec<u8>,
}

impl NesRom {
    pub fn new(info: NesRomInfo) -> Self {
        let mapper_args = MapperArgs {
            hardwired_nt: info.hardwired_nt,
            alternative_nt: info.alternative_nt,
            prgrom_size: info.prgrom.len() as u32,
            prgram_size: info.prgram_size,
            chrrom_size: info.chrrom.len() as u32,
            chrram_size: info.chrram_size,
        };

        let mapper = Mapper::new(info.mapper_id, info.submapper, mapper_args).unwrap();

        Self {
            mapper,
            prgrom: info.prgrom,
            prgram: vec![0u8; info.prgram_size as usize],
            chrrom: info.chrrom,
            chrram: vec![0u8; info.chrram_size as usize],
        }
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
