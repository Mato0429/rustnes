use crate::nes::{Mapper, Ppu};

pub const WRAM_SIZE: usize = 0x800;

pub struct CpuBus<'a> {
    pub wram: &'a mut [u8; WRAM_SIZE],
    pub ppu: &'a mut Ppu,
    pub mapper: &'a mut Box<dyn Mapper>,
}

impl<'a> CpuBus<'a> {
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000 => self.wram[(addr as usize) & 0x7FF],
            0x2000..0x4000 => self.ppu.cpu_read(addr & 0x7),
            0x4000..0x4020 => 0x00, // TODO: APU, DMA, I/O registers
            0x4020..=u16::MAX => self.mapper.cpu_read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x2000 => self.wram[(addr as usize) & 0x7FF] = data,
            0x2000..0x4000 => self.ppu.cpu_write(addr & 0x7, data),
            0x4000..0x4020 => (), // TODO: APU, DMA, I/O registers
            0x4020..=u16::MAX => self.mapper.cpu_write(addr, data),
        }
    }
}
