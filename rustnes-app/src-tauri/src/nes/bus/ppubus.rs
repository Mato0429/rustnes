use crate::nes::{
    mapper::{PpuReadHook, PpuWriteHook},
    Mapper,
};

pub const VRAM_SIZE: usize = 0x800;
pub const PALETTE_SIZE: usize = 0x20;

pub struct PpuBus<'a> {
    pub vram: &'a mut [u8; VRAM_SIZE],
    pub mapper: &'a mut Box<dyn Mapper>,
    pub palette: &'a mut [u8; PALETTE_SIZE],
}

impl<'a> PpuBus<'a> {
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x3F00 => {
                let hook = self.mapper.ppu_read(addr & 0xFFF);
                match hook {
                    PpuReadHook::InternalVram(addr) => self.vram[(addr as usize) & 0x7FF],
                    PpuReadHook::ExternalVram(data) => data,
                }
            }
            0x3F00..0x3FFF => self.palette[(addr as usize) & 0x1F],
            0x3FFF..=u16::MAX => panic!(""),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x3F00 => {
                let hook = self.mapper.ppu_write(addr & 0xFFF, data);
                match hook {
                    PpuWriteHook::InternalVram(vram_addr, data) => {
                        self.vram[(vram_addr as usize) & 0x7FF] = data;
                    }
                    PpuWriteHook::ExternalVram => (),
                }
            }
            0x3F00..0x3FFF => self.palette[(addr as usize) & 0x1F] = data,
            0x3FFF..=u16::MAX => panic!(""),
        }
    }
}
