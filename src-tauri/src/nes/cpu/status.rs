use bitflags::bitflags;

bitflags! {
    /// Manages the CPU status flags.
    ///
    /// Reference:
    /// - [NesDev - CPU power up state](https://www.nesdev.org/wiki/CPU_power_up_state)
    /// - [NesDev - Status flags](https://www.nesdev.org/wiki/Status_flags)
    /// - [NesDev - The B flag](https://www.nesdev.org/wiki/Status_flags#The_B_flag)
    #[derive(Clone, Copy, Debug)]
    pub struct StatusRegister: u8 {
        /// Bit 0: Carry
        const C = 0x01;

        /// Bit 1: Zero
        const Z = 0x02;

        /// Bit 2: Interrupt
        const I = 0x04;

        /// Bit 3: Decimal
        const D = 0x08;

        /// Bit 6: Overflow
        const V = 0x40;

        /// Bit 7: Negative
        const N = 0x80;
    }
}

/// An alias for [`StatusRegister`], use for bitwise operations and flag checks.
pub type Flags = StatusRegister;

impl StatusRegister {
    const B: u8 = 0x10;
    const R: u8 = 0x20;

    /// Creates a new `StatusRegister` in its power-on state.
    pub fn new() -> Self {
        Self::from_bits_retain(0x24)
    }

    /// Generates a byte to be pushed onto the stack from the `StatusRegister`.
    ///
    /// `set_b_flag` controls whether the B flag (Bit 4) is set in the pushed byte.
    /// This should be `true` for BRK/PHP, and false for hardware interrupts(NMI/IRQ).
    /// The R flag (Bit 5) is `1` by force.
    pub fn as_stack_byte(&self, set_b_flag: bool) -> u8 {
        let with_b = if set_b_flag {
            self.bits() | Self::B
        } else {
            self.bits() & !Self::B
        };

        with_b | Self::R
    }

    /// Updates the `StatusRegister` with a byte popped from the stack.
    ///
    /// The B flag (Bit 4) and The R flag (Bit 5) are not updated.
    /// This is because these bits do not exist as physical flags within the StatusRegister.
    pub fn set_from_stack_byte(&mut self, val: u8) {
        *self = Self::from_bits_truncate(val);
    }
}
