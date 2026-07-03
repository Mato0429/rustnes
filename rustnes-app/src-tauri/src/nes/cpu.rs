use crate::nes::Ppu;
use bitflags::bitflags;

const RESET_VECTOR: u16 = 0xFFFC;
const NMI_VECTOR: u16 = 0xFFFA;
const IRQ_VECTOR: u16 = 0xFFFE;

pub const WRAM_SIZE: usize = 0x800;

pub struct CpuBus<'a> {
    pub openbus: &'a mut u8,
    pub wram: &'a mut [u8; WRAM_SIZE],
    pub ppu: &'a mut Ppu,
}

impl<'a> CpuBus<'a> {
    fn read(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, data: u8) {}
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Status: u8 {
        const C = 0b0000_0001;
        const Z = 0b0000_0010;
        const I = 0b0000_0100;
        const D = 0b0000_1000;
        const V = 0b0100_0000;
        const N = 0b1000_0000;
    }
}

impl Status {
    const B: u8 = 0b0001_0000;
    const R: u8 = 0b0010_0000;

    pub const fn new(v: u8) -> Self {
        Self::from_bits_truncate(v)
    }

    pub fn as_byte(&self, b_flag: bool) -> u8 {
        let with_r = self.bits() | Self::R;
        if b_flag {
            with_r | Self::B
        } else {
            with_r
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Reg {
    a: u8,
    x: u8,
    y: u8,
    p: Status,
    sp: u8,
    pc: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Cpu {
    reg: Reg,
}

impl Cpu {
    pub const fn new() -> Self {
        Self {
            reg: Reg {
                a: 0,
                x: 0,
                y: 0,
                p: Status::new(0x34),
                sp: 0x00,
                pc: 0x8000,
            },
        }
    }

    pub fn reset(&mut self) {}

    pub fn tick(&mut self, bus: &mut CpuBus) {}
}
