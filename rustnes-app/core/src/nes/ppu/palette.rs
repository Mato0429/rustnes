pub const PALETTE_SIZE: usize = 0x20;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    inner: [u8; PALETTE_SIZE],
}

impl Palette {
    /// Creates a new `PaletteRam` in its power-on state.
    pub fn new(inner: [u8; PALETTE_SIZE]) -> Self {
        Self { inner }
    }

    /// Reads a color index.
    pub fn read(&self, palette_addr: u8) -> u8 {
        self.inner[Self::resolve_address(palette_addr)]
    }

    /// Writes a color index.
    pub fn write(&mut self, palette_addr: u8, data: u8) {
        self.inner[Self::resolve_address(palette_addr)] = data;
    }

    /// Resolves palette mirroring.
    #[inline]
    fn resolve_address(palette_addr: u8) -> usize {
        match palette_addr & 0x1F {
            0x10 => 0x00,
            0x14 => 0x04,
            0x18 => 0x08,
            0x1C => 0x0C,
            x => x as usize,
        }
    }
}
