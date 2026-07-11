pub const PALETTE_SIZE: usize = 0x20;

#[derive(Debug)]
pub struct Palette {
    inner: [u8; 0x20],
}

impl Palette {
    /// Creates a new `PaletteRam` in its power-on state.
    pub fn new(inner: [u8; 0x20]) -> Self {
        Self { inner }
    }

    /// Reads a color index.
    pub fn read(&self, palette_addr: u16) -> u8 {
        self.inner[Self::resolve_address(palette_addr)]
    }

    /// Writes a color index.
    pub fn write(&mut self, palette_addr: u16, data: u8) {
        self.inner[Self::resolve_address(palette_addr)] = data;
    }

    /// Resolves palette mirroring.
    #[inline]
    fn resolve_address(palette_addr: u16) -> usize {
        let is_bg = palette_addr & 0x3 == 0;
        if is_bg {
            0x00
        } else {
            palette_addr as usize
        }
    }
}
