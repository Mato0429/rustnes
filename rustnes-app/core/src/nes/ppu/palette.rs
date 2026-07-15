const PALETTE_SIZE: usize = 0x20;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    inner: [u8; PALETTE_SIZE],
}

impl Palette {
    /// Creates a new `PaletteRam` in its power-on state.
    pub fn new() -> Self {
        Self {
            inner: [0u8; PALETTE_SIZE],
        }
    }

    /// Reads a color index.
    pub fn read(&self, addr: u8) -> u8 {
        self.inner[Self::resolve_addr(addr) as usize] & 0x3F
    }

    /// Writes a color index.
    pub fn write(&mut self, addr: u8, data: u8) {
        self.inner[Self::resolve_addr(addr) as usize] = data & 0x3F;
    }

    /// Resolves palette mirroring.
    #[inline(always)]
    fn resolve_addr(addr: u8) -> u8 {
        let addr = addr & 0x1F;
        match addr {
            0x10 => 0x00,
            0x14 => 0x04,
            0x18 => 0x08,
            0x1C => 0x0C,
            _ => addr,
        }
    }
}
