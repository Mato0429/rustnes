use std::ops::{Deref, DerefMut};

pub trait U16Ext {
    fn low(&self) -> u8;
    fn high(&self) -> u8;
    fn with_low(&self, byte: u8) -> u16;
    fn with_high(&self, byte: u8) -> u16;
    fn set_low(&mut self, byte: u8);
    fn set_high(&mut self, byte: u8);
}

impl U16Ext for u16 {
    fn low(&self) -> u8 {
        self.to_le_bytes()[0]
    }

    fn high(&self) -> u8 {
        self.to_le_bytes()[1]
    }

    fn with_low(&self, byte: u8) -> u16 {
        u16::from_le_bytes([byte, self.high()])
    }

    fn with_high(&self, byte: u8) -> u16 {
        u16::from_le_bytes([self.low(), byte])
    }

    fn set_low(&mut self, byte: u8) {
        *self = self.with_low(byte);
    }

    fn set_high(&mut self, byte: u8) {
        *self = self.with_high(byte);
    }
}

enum CrossingState {
    CurrentPage,
    NextPage,
    PreviousPage,
}

pub struct CrossingAddr {
    inner: u16,
    state: CrossingState,
}

impl CrossingAddr {
    pub fn is_crossed(&self) -> bool {
        matches!(
            self.state,
            CrossingState::NextPage | CrossingState::PreviousPage
        )
    }

    pub fn apply_index(&mut self, idx: u8) {
        let (idxed_lo, is_crossed) = (self.inner as u8).overflowing_add(idx);
        self.inner.set_low(idxed_lo); // update only low byte.

        self.state = if is_crossed {
            CrossingState::NextPage
        } else {
            CrossingState::CurrentPage
        };
    }

    pub fn apply_index_signed(&mut self, idx: i8) {
        let (fixed_lo, is_crossed) = (self.inner as u8).overflowing_add_signed(idx);
        self.inner.set_low(fixed_lo); // update only low byte.

        self.state = if !is_crossed {
            CrossingState::CurrentPage
        } else if idx > 0 {
            CrossingState::NextPage
        } else {
            CrossingState::PreviousPage
        };
    }

    pub fn fix_page(&mut self) {
        match self.state {
            CrossingState::CurrentPage => (),
            CrossingState::NextPage => {
                self.inner.set_high(self.inner.high().wrapping_add(1));
            }
            CrossingState::PreviousPage => {
                self.inner.set_high(self.inner.high().wrapping_sub(1));
            }
        }

        self.state = CrossingState::CurrentPage;
    }
}

impl Deref for CrossingAddr {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CrossingAddr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
