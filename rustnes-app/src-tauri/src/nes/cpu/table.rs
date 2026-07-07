#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Mnemonic {
    /* Unique */ JAM, BRK, RTI, RTS, JSR, JMP, PHA, PHP, PLA, PLP,
    /* Branch */ BCC, BCS, BNE, BEQ, BVC, BVS, BPL, BMI,
    /* Short  */ INX, INY, DEX, DEY, CLC, CLD, CLI, CLV, SEC, SED, SEI, TAX, TAY, TSX, TXA, TYA, TXS,
    /* Read   */ NOP, ADC, SBC, AND, ORA, EOR, BIT, CMP, CPX, CPY, LDA, LDX, LDY, LAX, LXA, LAS, ALR, ARR, ANC, AXS, ANE,
    /* Modify */ ASL, LSR, ROL, ROR, INC, DEC, DCP, ISB, RRA, RLA, SLO, SRE,
    /* Write  */ STA, STY, SAX, STX, SHA, SHX, SHY, SHS,
}
