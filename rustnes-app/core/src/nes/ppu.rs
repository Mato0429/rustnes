mod latches;
mod palette;
mod registers;

use latches::{BgTileLiner, TileLine};
use palette::{Palette, PALETTE_SIZE};
use registers::{LoopyRegister, PpuCtrl, PpuMask, PpuRegister, PpuStat};

const SYSTEM_PALETTE: &[u8] = include_bytes!("/workspaces/rustnes/assets/palette/2C02G_U_wiki.pal");

const DISPLAY_WIDTH: usize = 256;
const DISPLAY_HEIGHT: usize = 240;
const CHANNEL: usize = 4;

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Debug, Clone, Copy)]
pub struct Ppu {
    display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * CHANNEL],

    scanline: usize,
    cycle: usize,
    odd_frame: bool,
    addr_latch: u16,

    ppudata_buffer: u8,
    reg: PpuRegister,
    lpy: LoopyRegister,

    bg_tileline: TileLine,
    bg_tileliner: BgTileLiner,

    pub palette: Palette,
    primary_oam: [u8; 256],
    secondary_oam: [u8; 32],
}

impl Ppu {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            display: [0xFF; DISPLAY_WIDTH * DISPLAY_HEIGHT * CHANNEL],
            scanline: 261,
            cycle: 0,
            odd_frame: false,
            addr_latch: 0x0000,

            ppudata_buffer: 0x00,

            reg: PpuRegister {
                ctrl: PpuCtrl::empty(),
                mask: PpuMask::empty(),
                stat: PpuStat::empty(),
                oamaddr: 0x00,
                ppudata: 0x00,
            },

            lpy: LoopyRegister {
                v: 0x0000,
                t: 0x0000,
                fine_x: 0x00,
                w: false,
            },

            bg_tileline: TileLine::default(),
            bg_tileliner: BgTileLiner::default(),

            palette: Palette::new([0x00; PALETTE_SIZE]),
            primary_oam: [0xFF; 256],
            secondary_oam: [0xFF; 32],
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

    pub fn display_buffer(&self) -> &[u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * CHANNEL] {
        &self.display
    }

    pub fn nmi_active(&self) -> bool {
        let is_vblank = self.reg.stat.contains(PpuStat::V);
        let nmi_enabled = self.reg.ctrl.contains(PpuCtrl::V);
        is_vblank && nmi_enabled
    }

    pub fn read_ppustat(&mut self) -> u8 {
        self.lpy.w = false;
        let byte = self.reg.stat.bits();
        self.reg.stat.remove(PpuStat::V);
        byte
    }

    pub fn read_oamdata(&mut self) -> u8 {
        self.primary_oam[self.reg.oamaddr as usize]
    }

    pub fn read_ppudata(&mut self, bus: &mut impl Bus) -> u8 {
        let byte = if self.lpy.v < 0x3F00 {
            self.ppudata_buffer
        } else {
            self.palette.read((self.lpy.v & 0x1F) as u8)
        };

        // VRAM is buffered regardless of the read destination
        self.ppudata_buffer = bus.read(self.lpy.v);

        // increment loopy value
        self.lpy.v = self.lpy.v.wrapping_add(1);

        byte
    }

    pub fn write_ppuctrl(&mut self, data: u8) {
        self.reg.ctrl = PpuCtrl::from_bits_truncate(data);
        self.lpy.t = (self.lpy.t & 0x73FF) | ((data as u16 & 0x3) << 10); // set nametable
    }

    pub fn write_ppumask(&mut self, data: u8) {
        self.reg.mask = PpuMask::from_bits_truncate(data)
    }

    pub fn write_oamaddr(&mut self, data: u8) {
        self.reg.oamaddr = data;
    }

    pub fn write_oamdata(&mut self, data: u8) {
        self.primary_oam[self.reg.oamaddr as usize] = data;
        self.reg.oamaddr = self.reg.oamaddr.wrapping_add(1);
    }

    pub fn write_ppuscrl(&mut self, data: u8) {
        print!(
            "A {:} {:}:{:}:{:}   ",
            data, self.lpy.w, self.lpy.v, self.lpy.t
        );
        if !self.lpy.w {
            // first write
            self.lpy.fine_x = data & 0x07;
            self.lpy.t = (self.lpy.t & 0x7FE0) | (data as u16 >> 3); // set coarseX
        } else {
            // second write
            self.lpy.t = (self.lpy.t & 0x0FFF) | ((data as u16 & 0x07) << 12); // set fineY
            self.lpy.t = (self.lpy.t & 0x7C1F) | ((data as u16 & 0xF8) << 2) // set coarseY
        }

        self.lpy.w ^= true; // toggle latch
        println!("{:}:{:}", self.lpy.v, self.lpy.t);
    }

