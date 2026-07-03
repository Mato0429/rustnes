pub const VRAM_SIZE: usize = 0x800;

pub struct PpuBus<'a> {
    openbus: &'a mut u8,
    vram: &'a mut [u8; VRAM_SIZE],
}

#[derive(Debug, Clone, Copy)]
pub struct Ppu {}

impl Ppu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn reset(&mut self) {}

    pub fn tick(&mut self, bus: &mut PpuBus) {}
}
