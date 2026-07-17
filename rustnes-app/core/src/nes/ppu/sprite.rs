use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SprPixLine {
    pub x: u8,
    pub attr: u8,
    pub pt_lo: u8,
    pub pt_hi: u8,
}

impl Ppu {
    pub fn advance_sprite_pipeline(&mut self, bus: &mut impl Bus) {
        if let (0..=239, 256) = (self.scanline, self.cycle) {
            self.evaluate_sprite();
        }

        if let (0..=239, 257..=320) = (self.scanline, self.cycle) {
            self.advance_spr_line_fetch(bus);
        }
    }

    fn evaluate_sprite(&mut self) {
        self.secondary_oam.fill(0xFF);

        let mut sec_spr = 0;
        for pri_spr in 0..64 {
            if 8 <= sec_spr {
                break;
            }

            let sprite_y = self.primary_oam[pri_spr * 4];
            if self.is_in_scanline(sprite_y) {
                let sprite = &self.primary_oam[(pri_spr * 4)..(pri_spr * 4 + 4)];
                self.secondary_oam[(sec_spr * 4)..(sec_spr * 4 + 4)].copy_from_slice(sprite);
                sec_spr += 1;
            }
        }
    }

    fn advance_spr_line_fetch(&mut self, bus: &mut impl Bus) {
        let spr_id = (self.cycle - 257) / 8;

        if 239 <= self.secondary_oam[spr_id * 4] {
            self.spr_lines[spr_id].pt_lo = 0x00;
            self.spr_lines[spr_id].pt_hi = 0x00;
            return;
        }

        match ((self.cycle - 1) % 8) + 1 {
            // copy sprite X
            1 => (),
            2 => self.spr_lines[spr_id].x = self.secondary_oam[spr_id * 4 + 3],

            // copy sprite attribute
            3 => (),
            4 => self.spr_lines[spr_id].attr = self.secondary_oam[spr_id * 4 + 2],

            // pattern lo fetch
            5 => self.latch_addr(self.spr_pt_addr(spr_id)),
            6 => self.spr_lines[spr_id].pt_lo = self.fetch(bus),

            // pattern hi fetch
            7 => self.latch_addr(self.spr_pt_addr(spr_id).wrapping_add(8)),
            8 => {
                self.spr_lines[spr_id].pt_hi = self.fetch(bus);
                let flipflag = self.spr_lines[spr_id].attr & 0x40 != 0;
                if flipflag {
                    self.spr_lines[spr_id].pt_hi = self.spr_lines[spr_id].pt_hi.reverse_bits();
                    self.spr_lines[spr_id].pt_lo = self.spr_lines[spr_id].pt_lo.reverse_bits();
                }
            }
            _ => unreachable!(),
        }
    }

    fn spr_pt_addr(&self, spr_id: usize) -> u16 {
        let sprite_y = self.secondary_oam[spr_id * 4];
        let tile_idx = self.secondary_oam[spr_id * 4 + 1];
        let spr_attr = self.secondary_oam[spr_id * 4 + 2];
        let dy = self.scanline as u8 - sprite_y;

        if self.is_8x8sprite() {
            let dy = if spr_attr & 0x80 != 0 { 7 - dy } else { dy };
            let table_flag = self.ctrl.contains(PpuCtrl::SprPtTableSelect);
            let table = if table_flag { 0x1000 } else { 0x0000 };
            table | ((tile_idx as u16) << 4) | dy as u16
        } else {
            let dy = if spr_attr & 0x80 != 0 { 15 - dy } else { dy };
            let table = if tile_idx & 0x01 != 0 { 0x1000 } else { 0x0000 };
            let tile_idx = if 7 < dy {
                tile_idx | 0x01
            } else {
                tile_idx & 0xFE
            };
            table | ((tile_idx as u16) << 4) | dy as u16
        }
    }

    fn is_in_scanline(&self, sprite_y: u8) -> bool {
        sprite_y <= self.scanline as u8 && (self.scanline as u8) < sprite_y + self.sprite_height()
    }

    fn is_8x8sprite(&self) -> bool {
        !self.ctrl.contains(PpuCtrl::SpriteHeightMode)
    }

    fn sprite_height(&self) -> u8 {
        if self.is_8x8sprite() {
            8
        } else {
            16
        }
    }
}
