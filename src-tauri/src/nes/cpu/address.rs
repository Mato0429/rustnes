pub trait ZeropageAddress {
    fn as_address(&self) -> Address;
}

impl ZeropageAddress for u8 {
    fn as_address(&self) -> Address {
        Address::new(*self as u16)
    }
}

#[derive(Clone, Copy, Debug)]
enum CrossingState {
    CurrentPage,
    NextPage,
    PreviousPage,
}

#[derive(Clone, Copy, Debug)]
pub struct Address {
    inner: u16,
    state: CrossingState,
}

impl Address {
    pub fn new(addr: u16) -> Self {
        Self {
            inner: addr,
            state: CrossingState::CurrentPage,
        }
    }

    pub fn as_u16(&self) -> u16 {
        self.inner
    }

    pub fn as_zeropage(&self) -> Self {
        Self::new(self.inner & 0x00FF)
    }

    pub fn is_crossed(&self) -> bool {
        matches!(
            self.state,
            CrossingState::NextPage | CrossingState::PreviousPage
        )
    }

    pub fn set_lo(&mut self, byte: u8) {
        let [_, hi] = self.inner.to_le_bytes();
        self.inner = u16::from_le_bytes([byte, hi]);
    }

    pub fn set_hi(&mut self, byte: u8) {
        let [lo, _] = self.inner.to_le_bytes();
        self.inner = u16::from_le_bytes([lo, byte]);
    }

    pub fn increment(&mut self) {
        self.inner = self.inner.wrapping_add(1);
    }

    pub fn apply_index(&mut self, idx: u8) {
        let (idxed_lo, is_crossed) = (self.inner as u8).overflowing_add(idx);
        self.set_lo(idxed_lo); // update only low byte.

        self.state = if is_crossed {
            CrossingState::NextPage
        } else {
            CrossingState::CurrentPage
        };
    }

    pub fn apply_index_zeropage(&mut self, idx: u8) {
        self.inner = self.inner.wrapping_add(idx as u16);
        self.set_hi(0x00);
        self.state = CrossingState::CurrentPage;
    }

    pub fn apply_index_signed(&mut self, idx: i8) {
        let (fixed_lo, is_crossed) = (self.inner as u8).overflowing_add_signed(idx);
        self.set_lo(fixed_lo); // update only low byte.

        self.state = if !is_crossed {
            CrossingState::CurrentPage
        } else if idx > 0 {
            CrossingState::NextPage
        } else {
            CrossingState::PreviousPage
        };
    }

    pub fn fix_page(&mut self) {
        let [_, hi] = self.inner.to_le_bytes();

        match self.state {
            CrossingState::CurrentPage => (),
            CrossingState::NextPage => {
                self.set_hi(hi.wrapping_add(1));
            }
            CrossingState::PreviousPage => {
                self.set_hi(hi.wrapping_sub(1));
            }
        }

        self.state = CrossingState::CurrentPage;
    }
}
