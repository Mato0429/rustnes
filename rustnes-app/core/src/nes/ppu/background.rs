use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BgPixLine {
    pub tile_idx: u8,
    pub at_lo: bool,
    pub at_hi: bool,
    pub pt_lo: u8,
    pub pt_hi: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BgPixLiner {
    at_lo: u16,
    at_hi: u16,
    pt_lo: u16,
    pt_hi: u16,
}

impl BgPixLiner {
    pub fn load_tileline(&mut self, latch: BgPixLine) {
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

impl Ppu {
    pub(super) fn advance_background_pipeline(&mut self, bus: &mut impl Bus) {
        if let (0..=239 | 261, 1..=256 | 321..=336) = (self.scanline, self.cycle) {
            self.advance_bg_line_fetch(bus)
        }

        if let (0..=239 | 261, 256) = (self.scanline, self.cycle) {
            self.increment_fine_y()
        }

        // copy horizontal(coarseX, nametable lo)
        if let (0..=239 | 261, 257) = (self.scanline, self.cycle) {
            self.scrl.v = (self.scrl.v & 0x7BE0) | (self.scrl.t & 0x041F);
        }

        if let (0..=239 | 261, 257..=320) = (self.scanline, self.cycle) {
            self.advance_bg_dummy_fetch(bus);
        }

        // copy vertical(fineY, coarseY, nametable hi)
        if let (261, 280..=304) = (self.scanline, self.cycle) {
            self.scrl.v = (self.scrl.v & 0x041F) | (self.scrl.t & 0x7BE0);
        }
    }

    fn advance_bg_line_fetch(&mut self, bus: &mut impl Bus) {
        self.bg_liner.shift();

        match ((self.cycle - 1) % 8) + 1 {
            // nametable fetch
            1 => self.latch_addr(self.bg_nt_addr()),
            2 => self.bg_line.tile_idx = self.fetch(bus),

            // attribute fetch
            3 => self.latch_addr(self.bg_at_addr()),
            4 => {
                let at_byte = self.fetch(bus);
                let [lo, hi] = self.extract_at_from_byte(at_byte);
                self.bg_line.at_lo = lo;
                self.bg_line.at_hi = hi;
            }

            // pattern lo fetch
            5 => self.latch_addr(self.bg_pt_addr()),
            6 => self.bg_line.pt_lo = self.fetch(bus),

            // pattern hi fetch
            7 => self.latch_addr(self.bg_pt_addr().wrapping_add(8)),
            8 => {
                self.bg_line.pt_hi = self.fetch(bus);
                self.bg_liner.load_tileline(self.bg_line);
                self.increment_coarse_x();
            }
            _ => unreachable!(),
        }
    }

    fn advance_bg_dummy_fetch(&mut self, bus: &mut impl Bus) {
        self.bg_liner.shift();

        match ((self.cycle - 1) % 8) + 1 {
            // unused nametable fetch
            1 => self.latch_addr(self.bg_nt_addr()),
            2 => self.bg_line.tile_idx = self.fetch(bus),

            // ignored nametable fetch
            3 => self.latch_addr(self.bg_nt_addr()),
            4 => {
                let _ = self.fetch(bus);
            }

            // do nothing while sprite fetch
            5..=8 => (),

            _ => unreachable!(),
        }
    }

    fn bg_nt_addr(&self) -> u16 {
        0x2000 | (self.scrl.v & 0x0FFF)
    }

    fn bg_at_addr(&self) -> u16 {
        0x23C0 | (self.scrl.v & 0x0C00) | ((self.scrl.v >> 4) & 0x38) | ((self.scrl.v >> 2) & 0x07)
    }

    fn bg_pt_addr(&self) -> u16 {
        let table_flag = self.ctrl.contains(PpuCtrl::BgPtTableSelect);
        let table = if table_flag { 0x1000 } else { 0x0000 };
        let fine_y = (self.scrl.v & 0x7000) >> 12;
        table | ((self.bg_line.tile_idx as u16) << 4) | fine_y
    }

    fn extract_at_from_byte(&self, byte: u8) -> [bool; 2] {
        let coarse_y = (self.scrl.v & 0x03E0) >> 5;
        let coarse_x = self.scrl.v & 0x001F;
        let shifted = byte >> (((coarse_y & 0x2) << 1) | coarse_x & 0x2);
        let lo = shifted & 0x01 != 0;
        let hi = shifted & 0x02 != 0;
        [lo, hi]
    }

    fn increment_coarse_x(&mut self) {
        let coarse_x = self.scrl.v & 0x001F;
        if coarse_x == 31 {
            self.scrl.v &= 0x7FE0; // set coarseX to 0
            self.scrl.v ^= 0x0400; // inverse nametable lo
        } else {
            self.scrl.v = (self.scrl.v & 0x7FE0) | coarse_x.wrapping_add(1)
        }
    }

    fn increment_fine_y(&mut self) {
        let fine_y = (self.scrl.v & 0x7000) >> 12;
        if fine_y == 7 {
            self.scrl.v &= 0x0FFF; // set fineY to 0
            self.increment_coarse_y();
        } else {
            self.scrl.v = (self.scrl.v & 0x0FFF) | (fine_y.wrapping_add(1) << 12)
        }
    }

    fn increment_coarse_y(&mut self) {
        let coarse_y = (self.scrl.v & 0x03E0) >> 5;
        if coarse_y == 29 {
            self.scrl.v &= 0x7C1F; // set coarseY to 0
            self.scrl.v ^= 0x0800; // inverse nametable hi
        } else if coarse_y == 31 {
            self.scrl.v &= 0x7C1F; // set coarseY to 0
        } else {
            self.scrl.v = (self.scrl.v & 0x7C1F) | (coarse_y.wrapping_add(1) << 5)
        }
    }
}
