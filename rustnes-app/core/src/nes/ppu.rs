mod background;
mod palette;
mod registers;
mod sprite;

use background::{BgPixLine, BgPixLiner};
use palette::Palette;
use registers::{PpuCtrl, PpuMask, PpuScrl, PpuStat};
use sprite::SprPixLine;

const SYSTEM_PALETTE: &[u8] = include_bytes!("./ppu/2C02G_U_wiki.pal");

const FRAME_WIDTH: usize = 256;
const FRAME_HEIGHT: usize = 240;

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Debug, Clone)]
pub struct Ppu {
    frontframe: Vec<u8>,
    backframe: Vec<u8>,
    is_frame_ready: bool,
    scanline: usize,
    cycle: usize,
    odd_frame: bool,
    addr_latch: u16,

    palette: Palette,
    primary_oam: [u8; 256],
    secondary_oam: [u8; 32],
    spr_lines: [SprPixLine; 8],

    ppudata_buffer: u8,
    write_toggle: bool,
    oamaddr: u8,
    ctrl: PpuCtrl,
    mask: PpuMask,
    stat: PpuStat,
    scrl: PpuScrl,

    bg_line: BgPixLine,
    bg_liner: BgPixLiner,
}

impl Ppu {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            frontframe: vec![0x00; FRAME_WIDTH * FRAME_HEIGHT],
            backframe: vec![0x00; FRAME_WIDTH * FRAME_HEIGHT],
            is_frame_ready: false,
            scanline: 261,
            cycle: 0,
            odd_frame: false,
            addr_latch: 0x0000,

            palette: Palette::new(),
            primary_oam: [0x00; 256],
            secondary_oam: [0x00; 32],
            spr_lines: [SprPixLine::default(); 8],

            ppudata_buffer: 0x00,
            write_toggle: false,
            oamaddr: 0x00,
            ctrl: PpuCtrl::empty(),
            mask: PpuMask::empty(),
            stat: PpuStat::empty(),
            scrl: PpuScrl {
                v: 0x0000,
                t: 0x0000,
                fine_x: 0x0,
            },

            bg_line: BgPixLine::default(),
            bg_liner: BgPixLiner::default(),
        }
    }

    /// Resets the PPU.
    ///
    /// See: [NesDev - PPU power up state](https://www.nesdev.org/wiki/PPU_power_up_state)
    pub fn reset(&mut self) {
        self.odd_frame = false;
        self.write_toggle = false;
        self.ctrl = PpuCtrl::empty();
        self.mask = PpuMask::empty();
        self.ppudata_buffer = 0x00;
    }

    pub fn output_frame(&mut self, buffer: &mut [u8]) {
        for (i, &color) in self.frontframe.iter().enumerate() {
            let pal_idx = (color * 3) as usize;
            buffer[i * 4] = SYSTEM_PALETTE[pal_idx];
            buffer[i * 4 + 1] = SYSTEM_PALETTE[pal_idx + 1];
            buffer[i * 4 + 2] = SYSTEM_PALETTE[pal_idx + 2];
            buffer[i * 4 + 3] = 0xFF;
        }
    }

    pub fn nmi_active(&self) -> bool {
        self.stat.contains(PpuStat::Vblank) && self.ctrl.contains(PpuCtrl::NmiEnable)
    }

    pub fn cpu_step(&mut self, bus: &mut impl Bus) {
        // TODO: Use subcycle for other PPU region
        for _ in 0..3 {
            self.tick(bus);
        }
    }

    // Cycles the PPU
    pub fn tick(&mut self, bus: &mut impl Bus) {
        let enable_bg = self.mask.contains(PpuMask::EnableBgRendering);
        let enable_spr = self.mask.contains(PpuMask::EnableSprRendering);

        if enable_bg || enable_spr {
            if let (0..=239, 1..=256) = (self.scanline, self.cycle) {
                self.render_pixel();
            }
        }

        if enable_bg {
            self.advance_background_pipeline(bus);
        }

        if enable_spr {
            self.advance_sprite_pipeline(bus);
        }

        if let (241, 1) = (self.scanline, self.cycle) {
            self.stat.insert(PpuStat::Vblank);
        }

        if let (261, 1) = (self.scanline, self.cycle) {
            self.stat.remove(PpuStat::Vblank);
            self.stat.remove(PpuStat::Sprite0Hit);
            self.stat.remove(PpuStat::SpriteOverflow);
        }

        self.advance_cycle();
    }

    pub(super) fn is_frame_ready(&mut self) -> bool {
        let res = self.is_frame_ready;
        self.is_frame_ready = false;
        res
    }

    fn render_pixel(&mut self) {
        let bg_pal = self.bg_liner.pixel_index(self.scrl.fine_x);
        let hide_bg = !self.mask.contains(PpuMask::ShowLeftmostBg) && self.cycle <= 8;
        let bg_color = if hide_bg { 0x00 } else { bg_pal & 0x03 };

        let mut spr_pal = 0x00;
        let mut priority = 0;
        for sprite in self.spr_lines {
            let dx = (self.cycle as isize - 1) - (sprite.x as isize);

            if (0..=7).contains(&dx) {
                let p0 = ((sprite.pt_lo << dx) & 0x80 != 0) as u8;
                let p1 = ((sprite.pt_hi << dx) & 0x80 != 0) as u8;
                let pt = (p1 << 1) | p0;
                if pt != 0x00 {
                    spr_pal = 0x10 | ((sprite.attr & 0x3) << 2) | pt;
                    priority = (sprite.attr & 0x20) >> 5;
                    break;
                }
            }
        }
        let hide_spr = !self.mask.contains(PpuMask::ShowLeftmostSpr) && self.cycle <= 8;
        let spr_color = if hide_spr { 0x00 } else { spr_pal & 0x03 };

        let pal = match (bg_color, spr_color, priority) {
            (0, 0, _) => 0x00,
            (0, _, _) => spr_pal,
            (_, 0, _) => bg_pal,
            (_, _, 0) => spr_pal,
            (_, _, 1) => bg_pal,
            _ => unreachable!(),
        };

        let color = self.palette.read(pal);
        self.set_pixel(self.cycle - 1, self.scanline, color);
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        if let (0..FRAME_WIDTH, 0..FRAME_HEIGHT) = (x, y) {
            let idx = FRAME_WIDTH * y + x;
            self.backframe[idx] = color;
        }
    }

    fn latch_addr(&mut self, addr: u16) {
        self.addr_latch = addr;
    }

    fn fetch(&mut self, bus: &mut impl Bus) -> u8 {
        bus.read(self.addr_latch & 0x7FFF)
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
        self.odd_frame ^= true; // Toggle odd/even frame
        self.cycle = 0;
        self.scanline = 0;
        self.is_frame_ready = true;
        std::mem::swap(&mut self.frontframe, &mut self.backframe);
    }
}
