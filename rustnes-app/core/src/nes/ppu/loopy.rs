use modular_bitfield::{self, bitfield, prelude::*};

/// Represents the PPU's internal 15-bit register (`0yyy NNYY YYYX XXXX`).
///
/// The 15 bits are partitioned to map directly to VRAM addresses during
/// different phases of the rendering pipeline (Nametable, Attribute, and Tile fetches).
/// It is also used for PPUADDR.
#[bitfield(bits = 15)]
#[derive(Clone, Copy, Debug)]
pub struct ScrollOffsets {
    pub coarse_x: B5,
    pub coarse_y: B5,
    pub nametable: B2,
    pub fine_y: B3,
}

impl ScrollOffsets {
    /// Returns the scroll offsets as a u16 value.
    ///
    /// bit 15 is reserved as `0`.
    pub fn as_u16(&self) -> u16 {
        u16::from_le_bytes(self.into_bytes())
    }

    /// Increments the scroll offsets by the specified value, while wrapping within the 15-bit range (0x7FFF).
    pub fn increment(&mut self, v: u16) {
        let res = self.as_u16().wrapping_add(v) & 0x7FFF;
        *self = Self::from_bytes(res.to_le_bytes());
    }

    /// Increments the Coarse X, wrapping and flipping the nametable X bit if the boundary is reached.
    pub fn increment_coarse_x(&mut self) {
        if self.coarse_x() == 31 {
            self.set_coarse_x(0);
            self.set_nametable(self.nametable() ^ 0x1); // inverse nametable x bit
        } else {
            self.set_coarse_x(self.coarse_x().wrapping_add(1) & 0x1F);
        }
    }

    /// Increments the Fine Y, wrapping and updating the Coarse Y and Nametable Y bit if the boundary is reached
    pub fn increment_fine_y(&mut self) {
        if self.fine_y() == 7 {
            self.set_fine_y(0);
            self.increment_coarse_y();
        } else {
            self.set_fine_y(self.fine_y().wrapping_add(1) & 0x7);
        }
    }

    /// Returns the address of the nametable pointed by the register.
    pub fn nametable_addr(&self) -> u16 {
        0x2000 | (self.as_u16() & 0x0FFF)
    }

    /// Returns the address of the attribute pointed by the register.
    pub fn attribute_addr(&self) -> u16 {
        let v = self.as_u16();
        0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
    }

    /// Extracts 2 bit attribute for the current tile from the specified attribute byte.
    ///
    /// An attribute byte controls a 32x32 pixel area (4x4 tiles),
    /// divided into four 16x16 pixel regions (2x2 tiles). Each region uses 2 bits of the byte.
    pub fn extract_attribute_from_byte(&self, byte: u8) -> u8 {
        let shift = ((self.coarse_y() & 0x2) << 1) | self.coarse_x() & 0x2;
        (byte >> shift) & 0x3
    }

    /// Increments the Coarse Y, wrapping and flipping the Nametable Y bit if the boundary is reached.
    fn increment_coarse_y(&mut self) {
        if self.coarse_y() >= 29 {
            self.set_coarse_y(0);
            self.set_nametable(self.nametable() ^ 0x2); // inverse nametable y bit
        } else {
            self.set_coarse_y(self.coarse_y().wrapping_add(1) & 0x1F);
        }
    }
}

/// Manages the PPU internal registers (v, t, x, w) for scrolling and VRAM addressing.
#[derive(Debug, Clone, Copy)]
pub struct LoopyRegister {
    pub v: ScrollOffsets,
    pub t: ScrollOffsets,
    fine_x: u8,
    w: bool,
}

impl LoopyRegister {
    /// Creates a new `LoopyRegister` in its power-on state.
    pub fn new() -> Self {
        Self {
            v: ScrollOffsets::new(),
            t: ScrollOffsets::new(),
            fine_x: 0,
            w: false,
        }
    }

    /// Resets the loopy register.
    pub fn reset(&mut self) {
        self.v = ScrollOffsets::new();
        self.t = ScrollOffsets::new();
        self.fine_x = 0;
        self.w = false;
    }

    /// Returns the value of `fine_x`.
    pub fn fine_x(&self) -> u8 {
        self.fine_x
    }

    /// Clears the `w` latch.
    pub fn clear_w(&mut self) {
        self.w = false;
    }

    /// Writes a byte as PPUSCROLL.
    pub fn write_ppuscroll(&mut self, data: u8) {
        if !self.w {
            // first write
            self.fine_x = data & 0x07;
            self.t.set_coarse_x(data >> 3);
        } else {
            // second write
            self.t.set_fine_y(data & 0x07);
            self.t.set_coarse_y(data >> 3);
        }
        self.w ^= true; // toggle latch
    }

    /// Writes a byte as PPUADDR.
    pub fn write_ppuaddr(&mut self, data: u8) {
        if !self.w {
            // first write
            let [low, _] = self.t.into_bytes();
            self.t = ScrollOffsets::from_bytes([low, data & 0x3F]);
        } else {
            // second write
            let [_, high] = self.t.into_bytes();
            self.t = ScrollOffsets::from_bytes([data, high]);
            self.v = self.t;
        };
        self.w ^= true; // toggle latch
    }

    /// Copies horizontal data from t to v.
    pub fn copy_horizontal(&mut self) {
        self.v.set_coarse_x(self.t.coarse_x());
        self.v
            .set_nametable((self.v.nametable() & 0x2) | (self.t.nametable() & 0x1));
    }

    /// Copies vertical data from t to v.
    pub fn copy_vertical(&mut self) {
        self.v.set_fine_y(self.t.fine_y());
        self.v.set_coarse_y(self.t.coarse_y());
        self.v
            .set_nametable((self.t.nametable() & 0x2) | (self.v.nametable() & 0x1));
    }
}
