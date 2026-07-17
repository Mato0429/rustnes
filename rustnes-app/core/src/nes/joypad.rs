#[derive(Debug, Clone, Copy, Default)]
pub struct Joypad {
    current: u8,
    snapshot: u8,
}

impl Joypad {
    pub fn update(&mut self, v: u8) {
        self.current = v;
    }

    pub fn strobo(&mut self) {
        self.snapshot = self.current;
    }

    pub fn read(&mut self) -> u8 {
        let res = self.snapshot & 0x01;
        self.snapshot = 0x80 | (self.snapshot >> 1);
        res
    }
}
