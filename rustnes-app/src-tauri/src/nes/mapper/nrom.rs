use super::{MapperLogic, PpuReadHook, PpuWriteHook};
use crate::nes::mapper::MapperCtx;

#[derive(Debug, Clone)]
pub struct Nrom {
    prgram: Vec<u8>,
    prgrom: Vec<u8>,
    chrrom: Vec<u8>,
}

impl Nrom {
    pub fn new(ctx: MapperCtx) -> Self {
        Self {
            prgram: vec![0u8; ctx.prgram_size as usize],
            prgrom: ctx.prgrom,
            chrrom: ctx.chrrom,
        }
    }
}

impl MapperLogic for Nrom {
    fn irq_active(&self) -> bool {
        false
    }

    fn cpu_step(&mut self) {}

    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x4020 => unreachable!(),
            0x4020..0x6000 => 0x00,
            0x6000..0x8000 => self.prgram[(addr - 0x6000) as usize],
            0x8000..0xC000 => self.prgrom[(addr - 0x8000) as usize],
            0xC000..=u16::MAX => self.prgrom[(addr - 0xC000) as usize],
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x4020 => unreachable!(),
            0x4020..0x6000 => (),
            0x6000..0x8000 => self.prgram[(addr - 0x6000) as usize] = data,
            0x8000..=u16::MAX => (),
        }
    }

    fn ppu_read(&mut self, _addr: u16) -> PpuReadHook {
        PpuReadHook::ExternalVram(0x00)
    }

    fn ppu_write(&mut self, _addr: u16, _data: u8) -> PpuWriteHook {
        PpuWriteHook::ExternalVram
    }
}
