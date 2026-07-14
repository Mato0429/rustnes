/// Stores color indices that point to the color in the system palette.
#[derive(Debug, Clone, Copy)]
pub struct PaletteRam([u8; 0x20]);

impl PaletteRam {
    /// Creates a new `PaletteRam` in its power-on state.
    pub fn new() -> Self {
        Self([0u8; 0x20])
    }

    /// Reads a color index.
    pub fn read(&self, palette_addr: u16) -> u8 {
        self.0[Self::resolve_address(palette_addr)]
    }

    /// Writes a color index.
    pub fn write(&mut self, palette_addr: u16, data: u8) {
        assert!(data & 0xC0 == 0, "invalid palette index: {data}");
        self.0[Self::resolve_address(palette_addr)] = data;
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
