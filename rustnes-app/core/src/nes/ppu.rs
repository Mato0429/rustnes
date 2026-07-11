mod flags;
mod palette;

use flags::{PpuCtrl, PpuMask, PpuRegister, PpuStat};
use palette::{Palette, PALETTE_SIZE};

const DISPLAY_HEIGHT: usize = 256;
const DISPLAY_WIDTH: usize = 240;

#[derive(Debug)]
pub struct Ppu {
    display: [u8; 256 * 240],

    scanline: usize,
    cycle: usize,
    odd_frame: bool,

    reg: PpuRegister,

    lpy: LoopyRegister,
    bg_latches: BgLatches,
    bg_sr: BgShiftRegisters,

    oam: [u8; 256],
    palette: Palette,
}

impl Ppu {
    /// Creates a new [`Ppu`] in its power-on state.
    pub fn new() -> Self {
        Self {
            display: [0u8; 256 * 240],
            scanline: 261,
            cycle: 0,
            odd_frame: false,

            reg: PpuRegister {
                ctrl: PpuCtrl::empty(),
                mask: PpuMask::empty(),
                stat: PpuStat::empty(),
                oamaddr: 0x00,
                ppudata: 0x00,
            },

            lpy: LoopyRegister::new(),
            bg_latches: BgLatches::new(),
            bg_sr: BgShiftRegisters::new(),
            oam: [0u8; 256],
            palette: Palette::new([0x00; PALETTE_SIZE]),
        }
    }

    /// Resets the PPU.
    ///
    /// See: [NesDev - PPU power up state](https://www.nesdev.org/wiki/PPU_power_up_state)
    pub fn reset(&mut self) {
        self.odd_frame = false;

        self.reg.ctrl = PpuCtrl::empty();
        self.reg.mask = PpuMask::empty();
        self.reg.ppudata = 0x00;
    }

    /// Returns the array of color index.
    pub fn display(&self) -> [u8; 256 * 240] {
        self.display
    }

    pub fn nmi_active(&mut self) -> bool {
        let is_vblank = self.reg.stat.contains(PpuStat::V);
        let nmi_enabled = self.reg.ctrl.contains(PpuCtrl::V);
        is_vblank && nmi_enabled
    }

    fn advance_timing(&mut self) {
        if self.cycle == 340 {
            // Next frame
            self.cycle = 0;
            self.odd_frame ^= true; // Toggle odd/even frame
            self.scanline = match self.scanline {
                261 => 0,
                n => n + 1,
            };
        } else {
            self.cycle += 1;
        }
    }

    pub fn read_ppustat(&mut self) -> u8 {
        self.lpy.clear_w();
        let byte = self.reg.stat.bits();
        self.reg.stat.remove(PpuStat::V);
        byte
    }

    pub fn read_oamdata(&mut self) -> u8 {
        self.oam[self.reg.oamaddr as usize]
    }

    pub fn read_ppudata(&mut self) -> u8 {
        let addr = self.lpy.v.as_u16();
        let byte = if (0x0000..0x3F00).contains(&addr) {
            self.reg.ppudata
        } else {
            self.palette.read(addr & 0x1F)
        };

        // VRAM is buffered regardless of the read destination
        self.reg.ppudata = self.bus.read(addr);

        // increment loopy value
        self.lpy.v.increment(self.reg.ctrl.get_increment());

        byte
    }

    pub fn write_ppuctrl(&mut self, data: u8) {
        self.reg.ctrl = PpuCtrl::from_bits_truncate(data);
        self.lpy.t.set_nametable(data & 0x3);
    }

    pub fn write_ppumask(&mut self, data: u8) {
        self.reg.mask = PpuMask::from_bits_truncate(data)
    }

    pub fn write_oamaddr(&mut self, data: u8) {
        self.reg.oamaddr = data;
    }

    pub fn write_oamdata(&mut self, data: u8) {
        self.oam[self.reg.oamaddr as usize] = data;
        self.reg.oamaddr = self.reg.oamaddr.wrapping_add(1);
    }

    pub fn write_ppuscrl(&mut self, data: u8) {
        self.lpy.write_ppuscroll(data);
    }

    pub fn write_ppuaddr(&mut self, data: u8) {
        self.lpy.write_ppuaddr(data);
    }