    pub fn write_ppuaddr(&mut self, data: u8) {
        print!(
            "B {:} {:}:{:}:{:}   ",
            data, self.lpy.w, self.lpy.v, self.lpy.t
        );
        if !self.lpy.w {
            // first write
            self.lpy.t = (self.lpy.t & 0x00FF) | (((data as u16) & 0x3F) << 8);
        } else {
            // second write
            self.lpy.t = (self.lpy.t & 0xFF00) | (data as u16);
            self.lpy.v = self.lpy.t;
        }

        self.lpy.w ^= true; // toggle latch
        println!("{:}:{:}", self.lpy.v, self.lpy.t);
    }

    pub fn write_ppudata(&mut self, bus: &mut impl Bus, data: u8) {
        print!(
            "C {:} {:}:{:}:{:}   ",
            data, self.lpy.w, self.lpy.v, self.lpy.t
        );
        if 0x3EFF < self.lpy.v {
            self.palette.write((self.lpy.v & 0x1F) as u8, data)
        } else {
            bus.write(self.lpy.v & 0x2FFF, data);
        }

        //  Increment loopy value
        let incflag = self.reg.ctrl.contains(PpuCtrl::I);
        let incremnet = if incflag { 32 } else { 1 };
        self.lpy.v = self.lpy.v.wrapping_add(incremnet);

        println!("{:}:{:}", self.lpy.v, self.lpy.t);
    }

    pub fn cpu_step(&mut self, bus: &mut impl Bus) {
        // TODO: Use subcycle for other PPU region
        for _ in 0..3 {
            self.tick(bus);
        }
    }

    // Cycles the PPU
    pub fn tick(&mut self, bus: &mut impl Bus) {
        let render_bg = self.reg.mask.contains(PpuMask::RenderBG);
        let render_sprite = self.reg.mask.contains(PpuMask::RenderSpr);

        if !render_bg && !render_sprite {
            self.advance_cycle();
            return;
        }

        self.render_pixel();

        if self.scanline == 261 && self.cycle == 304 {
            println!("t:{:}, v:{:}", self.lpy.t, self.lpy.v);
        }

        // Ppu I/O
        match (self.scanline, self.cycle) {
            (0..=239 | 261, 1..=256 | 321..=336) => self.bg_tile_fetch(bus),
            (0..=239 | 261, 257..=320) => (), // TODO: sprite pattern fetch
            (0..=239 | 261, 337..=340) => (), // TODO: dummy read
            _ => (),
        }

        // Ppu events
        match (self.scanline, self.cycle) {
            // VBlank start
            (241, 1) => self.reg.stat.insert(PpuStat::V),

            // Horizontal reset
            (0..=239 | 261, 256) => self.increment_fine_y(),
            (0..=239 | 261, 257) => {
                // copy horizontal
                self.lpy.v = (self.lpy.v & 0x7FE0) | (self.lpy.t & 0x001F); // copy coarseX
                self.lpy.v = (self.lpy.v & 0x3BFF) | (self.lpy.t & 0x0400); // copy nametable lo
            }

            // VBlank end
            (261, 1) => {
                self.reg.stat.remove(PpuStat::V);
                self.reg.stat.remove(PpuStat::S);
                self.reg.stat.remove(PpuStat::O);
            }

            // Vertical reset
            (261, 280..=304) => {
                // copy vertical
                self.lpy.v = (self.lpy.v & 0x0FFF) | (self.lpy.t & 0x7000); // copy fineY
                self.lpy.v = (self.lpy.v & 0x7C1F) | (self.lpy.t & 0x03E0); // copy coarseY
                self.lpy.v = (self.lpy.v & 0x77FF) | (self.lpy.t & 0x0800); // copy nametable hi
            }

            _ => (),
        }

        self.advance_cycle();
    }

    fn latch_addr(&mut self, addr: u16) {
        self.addr_latch = addr;
    }

    fn fetch(&mut self, bus: &mut impl Bus) -> u8 {
        bus.read(self.addr_latch)
    }

    fn advance_cycle(&mut self) {
        if self.cycle == 340 {
            self.reset_scanline();
        } else {
            self.cycle += 1;
        }
    }

    fn reset_scanline(&mut self) {
        self.cycle = 0;
        if self.scanline == 261 {
            self.reset_frame();
        } else {
            self.scanline += 1
        }
    }

    fn reset_frame(&mut self) {
        // TODO: update display buffer
        self.cycle = 0;
        self.scanline = 0;
        self.odd_frame ^= true; // Toggle odd/even frame
    }

