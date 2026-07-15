#[derive(Debug, Clone, Copy, Default)]
pub struct PixLine {
    pub tile_idx: u8,
    pub at_lo: bool,
    pub at_hi: bool,
    pub pt_lo: u8,
    pub pt_hi: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BgPixLiner {
    at_lo: u16,
    at_hi: u16,
    pt_lo: u16,
    pt_hi: u16,
}

impl BgPixLiner {
    pub fn load_tileline(&mut self, latch: PixLine) {
        self.at_lo = (self.at_lo & 0xFF00) | if latch.at_lo { 0xFF } else { 0x00 };
        self.at_hi = (self.at_hi & 0xFF00) | if latch.at_hi { 0xFF } else { 0x00 };
        self.pt_lo = (self.pt_lo & 0xFF00) | latch.pt_lo as u16;
        self.pt_hi = (self.pt_hi & 0xFF00) | latch.pt_hi as u16;
    }

    pub fn shift(&mut self) {
        self.at_lo <<= 1;
        self.at_hi <<= 1;
        self.pt_lo <<= 1;
        self.pt_hi <<= 1;
    }

    pub fn pixel_index(&self, fine_x: u8) -> u8 {
        let shift = 15 - fine_x;
        let p0 = ((self.pt_lo >> shift) & 0x01) as u8;
        let p1 = ((self.pt_hi >> shift) & 0x01) as u8;
        let a0 = ((self.at_lo >> shift) & 0x01) as u8;
        let a1 = ((self.at_hi >> shift) & 0x01) as u8;
        (a1 << 3) | (a0 << 2) | (p1 << 1) | p0
    }
}
