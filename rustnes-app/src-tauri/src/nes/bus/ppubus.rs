pub const VRAM_SIZE: usize = 0x800;

/// # Mapping
/// | Address          | Size     | Device      | Description                    |
/// |:-----------------|:---------|:------------|:-------------------------------|
/// | `0x0000..0x2000` | `0x2000` | PT(Mapper)  | **Panics if remapped to VRAM** |
/// | `0x2000..0x3000` | `0x1000` | NT(Mapper)  | May be remapped to VRAM        |
/// | `0x3000..0x3F00` |          | NT(Mirror)  | `0x2000..0x2EFF`               |
/// | `0x3F00..0x4000` | `0x0100` | Palette RAM | **Panics on access**           |
/// | `0x4000..`       | `0xC000` | OutOfScope  | **Panics on access**           |
#[derive(Debug)]
pub struct PpuBus<'a> {
    pub openbus: &'a mut u8,
    pub vram: &'a mut [u8; VRAM_SIZE],
}

impl<'a> PpuBus<'a> {
    pub fn read(&mut self, addr: u16) -> u8 {
        todo!()
    }

    pub fn write(&mut self, addr: u16, data: u8) {}
}
