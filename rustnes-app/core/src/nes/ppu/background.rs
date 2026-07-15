use super::*;

impl Ppu {
    pub(super) fn advance_background_pipeline(&mut self, bus: &mut impl Bus) {
        if let (0..=239 | 261, 1..=256 | 321..=336) = (self.scanline, self.cycle) {
            self.advance_tile_fetch(bus)
        }

        if let (0..=239 | 261, 256) = (self.scanline, self.cycle) {
            self.increment_fine_y()
        }

        // copy horizontal(coarseX, nametable lo)
        if let (0..=239 | 261, 257) = (self.scanline, self.cycle) {
            self.scrl.v = (self.scrl.v & 0x7BE0) | (self.scrl.t & 0x041F);
        }

        // copy vertical(fineY, coarseY, nametable hi)
        if let (261, 280..=304) = (self.scanline, self.cycle) {
            self.scrl.v = (self.scrl.v & 0x041F) | (self.scrl.t & 0x7BE0);
        }
    }

    fn advance_tile_fetch(&mut self, bus: &mut impl Bus) {
        match ((self.cycle - 1) % 8) + 1 {
            // nametable fetch
            1 => self.latch_addr(self.nt_addr()),
            2 => self.bg_line.tile_idx = self.fetch(bus),

            // attribute fetch
            3 => self.latch_addr(self.at_addr()),
            4 => {
                let at_byte = self.fetch(bus);
                let [lo, hi] = self.extract_at_from_byte(at_byte);
                self.bg_line.at_lo = lo;
                self.bg_line.at_hi = hi;
            }

            // pattern lo fetch
            5 => self.latch_addr(self.pt_addr()),
            6 => self.bg_line.pt_lo = self.fetch(bus),

            // pattern hi fetch
            7 => self.latch_addr(self.pt_addr().wrapping_add(8)),
            8 => {
                self.bg_line.pt_hi = self.fetch(bus);
                self.bg_liner.load_tileline(self.bg_line);
                self.increment_coarse_x();
            }
            _ => unreachable!(),
        }
    }

    fn nt_addr(&self) -> u16 {
        0x2000 | (self.scrl.v & 0x0FFF)
    }

    fn at_addr(&self) -> u16 {
        0x23C0 | (self.scrl.v & 0x0C00) | ((self.scrl.v >> 4) & 0x38) | ((self.scrl.v >> 2) & 0x07)
    }

    fn pt_addr(&self) -> u16 {
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
