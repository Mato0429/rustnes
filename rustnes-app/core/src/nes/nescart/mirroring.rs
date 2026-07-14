#[derive(Debug, Clone, Copy)]
pub enum VramTarget {
    Internal,
    External,
}

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen0,
    SingleScreen1,
    FourScreen,
}

impl Mirroring {
    /// Resolves the specified address using the current mirroring mode.
    ///
    /// # Panics
    ///
    /// Panics if the specified address is outside the range `0x2000..0x3000`.
    /// This is because the address that triggers the panic must be handled by the PPU bus.
    pub fn resolve(&self, bus_addr: u16) -> (u16, VramTarget) {
        debug_assert!(
            (0x2000..=0x2FFF).contains(&bus_addr),
            "address out of range: {bus_addr:#06X}"
        );

        let idx = (bus_addr & 0x0C00) >> 10; // NameTable index
        let addr = bus_addr & 0x03FF; // Address within NameTable

        const NAMETABLE_SIZE: u16 = 0x400;

        match (self, idx) {
            (Self::Horizontal, 0 | 1) => (addr, VramTarget::Internal),
            (Self::Horizontal, 2 | 3) => (addr + NAMETABLE_SIZE, VramTarget::Internal),

            (Self::Vertical, 0 | 2) => (addr, VramTarget::Internal),
            (Self::Vertical, 1 | 3) => (addr + NAMETABLE_SIZE, VramTarget::Internal),

            (Self::SingleScreen0, _) => (addr, VramTarget::Internal),
            (Self::SingleScreen1, _) => (addr + NAMETABLE_SIZE, VramTarget::Internal),

            (Self::FourScreen, 0) => (addr, VramTarget::Internal),
            (Self::FourScreen, 1) => (addr + NAMETABLE_SIZE, VramTarget::Internal),
            (Self::FourScreen, 2) => (addr, VramTarget::External),
            (Self::FourScreen, 3) => (addr + NAMETABLE_SIZE, VramTarget::External),

            _ => unreachable!(),
        }
    }
}
