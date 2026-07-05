use crate::nes::emufile::RomInfo;
use crate::nes::mapper::{Mapper, Mirroring, PpuReadHook, PpuWriteHook};

#[derive(Debug, Clone)]
pub struct Nrom {
    mirroring: Mirroring,
    prgrom: Vec<u8>,
    chrrom: Vec<u8>,
    prgram: Vec<u8>,
}

impl Mapper for Nrom {
    fn new(info: RomInfo) -> Self
    where
        Self: Sized,
    {
        Self {
            mirroring: info.hardwired_nt,
            prgrom: info.prgrom,
            chrrom: info.chrrom,
            prgram: vec![0u8; info.prgram_size as usize],
        }
    }

    fn irq_active(&self) -> bool {
        false
    }

    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x4020 => panic!("the address({:#06x}) must be handled by cpubus", addr),
            0x4020..0x6000 => 0x00,
            0x6000..0x8000 => self.prgram[(addr as usize) % self.prgram.len()],
            0x8000..=u16::MAX => self.prgrom[(addr as usize) % self.prgrom.len()],
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x4020 => panic!("the address({:#06x}) must be handled by cpubus", addr),
            0x4020..0x6000 => (),
            0x6000..0x8000 => {
                let len = self.prgram.len();
                self.prgram[(addr as usize) % len] = data;
            }
            0x8000..=u16::MAX => {
                let len = self.prgrom.len();
                self.prgrom[(addr as usize) % len] = data;
            }
        }
    }

    fn ppu_read(&mut self, addr: u16) -> PpuReadHook {
        match addr {
            0x0000..0x3000 => PpuReadHook::InternalVram(addr & 0x7FF),
            0x3000..=u16::MAX => panic!("the address({:#06x}) must be handled by ppu", addr),
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) -> PpuWriteHook {
        match addr {
            0x0000..0x3000 => PpuWriteHook::InternalVram(addr & 0x7FF, data),
            0x3000..=u16::MAX => panic!("the address({:#06x}) must be handled by ppu", addr),
        }
    }
}
