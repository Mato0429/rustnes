use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PpuCtrl: u8 {
        /// VRAM address increment per CPU read/write to PPUDATA
        /// (0: add 1, going across; 1: add 32, going down)
        const I = 0x04;

        /// 8x8 sprite pattern table addr
        /// (0: 0x0000, 1: 0x1000) ignored in 8x16 mode
        const S = 0x08;

        /// Background pattern table addr (0: 0x0000, 1: 0x1000)
        const B = 0x10;

        /// Sprite size (0: 8x8, 1: 8x16)
        const H = 0x20;

        /// PPU Master/Slave select
        /// (0: read backdrop from EXT pins; 1: output color on EXT pins)
        const P = 0x40;

        /// Vblank NMI enable
        const V = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PpuMask: u8 {
        /// Grayscale enable
        const Gray = 0x01;

        /// 1: Show background in leftmost 8pxs of screen, 0: Hide
        const LeftBG = 0x02;

        /// 1: Show sprites in leftmost 8pxs of screen, 0: Hide
        const LeftSpr = 0x04;

        /// Enable background rendering
        const RenderBG = 0x08;

        /// Enable sprite rendering
        const RenderSpr = 0x10;

        /// Emphasize red
        const EmpR = 0x20;

        /// Emphasize green
        const EmpG = 0x40;

        /// Emphasize blue
        const EmpB = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PpuStat: u8 {
        /// Sprite overflow flag
        const O = 0x20;

        /// Sprite 0 hit flag
        const S = 0x40;

        /// Vblank flag, cleared on read.
        const V = 0x80;
    }
}

/// Holds the PPU's rendering state and control flags.
#[derive(Debug, Clone, Copy)]
pub struct PpuRegister {
    pub ctrl: PpuCtrl,
    pub mask: PpuMask,
    pub stat: PpuStat,
    pub oamaddr: u8,
    pub ppudata: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct LoopyRegister {
    pub v: u16,
    pub t: u16,
    pub fine_x: u8,
    pub w: bool,
}
