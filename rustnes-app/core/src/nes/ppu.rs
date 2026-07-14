mod bg_sr;
mod loopy;
mod palette;
mod register;

use self::{
    bg_sr::{BgLatches, BgShiftRegisters},
    loopy::LoopyRegister,
    palette::PaletteRam,
    register::{PpuCtrl, PpuMask, PpuRegister, PpuStatus},
};

const SYSTEM_PALETTE: &[u8] = include_bytes!("/workspaces/rustnes/assets/palette/2C02G_U_wiki.pal");

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

/// The graphic device of the emulator.
#[derive(Debug, Clone, Copy)]
pub struct Ppu {
    display: [u8; 256 * 240 * 4],

    scanline: usize,
    cycle: usize,
    is_odd_frame: bool,
    previous_nmi: bool,

    reg: PpuRegister,
    lpy: LoopyRegister,
    bg_latches: BgLatches,
    bg_sr: BgShiftRegisters,

    oam: [u8; 256],
    palette: PaletteRam,
}

impl Ppu {
    /// Creates a new [`Ppu`] in its power-on state.
    pub fn new() -> Self {
        Self {
            display: [0u8; 256 * 240 * 4],
            scanline: 261,
            cycle: 0,
            is_odd_frame: false,
            previous_nmi: false,
            reg: PpuRegister::new(),
            lpy: LoopyRegister::new(),
            bg_latches: BgLatches::new(),
            bg_sr: BgShiftRegisters::new(),
            oam: [0u8; 256],
            palette: PaletteRam::new(),
        }
    }

    /// Resets the PPU.
    ///
    /// See: [NesDev - PPU power up state](https://www.nesdev.org/wiki/PPU_power_up_state)
    pub fn reset(&mut self) {
        self.is_odd_frame = false;
        self.previous_nmi = false;

        self.reg.reset();
        self.lpy.reset();
    }

    /// Returns the array of color index.
    pub fn display_buffer(&self) -> &[u8; 256 * 240 * 4] {
        &self.display
    }

    pub fn cpu_step(&mut self, bus: &mut impl Bus) {
        for _ in 0..3 {
            self.tick(bus);
        }
    }

    pub fn nmi_active(&self) -> bool {
        let is_vblank = self.reg.stat.contains(PpuStatus::V);
        let nmi_enabled = self.reg.ctrl.contains(PpuCtrl::V);
        is_vblank && nmi_enabled
    }

    pub fn read_ppustat(&mut self) -> u8 {
        self.lpy.clear_w();
        let byte = self.reg.stat.bits();
        self.reg.stat.remove(PpuStatus::V);
        byte
    }

    pub fn read_oamdata(&mut self) -> u8 {
        self.oam[self.reg.oam_addr as usize]
    }

    pub fn read_ppudata(&mut self, bus: &mut impl Bus) -> u8 {
        let addr = self.lpy.v.as_u16();
        let byte = if (0x0000..0x3F00).contains(&addr) {
            self.reg.ppu_data_buf
        } else {
            self.palette.read(addr & 0x1F)
        };

        // VRAM is buffered regardless of the read destination
        self.reg.ppu_data_buf = bus.read(addr);

        // increment loopy value
        self.lpy.v.increment(self.reg.ctrl.get_increment());

        byte
    }

    pub fn write_ppuctrl(&mut self, data: u8) {
        self.reg.ctrl = PpuCtrl::from_bits_truncate(data);
        self.lpy.t.set_nametable(data & 0x3);
    }

    pub fn write_ppumask(&mut self, data: u8) {
        self.reg.mask = PpuMask::from_bits_truncate(data);
    }

    pub fn write_oamaddr(&mut self, data: u8) {
        self.reg.oam_addr = data
    }

    pub fn write_oamdata(&mut self, data: u8) {
        self.oam[self.reg.oam_addr as usize] = data;
        self.reg.oam_addr = self.reg.oam_addr.wrapping_add(1);
    }