    fn increment_coarse_x(&mut self) {
        let coarse_x = self.lpy.v & 0x001F;
        if coarse_x >= 31 {
            self.lpy.v &= 0x7FE0; // set coarseX to 0
            self.lpy.v ^= 0x0400; // inverse nametable lo
        } else {
            self.lpy.v = (self.lpy.v & 0x7FE0) | coarse_x.wrapping_add(1)
        }
    }

    fn increment_fine_y(&mut self) {
        let fine_y = (self.lpy.v & 0x7000) >> 12;
        if fine_y >= 7 {
            self.lpy.v &= 0x0FFF; // set fineY to 0
            self.increment_coarse_y();
        } else {
            self.lpy.v = (self.lpy.v & 0x0FFF) | (fine_y.wrapping_add(1) << 12)
        }
    }

    fn increment_coarse_y(&mut self) {
        let coarse_y = (self.lpy.v & 0x03E0) >> 5;
        if coarse_y >= 29 {
            self.lpy.v &= 0x7C1F; // set coarseY to 0
            self.lpy.v ^= 0x0800; // inverse nametable hi
        } else {
            self.lpy.v = (self.lpy.v & 0x7C1F) | (coarse_y.wrapping_add(1) << 5)
        }
    }

    fn bg_tile_fetch(&mut self, bus: &mut impl Bus) {
        match ((self.cycle - 1) % 8) + 1 {
            // nametable fetch
            1 => self.latch_addr(self.bg_nt_addr()),
            2 => self.bg_tileline.tile_idx = self.fetch(bus),

            // attribute fetch
            3 => self.latch_addr(self.bg_at_addr()),
            4 => {
                let at_byte = self.fetch(bus);
                let [lo, hi] = self.extract_at_from_byte(at_byte);
                self.bg_tileline.at_lo = lo;
                self.bg_tileline.at_hi = hi;
            }

            // pattern lo fetch
            5 => self.latch_addr(self.bg_pt_addr()),
            6 => self.bg_tileline.pt_lo = self.fetch(bus),

            // pattern hi fetch
            7 => self.latch_addr(self.bg_pt_addr().wrapping_add(8)),
            8 => {
                self.bg_tileline.pt_hi = self.fetch(bus);
                self.bg_tileliner.load_tileline(self.bg_tileline);
                self.increment_coarse_x();
            }
            _ => unreachable!(),
        }
    }

    fn bg_nt_addr(&self) -> u16 {
        0x2000 | (self.lpy.v & 0x0FFF)
    }

    fn bg_at_addr(&self) -> u16 {
        0x23C0 | (self.lpy.v & 0x0C00) | ((self.lpy.v >> 4) & 0x38) | ((self.lpy.v >> 2) & 0x07)
    }

    fn bg_pt_addr(&self) -> u16 {
        let baseflag = self.reg.ctrl.contains(PpuCtrl::B);
        let base = if baseflag { 0x1000 } else { 0x0000 };
        let fine_y = (self.lpy.v & 0x7000) >> 12;
        base | ((self.bg_tileline.tile_idx as u16) << 4) | fine_y
    }

    fn extract_at_from_byte(&self, byte: u8) -> [bool; 2] {
        let coarse_y = (self.lpy.v & 0x03E0) >> 5;
        let coarse_x = self.lpy.v & 0x001F;
        let shifted = byte >> (((coarse_y & 0x2) << 1) | coarse_x & 0x2);
        let lo = shifted & 0x01 != 0;
        let hi = shifted & 0x02 != 0;
        [lo, hi]
    }

    fn set_pixel(&mut self, x: usize, y: usize, pixel_idx: u8) {
        let display_idx = (DISPLAY_WIDTH * y + x) * CHANNEL;
        let color_idx = self.palette.read(pixel_idx) as usize * 3;
        let r = SYSTEM_PALETTE[color_idx];
        let g = SYSTEM_PALETTE[color_idx + 1];
        let b = SYSTEM_PALETTE[color_idx + 2];

        self.display[display_idx] = r;
        self.display[display_idx + 1] = g;
        self.display[display_idx + 2] = b;
        self.display[display_idx + 3] = 0xFF;
    }

    fn render_pixel(&mut self) {
        let in_fetch_cycle = matches!(self.cycle, 1..=256 | 321..=336);
        let in_render_scanline = matches!(self.scanline, 0..=239 | 261);

        if in_fetch_cycle && in_render_scanline {
            let bg_pixel_idx = self.bg_tileliner.pixel_index(self.lpy.fine_x);
            self.bg_tileliner.shift();

            if let (1..=256, 0..=239) = (self.cycle, self.scanline) {
                let x = self.cycle - 1;
                let y = self.scanline;
                self.set_pixel(x, y, bg_pixel_idx);
            }
        }
    }
}
