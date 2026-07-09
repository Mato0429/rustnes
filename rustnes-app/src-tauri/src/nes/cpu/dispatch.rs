use super::ZERO_PAGE;
use super::{Addressing, Mnemonic};
use super::{Bus, Cpu};

// TODO: impl AccessType for mnemonics

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Operand {
    Implied,
    Accumulator,
    Immediate(u8),
    Relative(i8),
    FixedPage(u16),
    SamePage(u16),
    DiffPage(u16),
}

impl Cpu {
    pub(super) fn exec_opcode<B: Bus>(&mut self, bus: &mut B, opcode: (Mnemonic, Addressing)) {
        let exception: Option<fn(&mut Self, &mut B)> = match opcode {
            (Mnemonic::BRK, Addressing::Implied) => Some(Self::brk_implied),
            (Mnemonic::RTS, Addressing::Implied) => Some(Self::rts_implied),
            (Mnemonic::RTI, Addressing::Implied) => Some(Self::rti_implied),
            (Mnemonic::PHA, Addressing::Implied) => Some(Self::pha_implied),
            (Mnemonic::PHP, Addressing::Implied) => Some(Self::php_implied),
            (Mnemonic::PLA, Addressing::Implied) => Some(Self::pla_implied),
            (Mnemonic::PLP, Addressing::Implied) => Some(Self::plp_implied),
            (Mnemonic::JSR, Addressing::Absolute) => Some(Self::jsr_absolute),
            (Mnemonic::JMP, Addressing::Absolute) => Some(Self::jmp_absolute),
            (Mnemonic::JMP, Addressing::Indirect) => Some(Self::jmp_indirect),
            _ => None,
        };

        if let Some(exception_fn) = exception {
            exception_fn(self, bus);
        } else {
            let (mnemonic, addressing) = opcode;
            let operand = self.decode_operand(bus, addressing);
        }
    }

    fn decode_operand(&mut self, bus: &mut impl Bus, addressing: Addressing) -> Operand {
        match addressing {
            Addressing::Implied => {
                self.read_at_pc(bus);
                Operand::Implied
            }

            Addressing::Accumulator => {
                self.read_at_pc(bus);
                Operand::Accumulator
            }

            Addressing::Immediate => {
                let value = self.fetch(bus);
                Operand::Immediate(value)
            }

            Addressing::Relative => {
                let offset = self.fetch(bus) as i8;
                Operand::Relative(offset)
            }

            Addressing::ZeroPage => {
                let lo = self.fetch(bus);
                let base = u16::from_le_bytes([lo, ZERO_PAGE]);
                Operand::FixedPage(base)
            }

            Addressing::ZeroPageX => {
                let base = self.fetch(bus);
                self.read(bus, u16::from_le_bytes([base, ZERO_PAGE]));
                let indexed = u16::from_le_bytes([base.wrapping_add(self.reg.x), ZERO_PAGE]);
                Operand::FixedPage(indexed)
            }

            Addressing::ZeroPageY => {
                let base = self.fetch(bus);
                self.read(bus, u16::from_le_bytes([base, ZERO_PAGE]));
                let indexed = u16::from_le_bytes([base.wrapping_add(self.reg.y), ZERO_PAGE]);
                Operand::FixedPage(indexed)
            }

            Addressing::Absolute => {
                let lo = self.fetch(bus);
                let hi = self.fetch(bus);
                let base = u16::from_le_bytes([lo, hi]);
                Operand::FixedPage(base)
            }

            Addressing::AbsoluteX => {
                let lo = self.fetch(bus);
                let hi = self.fetch(bus);

                let (lo, is_crossed) = lo.overflowing_add(self.reg.x);
                let indexed = u16::from_le_bytes([lo, hi]);
                if is_crossed {
                    Operand::DiffPage(indexed)
                } else {
                    Operand::SamePage(indexed)
                }
            }

            Addressing::AbsoluteY => {
                let lo = self.fetch(bus);
                let hi = self.fetch(bus);

                let (lo, is_crossed) = lo.overflowing_add(self.reg.y);
                let indexed = u16::from_le_bytes([lo, hi]);
                if is_crossed {
                    Operand::DiffPage(indexed)
                } else {
                    Operand::SamePage(indexed)
                }
            }

            Addressing::XIdxedInd => {
                let ptr = self.fetch(bus);
                let idxed_ptr = ptr.wrapping_add(self.reg.x);
                self.read(bus, u16::from_le_bytes([ptr, ZERO_PAGE])); // dummy read to index

                let vec_lo = u16::from_le_bytes([idxed_ptr, ZERO_PAGE]);
                let vec_hi = u16::from_le_bytes([idxed_ptr.wrapping_add(1), ZERO_PAGE]);
                let lo = self.read(bus, vec_lo);
                let hi = self.read(bus, vec_hi);

                let addr = u16::from_le_bytes([lo, hi]);
                Operand::FixedPage(addr)
            }

            Addressing::IndYIdxed => {
                let ptr = self.fetch(bus);

                let vec_lo = u16::from_le_bytes([ptr, ZERO_PAGE]);
                let vec_hi = u16::from_le_bytes([ptr.wrapping_add(1), ZERO_PAGE]);
                let lo = self.read(bus, vec_lo);
                let hi = self.read(bus, vec_hi);

                let (idxed_lo, is_crossed) = lo.overflowing_add(self.reg.y);
                let indexed = u16::from_le_bytes([idxed_lo, hi]);
                if is_crossed {
                    Operand::DiffPage(indexed)
                } else {
                    Operand::SamePage(indexed)
                }
            }

            Addressing::Indirect => unreachable!(),
        }
    }
}