    pub fn write_ppuscrl(&mut self, data: u8) {
        self.lpy.write_ppuscroll(data)
    }

    pub fn write_ppuaddr(&mut self, data: u8) {
        self.lpy.write_ppuaddr(data)
    }

    pub fn write_ppudata(&mut self, bus: &mut impl Bus, data: u8) {
        let addr = self.lpy.v.as_u16();
        if (0x0000..0x3F00).contains(&addr) {
            bus.write(addr, data);
        } else {
            self.palette.write(addr & 0x1F, data & 0x3F);
        }

        //  Increment loopy value
        self.lpy.v.increment(self.reg.ctrl.get_increment());
    }

    // Cycles the PPU
    pub fn tick(&mut self, bus: &mut impl Bus) {
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
                    1 => self.bg_latches.tile_idx = bus.read(self.lpy.v.nametable_addr()),
                    3 => {
                        let at_byte = bus.read(self.lpy.v.attribute_addr());
                        self.bg_latches.at = self.lpy.v.extract_attribute_from_byte(at_byte);
                    }
                    5 => self.bg_latches.pt_low = bus.read(self.bg_pattern_addr()),
                    7 => {
                        self.bg_latches.pt_high = bus.read(self.bg_pattern_addr().wrapping_add(8));

                        self.bg_sr.load(&self.bg_latches);
                        self.lpy.v.increment_coarse_x();
                    }
                    _ => (),
                },

                // Dummy fetch
                257..=320 | 337..=340 => match fetch_loop {
                    1 => self.bg_latches.tile_idx = bus.read(self.lpy.v.nametable_addr()), // Unused
                    3 => {
                        bus.read(self.lpy.v.attribute_addr()); // Ignored
                    }
                    _ => (),
                },

                _ => unreachable!(),
            }
        }

        // Special events
        match (self.scanline, self.cycle) {
            // Start VBlank
            (241, 1) => self.reg.stat.insert(PpuStatus::V),

            // End VBlank
            (261, 1) => {
                self.reg.stat.remove(PpuStatus::V);
                self.reg.stat.remove(PpuStatus::S);
                self.reg.stat.remove(PpuStatus::O);
            }

            // End Scanline
            (_, 256) if during_fetch => self.lpy.v.increment_fine_y(),

            // Reset horizontal
            (_, 257) if during_fetch => self.lpy.copy_horizontal(),

            // Reset vertical
            (261, c) if (280..=304).contains(&c) => self.lpy.copy_vertical(),

            // Skip the last cycle of the pre-render line (odd frame only)
            (261, 339) if self.is_odd_frame => {
                self.is_odd_frame = false;
                self.scanline = 0;
                self.cycle = 0;
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

        let idx = (256 * y + x) * 4;
        let color = (color_idx * 3) as usize;
        self.display[idx] = SYSTEM_PALETTE[color];
        self.display[idx + 1] = SYSTEM_PALETTE[color + 1];
        self.display[idx + 2] = SYSTEM_PALETTE[color + 2];
        self.display[idx + 3] = 0xFF;
    }

    /// Returns the address of the pattern pointed by the v register and its control flags.
    fn bg_pattern_addr(&self) -> u16 {
        let fine_y = self.lpy.v.fine_y() as u16;
        let tile_idx = self.bg_latches.tile_idx as u16;
        self.reg.ctrl.bg_pattern_base() | (tile_idx << 4) | fine_y
    }

    /// Advances internal cycle and scanline counters handling frame transitions.
    fn advance_timing(&mut self) {
        if self.cycle == 340 {
            // Next frame
            self.cycle = 0;
            self.is_odd_frame ^= true; // Toggle odd/even frame
            self.scanline = match self.scanline {
                261 => 0,
                n => n + 1,
            };
        } else {
            self.cycle += 1;
        }
    }
}
