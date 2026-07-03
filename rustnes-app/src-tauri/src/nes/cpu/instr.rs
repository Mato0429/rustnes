use maplit::hashmap;
use std::collections::HashMap;
use std::sync::LazyLock;

// These will not be supported in the future:
// ANE, LXA, SHA, SHX, SHY, SHS

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[allow(clippy::upper_case_acronyms)]
pub enum Mnemonic {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Addressing {
    Implied,
    Accumulator,
    Immediate,
    Relative,
    Zeropage,
    ZeropageX,
    ZeropageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    XIndexedIndirect,
    IndirectYIndexed,
    AbsoluteIndirect,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Opcode {
    mnemonic: Mnemonic,
    addressing: Addressing,
}

use Addressing::*;
use Mnemonic::*;

pub const OPCODE_TABLE: LazyLock<HashMap<u8, Opcode>> = LazyLock::new(|| {
    hashmap! {
        0x00 => Opcode { mnemonic: BRK, addressing: Implied },
        0x01 => Opcode { mnemonic: ORA, addressing: XIndexedIndirect },
        0x05 => Opcode { mnemonic: ORA, addressing: Zeropage },
        0x06 => Opcode { mnemonic: ASL, addressing: Zeropage },
        0x08 => Opcode { mnemonic: PHP, addressing: Implied },
        0x09 => Opcode { mnemonic: ORA, addressing: Immediate },
        0x0A => Opcode { mnemonic: ASL, addressing: Accumulator },
        0x0D => Opcode { mnemonic: ORA, addressing: Absolute },
        0x0E => Opcode { mnemonic: ASL, addressing: Absolute },

        0x10 => Opcode { mnemonic: BPL, addressing: Relative },
        0x11 => Opcode { mnemonic: ORA, addressing: IndirectYIndexed },
        0x15 => Opcode { mnemonic: ORA, addressing: ZeropageX },
        0x16 => Opcode { mnemonic: ASL, addressing: ZeropageX },
        0x18 => Opcode { mnemonic: CLC, addressing: Implied },
        0x19 => Opcode { mnemonic: ORA, addressing: AbsoluteY },
        0x1D => Opcode { mnemonic: ORA, addressing: AbsoluteX },
        0x1E => Opcode { mnemonic: ASL, addressing: AbsoluteX },

        0x20 => Opcode { mnemonic: JSR, addressing: Absolute },
        0x21 => Opcode { mnemonic: AND, addressing: XIndexedIndirect },
        0x24 => Opcode { mnemonic: BIT, addressing: Zeropage },
        0x25 => Opcode { mnemonic: AND, addressing: Zeropage },
        0x26 => Opcode { mnemonic: ROL, addressing: Zeropage },
        0x28 => Opcode { mnemonic: PLP, addressing: Implied },
        0x29 => Opcode { mnemonic: AND, addressing: Immediate },
        0x2A => Opcode { mnemonic: ROL, addressing: Accumulator },
        0x2C => Opcode { mnemonic: BIT, addressing: Absolute },
        0x2D => Opcode { mnemonic: AND, addressing: Absolute },
        0x2E => Opcode { mnemonic: ROL, addressing: Absolute },

        0x30 => Opcode { mnemonic: BMI, addressing: Relative },
        0x31 => Opcode { mnemonic: AND, addressing: IndirectYIndexed },
        0x35 => Opcode { mnemonic: AND, addressing: ZeropageX },
        0x36 => Opcode { mnemonic: ROL, addressing: ZeropageX },
        0x38 => Opcode { mnemonic: SEC, addressing: Implied },
        0x39 => Opcode { mnemonic: AND, addressing: AbsoluteY },
        0x3D => Opcode { mnemonic: AND, addressing: AbsoluteX },
        0x3E => Opcode { mnemonic: ROL, addressing: AbsoluteX },

        0x40 => Opcode { mnemonic: RTI, addressing: Implied },
        0x41 => Opcode { mnemonic: EOR, addressing: XIndexedIndirect },
        0x45 => Opcode { mnemonic: EOR, addressing: Zeropage },
        0x46 => Opcode { mnemonic: LSR, addressing: Zeropage },
        0x48 => Opcode { mnemonic: PHA, addressing: Implied },
        0x49 => Opcode { mnemonic: EOR, addressing: Immediate },
        0x4A => Opcode { mnemonic: LSR, addressing: Accumulator },
        0x4C => Opcode { mnemonic: JMP, addressing: Absolute },
        0x4D => Opcode { mnemonic: EOR, addressing: Absolute },
        0x4E => Opcode { mnemonic: LSR, addressing: Absolute },

        0x50 => Opcode { mnemonic: BVC, addressing: Relative },
        0x51 => Opcode { mnemonic: EOR, addressing: IndirectYIndexed },
        0x55 => Opcode { mnemonic: EOR, addressing: ZeropageX },
        0x56 => Opcode { mnemonic: LSR, addressing: ZeropageX },
        0x58 => Opcode { mnemonic: CLI, addressing: Implied },
        0x59 => Opcode { mnemonic: EOR, addressing: AbsoluteY },
        0x5D => Opcode { mnemonic: EOR, addressing: AbsoluteX },
        0x5E => Opcode { mnemonic: LSR, addressing: AbsoluteX },

        0x60 => Opcode { mnemonic: RTS, addressing: Implied },
        0x61 => Opcode { mnemonic: ADC, addressing: XIndexedIndirect },
        0x65 => Opcode { mnemonic: ADC, addressing: Zeropage },
        0x66 => Opcode { mnemonic: ROR, addressing: Zeropage },
        0x68 => Opcode { mnemonic: PLA, addressing: Implied },
        0x69 => Opcode { mnemonic: ADC, addressing: Immediate },
        0x6A => Opcode { mnemonic: ROR, addressing: Accumulator },
        0x6C => Opcode { mnemonic: JMP, addressing: AbsoluteIndirect },
        0x6D => Opcode { mnemonic: ADC, addressing: Absolute },
        0x6E => Opcode { mnemonic: ROR, addressing: Absolute },

        0x70 => Opcode { mnemonic: BVS, addressing: Relative },
        0x71 => Opcode { mnemonic: ADC, addressing: IndirectYIndexed },
        0x75 => Opcode { mnemonic: ADC, addressing: ZeropageX },
        0x76 => Opcode { mnemonic: ROR, addressing: ZeropageX },
        0x78 => Opcode { mnemonic: SEI, addressing: Implied },
        0x79 => Opcode { mnemonic: ADC, addressing: AbsoluteY },
        0x7D => Opcode { mnemonic: ADC, addressing: AbsoluteX },
        0x7E => Opcode { mnemonic: ROR, addressing: AbsoluteX },

        0x81 => Opcode { mnemonic: STA, addressing: XIndexedIndirect },
        0x84 => Opcode { mnemonic: STY, addressing: Zeropage },
        0x85 => Opcode { mnemonic: STA, addressing: Zeropage },
        0x86 => Opcode { mnemonic: STX, addressing: Zeropage },
        0x88 => Opcode { mnemonic: DEY, addressing: Implied },
        0x8A => Opcode { mnemonic: TXA, addressing: Implied },
        0x8C => Opcode { mnemonic: STY, addressing: Absolute },
        0x8D => Opcode { mnemonic: STA, addressing: Absolute },
        0x8E => Opcode { mnemonic: STX, addressing: Absolute },

        0x90 => Opcode { mnemonic: BCC, addressing: Relative },
        0x91 => Opcode { mnemonic: STA, addressing: IndirectYIndexed },
        0x94 => Opcode { mnemonic: STY, addressing: ZeropageX },
        0x95 => Opcode { mnemonic: STA, addressing: ZeropageX },
        0x96 => Opcode { mnemonic: STX, addressing: ZeropageY },
        0x98 => Opcode { mnemonic: TYA, addressing: Implied },
        0x99 => Opcode { mnemonic: STA, addressing: AbsoluteY },
        0x9A => Opcode { mnemonic: TXS, addressing: Implied },
        0x9D => Opcode { mnemonic: STA, addressing: AbsoluteX },

        0xA0 => Opcode { mnemonic: LDY, addressing: Immediate },
        0xA1 => Opcode { mnemonic: LDA, addressing: XIndexedIndirect },
        0xA2 => Opcode { mnemonic: LDX, addressing: Immediate },
        0xA4 => Opcode { mnemonic: LDY, addressing: Zeropage },
        0xA5 => Opcode { mnemonic: LDA, addressing: Zeropage },
        0xA6 => Opcode { mnemonic: LDX, addressing: Zeropage },
        0xA8 => Opcode { mnemonic: TAY, addressing: Implied },
        0xA9 => Opcode { mnemonic: LDA, addressing: Immediate },
        0xAA => Opcode { mnemonic: TAX, addressing: Implied },
        0xAC => Opcode { mnemonic: LDY, addressing: Absolute },
        0xAD => Opcode { mnemonic: LDA, addressing: Absolute },
        0xAE => Opcode { mnemonic: LDX, addressing: Absolute },

        0xB0 => Opcode { mnemonic: BCS, addressing: Relative },
        0xB1 => Opcode { mnemonic: LDA, addressing: IndirectYIndexed },
        0xB4 => Opcode { mnemonic: LDY, addressing: ZeropageX },
        0xB5 => Opcode { mnemonic: LDA, addressing: ZeropageX },
        0xB6 => Opcode { mnemonic: LDX, addressing: ZeropageY },
        0xB8 => Opcode { mnemonic: CLV, addressing: Implied },
        0xB9 => Opcode { mnemonic: LDA, addressing: AbsoluteY },
        0xBA => Opcode { mnemonic: TSX, addressing: Implied },
        0xBC => Opcode { mnemonic: LDY, addressing: AbsoluteX },
        0xBD => Opcode { mnemonic: LDA, addressing: AbsoluteX },
        0xBE => Opcode { mnemonic: LDX, addressing: AbsoluteY },

        0xC0 => Opcode { mnemonic: CPY, addressing: Immediate },
        0xC1 => Opcode { mnemonic: CMP, addressing: XIndexedIndirect },
        0xC4 => Opcode { mnemonic: CPY, addressing: Zeropage },
        0xC5 => Opcode { mnemonic: CMP, addressing: Zeropage },
        0xC6 => Opcode { mnemonic: DEC, addressing: Zeropage },
        0xC8 => Opcode { mnemonic: INY, addressing: Implied },
        0xC9 => Opcode { mnemonic: CMP, addressing: Immediate },
        0xCA => Opcode { mnemonic: DEX, addressing: Implied },
        0xCC => Opcode { mnemonic: CPY, addressing: Absolute },
        0xCD => Opcode { mnemonic: CMP, addressing: Absolute },
        0xCE => Opcode { mnemonic: DEC, addressing: Absolute },

        0xD0 => Opcode { mnemonic: BNE, addressing: Relative },
        0xD1 => Opcode { mnemonic: CMP, addressing: IndirectYIndexed },
        0xD5 => Opcode { mnemonic: CMP, addressing: ZeropageX },
        0xD6 => Opcode { mnemonic: DEC, addressing: ZeropageX },
        0xD8 => Opcode { mnemonic: CLD, addressing: Implied },
        0xD9 => Opcode { mnemonic: CMP, addressing: AbsoluteY },
        0xDD => Opcode { mnemonic: CMP, addressing: AbsoluteX },
        0xDE => Opcode { mnemonic: DEC, addressing: AbsoluteX },

        0xE0 => Opcode { mnemonic: CPX, addressing: Immediate },
        0xE1 => Opcode { mnemonic: SBC, addressing: XIndexedIndirect },
        0xE4 => Opcode { mnemonic: CPX, addressing: Zeropage },
        0xE5 => Opcode { mnemonic: SBC, addressing: Zeropage },
        0xE6 => Opcode { mnemonic: INC, addressing: Zeropage },
        0xE8 => Opcode { mnemonic: INX, addressing: Implied },
        0xE9 => Opcode { mnemonic: SBC, addressing: Immediate },
        0xEA => Opcode { mnemonic: NOP, addressing: Implied },
        0xEC => Opcode { mnemonic: CPX, addressing: Absolute },
        0xED => Opcode { mnemonic: SBC, addressing: Absolute },
        0xEE => Opcode { mnemonic: INC, addressing: Absolute },

        0xF0 => Opcode { mnemonic: BEQ, addressing: Relative },
        0xF1 => Opcode { mnemonic: SBC, addressing: IndirectYIndexed },
        0xF5 => Opcode { mnemonic: SBC, addressing: ZeropageX },
        0xF6 => Opcode { mnemonic: INC, addressing: ZeropageX },
        0xF8 => Opcode { mnemonic: SED, addressing: Implied },
        0xF9 => Opcode { mnemonic: SBC, addressing: AbsoluteY },
        0xFD => Opcode { mnemonic: SBC, addressing: AbsoluteX },
        0xFE => Opcode { mnemonic: INC, addressing: AbsoluteX },
    }
});