    pub fn write_ppudata(&mut self, data: u8) {
        let addr = self.lpy.v.as_u16();
        if (0x0000..0x3F00).contains(&addr) {
            self.bus.write(addr, data);
        } else {
            self.palette.write(addr & 0x1F, data & 0x3F);
        }

        //  Increment loopy value
        self.lpy.v.increment(self.reg.ctrl.get_increment());
    }

    pub fn cpu_step(&mut self) {
        // TODO: Use subcycle for other PPU region
        for _ in 0..3 {
            self.tick();
        }
    }

    // Cycles the PPU
    pub fn tick(&mut self) {
        // Idle cycle, Early return
        if self.cycle == 0 {
            self.advance_timing();
            return;
        }

        // Draw background pixel during Visible
        if (0..=239).contains(&self.scanline) && (1..=256).contains(&self.cycle) {
            let pallet_addr = self.bg_sr.palette_address(15 - self.lpy.fine_x());
            let bg_color_idx = self.palette.read(pallet_addr);
            self.render_pixel(bg_color_idx);
            self.bg_sr.shift();
        }

        // Fetch Tables during Visible/Pre-render lines
        let during_fetch = !(240..=260).contains(&self.scanline);
        if during_fetch {
            // This loops from 0 to 7 ignoring the first idle cycle
            let fetch_loop = (self.cycle - 1) & 0x7;

            match self.cycle {
                // Valid fetch
                1..=256 | 321..=336 => match fetch_loop {
                    1 => self.bg_latches.tile_idx = self.bus.read(self.lpy.v.nametable_addr()),
                    3 => {
                        let at_byte = self.bus.read(self.lpy.v.attribute_addr());
                        self.bg_latches.at = self.lpy.v.extract_attribute_from_byte(at_byte);
                    }
                    5 => self.bg_latches.pt_low = self.bus.read(self.bg_pattern_addr()),
                    7 => {
                        self.bg_latches.pt_high =
                            self.bus.read(self.bg_pattern_addr().wrapping_add(8));

                        self.bg_sr.load(&self.bg_latches);
                        self.lpy.v.increment_coarse_x();
                    }
                    _ => (),
                },

                // Dummy fetch
                257..=320 | 337..=340 => match fetch_loop {
                    1 => self.bg_latches.tile_idx = self.bus.read(self.lpy.v.nametable_addr()), // Unused
                    3 => {
                        self.bus.read(self.lpy.v.attribute_addr()); // Ignored
                    }
                    _ => (),
                },

                _ => unreachable!(),
            }
        }

        // Special events
        match (self.scanline, self.cycle) {
            // Start VBlank
            (241, 1) => self.reg.stat.insert(PpuStat::V),

            // End VBlank
            (261, 1) => {
                self.reg.stat.remove(PpuStat::V);
                self.reg.stat.remove(PpuStat::S);
                self.reg.stat.remove(PpuStat::O);
            }

            // End Scanline
            (_, 256) if during_fetch => self.lpy.v.increment_fine_y(),

            // Reset horizontal
            (_, 257) if during_fetch => self.lpy.copy_horizontal(),

            // Reset vertical
            (261, c) if (280..=304).contains(&c) => self.lpy.copy_vertical(),

            // Skip the last cycle of the pre-render line (odd frame only)
            (261, 339) if self.odd_frame => {
                self.scanline = 0;
                self.cycle = 0;
                self.odd_frame = false;
                return;
            }

            _ => (),
        };

        self.advance_timing();
    }

    /// Renders a pixel at the current scanline and cycle coordinates.
    ///
    /// The pixel is only rendered if the scanline and cycle are
    /// within the visible area of the screen (Scanline 0-239, Cycle 1-256).
    fn render_pixel(&mut self, color_idx: u8) {
        let (x, y) = match (self.cycle, self.scanline) {
            (1..=256, 0..=239) => (self.cycle - 1, self.scanline),
            _ => return,
        };

        let idx = 256 * y + x;
        self.display[idx] = color_idx;
    }

    /// Returns the address of the pattern pointed by the v register and its control flags.
    fn bg_pattern_addr(&self) -> u16 {
        let fine_y = self.lpy.v.fine_y() as u16;
        let tile_idx = self.bg_latches.tile_idx as u16;
        self.reg.ctrl.bg_pattern_base() | (tile_idx << 4) | fine_y
    }
}
