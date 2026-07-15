mod background;
mod palette;
mod pixline;
mod registers;

use palette::Palette;
use pixline::{BgPixLiner, PixLine};
use registers::{PpuCtrl, PpuMask, PpuScrl, PpuStat};

const SYSTEM_PALETTE: &[u8] = include_bytes!("/workspaces/rustnes/assets/palette/2C02G_U_wiki.pal");

const DISPLAY_WIDTH: usize = 256;
const DISPLAY_HEIGHT: usize = 240;

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Debug, Clone, Copy)]
pub struct Ppu {
    display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4],
    scanline: usize,
    cycle: usize,
    odd_frame: bool,
    addr_latch: u16,

    palette: Palette,
    primary_oam: [u8; 256],
    secondary_oam: [u8; 32],

    ppudata_buffer: u8,
    write_toggle: bool,
    ppudata: u8,
    oamaddr: u8,
    ctrl: PpuCtrl,
    mask: PpuMask,
    stat: PpuStat,
    scrl: PpuScrl,

    bg_line: PixLine,
    bg_liner: BgPixLiner,
}

impl Ppu {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            display: [0xFF; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4],
            scanline: 261,
            cycle: 0,
            odd_frame: false,
            addr_latch: 0x0000,

            palette: Palette::new(),
            primary_oam: [0xFF; 256],
            secondary_oam: [0xFF; 32],

            ppudata_buffer: 0x00,
            write_toggle: false,
            ppudata: 0x00,
            oamaddr: 0x00,
            ctrl: PpuCtrl::empty(),
            mask: PpuMask::empty(),
            stat: PpuStat::empty(),
            scrl: PpuScrl {
                v: 0x0000,
                t: 0x0000,
                fine_x: 0x0,
            },

            bg_line: PixLine::default(),
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
        self.ppudata = 0x00;
    }

    pub fn display_buffer(&self) -> &[u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4] {
        &self.display
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

    fn render_pixel(&mut self) {
        let bg_pal = self.bg_liner.pixel_index(self.scrl.fine_x);
        let backdrop_bg =
            bg_pal & 0x03 == 0 || !self.mask.contains(PpuMask::ShowLeftmostBg) && self.cycle <= 8;
        let bg_pal = if backdrop_bg { 0x00 } else { bg_pal };
        self.bg_liner.shift();

        let color_idx = self.palette.read(bg_pal);
        self.set_pixel(self.cycle - 1, self.scanline, color_idx);
    }

    fn set_pixel(&mut self, x: usize, y: usize, color_idx: u8) {
        if let (0..DISPLAY_WIDTH, 0..DISPLAY_HEIGHT) = (x, y) {
            let syspal_idx = color_idx as usize * 3;
            let display_idx = (DISPLAY_WIDTH * y + x) * 4;

            self.display[display_idx] = SYSTEM_PALETTE[syspal_idx]; // R
            self.display[display_idx + 1] = SYSTEM_PALETTE[syspal_idx + 1]; // G
            self.display[display_idx + 2] = SYSTEM_PALETTE[syspal_idx + 2]; // B
            self.display[display_idx + 3] = 0xFF; // A
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
        // TODO: update display buffer
        self.cycle = 0;
        self.scanline = 0;
        self.odd_frame ^= true; // Toggle odd/even frame
    }
}
