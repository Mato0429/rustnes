/// Holds the fetched data for a single line of a tile (8 pixels)
/// before it is loaded into shift registers.
#[derive(Default, Debug, Clone, Copy)]
pub struct BgLatches {
    pub tile_idx: u8,
    pub at: u8,
    pub pt_low: u8,
    pub pt_high: u8,
}

impl BgLatches {
    /// Creates a new `BgLatches` in its power-on state.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Manages 16-bit shift registers to provide a continuous stream
/// of pixel data to support horizontal scroll.
#[derive(Default, Debug, Clone, Copy)]
pub struct BgShiftRegisters {
    pt_low: u16,
    pt_high: u16,
    at_low: u16,
    at_high: u16,
}

impl BgShiftRegisters {
    /// Creates a new `BgShiftRegsters`
    pub fn new() -> Self {
        Self::default()
    }

    /// Shifts the shift registers.
    pub fn shift(&mut self) {
        self.pt_low <<= 1;
        self.pt_high <<= 1;
        self.at_low <<= 1;
        self.at_high <<= 1;
    }

    /// Loads the next tile data to be rendered from temporary latches.
    ///
    /// Only the lower 2 bits of the latch's attributes are used; the rest are ignored.
    pub fn load(&mut self, latches: &BgLatches) {
        self.pt_low = (self.pt_low & 0xFF00) | latches.pt_low as u16;
        self.pt_high = (self.pt_high & 0xFF00) | latches.pt_high as u16;
        self.at_low = (self.at_low & 0xFF00) | if latches.at & 0x1 != 0 { 0xFF } else { 0x00 };
        self.at_high = (self.at_high & 0xFF00) | if latches.at & 0x2 != 0 { 0xFF } else { 0x00 };
    }

    /// Returns the internal address of palette RAM as a result of pixel data composition.
    pub fn palette_address(&self, fine_x: u8) -> u16 {
        let shift = 15 - fine_x;
        let p0 = (self.pt_low >> shift) & 0x01;
        let p1 = (self.pt_high >> shift) & 0x01;
        let a0 = (self.at_low >> shift) & 0x01;
        let a1 = (self.at_high >> shift) & 0x01;
        (a1 << 3) | (a0 << 2) | (p1 << 1) | p0
    }
}
