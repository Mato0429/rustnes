const ZERO_PAGE: u8 = 0x00;
const STACK_PAGE: u8 = 0x01;

use modular_bitfield::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Word {
    pub lo: u8,
    pub hi: u8,
}

impl Word {
    pub fn from_le_bytes(bytes: [u8; 2]) -> Self {
        Self::from(u16::from_le_bytes(bytes))
    }

    pub fn from_zeropage(lo: u8) -> Self {
        Self::from_le_bytes([lo, ZERO_PAGE])
    }
}

impl From<Word> for u16 {
    fn from(value: Word) -> Self {
        Self::from_le_bytes([value.lo, value.hi])
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        let [lo, hi] = value.to_le_bytes();
        Self { lo, hi }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackPtr {
    pub inner: u8,
}

impl From<StackPtr> for u8 {
    fn from(value: StackPtr) -> Self {
        value.inner
    }
}

impl From<u8> for StackPtr {
    fn from(value: u8) -> Self {
        Self { inner: value }
    }
}

impl From<StackPtr> for u16 {
    fn from(value: StackPtr) -> Self {
        Self::from_le_bytes([value.into(), STACK_PAGE])
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct Status {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    #[skip] // Break (bit4) depends
    __: B2, // Reserved(bit5) is always 1
    pub overflow: bool,
    pub negative: bool,
}

impl Status {
    const BREAK_MASK: u8 = 0b0001_0000;
    const RESERVED_MASK: u8 = 0b0010_0000;

    pub fn as_byte(&self, b_flag: bool) -> u8 {
        let with_r = self.into_bytes()[0] | Self::RESERVED_MASK;
        if b_flag {
            with_r | Self::BREAK_MASK
        } else {
            with_r
        }
    }
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Status::from_bytes([value])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: Status,
    pub sp: StackPtr,
    pub pc: Word,
}
