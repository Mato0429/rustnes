use bitflags::bitflags;

use super::{Bus, Ppu};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PpuCtrl: u8 {
        /// VRAM address increment per CPU read/write to PPUDATA
        /// (0: add 1, going across; 1: add 32, going down)
        const IncrementMode = 0x04;

        /// 8x8 sprite pattern table addr
        /// (0: 0x0000, 1: 0x1000) ignored in 8x16 mode
        const SprPtTableSelect = 0x08;

        /// Background pattern table addr (0: 0x0000, 1: 0x1000)
        const BgPtTableSelect = 0x10;

        /// Sprite size (0: 8x8, 1: 8x16)
        const SpriteHeightMode = 0x20;

        /// PPU Master/Slave select
        /// (0: read backdrop from EXT pins; 1: output color on EXT pins)
        const IsSlave = 0x40;

        /// Vblank NMI enable
        const NmiEnable = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PpuMask: u8 {
        /// Grayscale enable
        const Grayscale = 0x01;

        /// 1: Show background in leftmost 8pxs of screen, 0: Hide
        const ShowLeftmostBg = 0x02;

        /// 1: Show sprites in leftmost 8pxs of screen, 0: Hide
        const ShowLeftmostSpr = 0x04;

        /// Enable background rendering
        const EnableBgRendering = 0x08;

        /// Enable sprite rendering
        const EnableSprRendering = 0x10;

        /// Emphasize red
        const EmphasizeRed = 0x20;

        /// Emphasize green
        const EmphasizeGreen = 0x40;

        /// Emphasize blue
        const EmphasizeBlue = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PpuStat: u8 {
        /// Sprite overflow flag
        const SpriteOverflow = 0x20;

        /// Sprite 0 hit flag
        const Sprite0Hit = 0x40;

        /// Vblank flag, cleared on read.
        const Vblank = 0x80;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PpuScrl {
    pub v: u16,
    pub t: u16,
    pub fine_x: u8,
}

impl Ppu {
    pub fn read_ppustat(&mut self) -> u8 {
        self.write_toggle = false;

        // DEBUG:
        if self.scanline == 30 && (self.cycle as isize - 130) < 7 {
            self.stat.insert(PpuStat::Sprite0Hit);
        }
        if self.stat.contains(PpuStat::Vblank) {
            self.stat.remove(PpuStat::Sprite0Hit);
        }

        let byte = self.stat.bits();
        self.stat.remove(PpuStat::Vblank);
        byte
    }

    pub fn read_oamdata(&mut self) -> u8 {
        self.primary_oam[self.oamaddr as usize]
    }

    pub fn read_ppudata(&mut self, bus: &mut impl Bus) -> u8 {
        let addr = self.scrl.v & 0x7FFF;
        let byte = if (0x0000..0x3F00).contains(&addr) {
            self.ppudata
        } else {
            self.palette.read((addr & 0x1F) as u8)
        };

        // VRAM is buffered regardless of the read destination
        self.ppudata_buffer = bus.read(self.scrl.v % 0x3F00);

        self.increment_loopy();
        byte
    }

    pub fn write_ppuctrl(&mut self, data: u8) {
        self.ctrl = PpuCtrl::from_bits_truncate(data);
        // set nametable
        self.scrl.t = (self.scrl.t & 0x73FF) | ((data as u16 & 0x3) << 10);
    }

    pub fn write_ppumask(&mut self, data: u8) {
        self.mask = PpuMask::from_bits_truncate(data)
    }

    pub fn write_oamaddr(&mut self, data: u8) {
        self.oamaddr = data;
    }

    pub fn write_oamdata(&mut self, data: u8) {
        self.primary_oam[self.oamaddr as usize] = data;
        self.oamaddr = self.oamaddr.wrapping_add(1);
    }

    pub fn write_ppuscrl(&mut self, data: u8) {
        if !self.write_toggle {
            // first write
            // set fineX, coarseX
            self.scrl.fine_x = data & 0x07;
            self.scrl.t = (self.scrl.t & 0x7FE0) | (data as u16 >> 3);
        } else {
            // second write
            // set fineY, coarseX
            self.scrl.t = (self.scrl.t & 0x0FFF) | ((data as u16 & 0x07) << 12);
            self.scrl.t = (self.scrl.t & 0x7C1F) | ((data as u16 & 0xF8) << 2);
        }

        self.write_toggle ^= true; // toggle latch
    }

    pub fn write_ppuaddr(&mut self, data: u8) {
        if !self.write_toggle {
            // first write
            self.scrl.t = (self.scrl.t & 0x00FF) | (((data as u16) & 0x3F) << 8);
        } else {
            // second write
            self.scrl.t = (self.scrl.t & 0xFF00) | (data as u16);
            self.scrl.v = self.scrl.t;
        }

        self.write_toggle ^= true; // toggle latch
    }

    pub fn write_ppudata(&mut self, bus: &mut impl Bus, data: u8) {
        let addr = self.scrl.v & 0x3FFF;
        if (0x0000..0x3F00).contains(&addr) {
            bus.write(addr, data);
        } else {
            self.palette.write((addr & 0x1F) as u8, data & 0x3F);
        }

        self.increment_loopy();
    }

    fn increment_loopy(&mut self) {
        let incflag = self.ctrl.contains(PpuCtrl::IncrementMode);
        let incremnet = if incflag { 32 } else { 1 };
        self.scrl.v = self.scrl.v.wrapping_add(incremnet);
    }
}
