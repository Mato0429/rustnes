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
    pub fn read(&self, addr: u8) -> u8 {
        self.inner[Self::resolve_address(addr)]
    }

    /// Writes a color index.
    pub fn write(&mut self, addr: u8, data: u8) {
        self.inner[Self::resolve_address(addr)] = data;
    }

    /// Resolves palette mirroring.
    #[inline]
    fn resolve_address(addr: u8) -> usize {
        if addr & 0x03 == 0 {
            0x00
        } else {
            (addr & 0x1F) as usize
        }
    }
}
