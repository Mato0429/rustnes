use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug)]
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

    pub fn as_byte(&self, b_flag: bool) -> u8 {
        let with_r = self.bits() | Self::R;
        if b_flag {
            with_r | Self::B
        } else {
            with_r
        }
    }
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Status::from_bits_truncate(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: Status,
    pub sp: u8,
    pub pc: u16,
}
