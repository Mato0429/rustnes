use crate::nes::cpu::opcode::{Branch, Modify, Read, UnstableWrite, Write};

use super::opcode::{Short, Unique};
use super::ZERO_PAGE;
use super::{Addressing, Mnemonic};
use super::{Bus, Cpu};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Operand {
    Accumulator,
    Immediate(u8),
    FixedPage(u16),
    SamePage(u16),
    DiffPage(u16),
}

/*
enum Target {
    Depend (Long, Short, Relative, Indirect, Nop Implied, Undefined)
    Accumulator
    Immediate
    Zeropage
    ZeropageX
    ZeropageY
    Absolute
    AbsoluteX
    AbsoluteY
    XIndexed
    IndirectY
}
*/

impl Cpu {
    pub(super) fn exec_opcode<B: Bus>(&mut self, bus: &mut B, opcode: (Mnemonic, Addressing)) {
        let (mnemonic, addressing) = opcode;

        match mnemonic {
            Mnemonic::Unique(op) => match op {
                // NOP_IMPLIED here
                Unique::JAM => self.jam_undefined(bus),
                Unique::BRK => self.brk_implied(bus),
                Unique::RTI => self.rti_implied(bus),
                Unique::RTS => self.rts_implied(bus),
                Unique::PHA => self.pha_implied(bus),
                Unique::PHP => self.php_implied(bus),
                Unique::PLA => self.pla_implied(bus),
                Unique::PLP => self.plp_implied(bus),
                Unique::JSR => self.jsr_absolute(bus),
                Unique::JMP if matches!(addressing, Addressing::Absolute) => self.jmp_absolute(bus),
                Unique::JMP if matches!(addressing, Addressing::Indirect) => self.jmp_indirect(bus),
                _ => panic!(),
            },

            Mnemonic::Short(op) => {
                self.read_at_pc(bus);

                match op {
                    Short::INX => self.inx_implied(),
                    Short::INY => self.iny_implied(),
                    Short::DEX => self.dex_implied(),
                    Short::DEY => self.dey_implied(),
                    Short::CLC => self.clc_implied(),
                    Short::CLD => self.cld_implied(),
                    Short::CLI => self.cli_implied(),
                    Short::CLV => self.clv_implied(),
                    Short::SEC => self.sec_implied(),
                    Short::SED => self.sed_implied(),
                    Short::SEI => self.sei_implied(),
                    Short::TAX => self.tax_implied(),
                    Short::TAY => self.tay_implied(),
                    Short::TXA => self.txa_implied(),
                    Short::TYA => self.tya_implied(),
                    Short::TSX => self.tsx_implied(),
                    Short::TXS => self.txs_implied(),
                }
            }

            Mnemonic::Branch(op) => {
                let offset = self.fetch(bus) as i8;

                let is_branched = match op {
                    Branch::BCS => self.bcs_relative(),
                    Branch::BCC => self.bcc_relative(),
                    Branch::BNE => self.bne_relative(),
                    Branch::BEQ => self.beq_relative(),
                    Branch::BVS => self.bvs_relative(),
                    Branch::BVC => self.bvc_relative(),
                    Branch::BMI => self.bmi_relative(),
                    Branch::BPL => self.bpl_relative(),
                };

                if !is_branched {
                    return;
                }

                self.read_at_pc(bus);
                let [old_lo, old_hi] = self.reg.pc.to_le_bytes();
                let (new_lo, is_crossed) = old_lo.overflowing_add_signed(offset);
                self.reg.pc = u16::from_le_bytes([new_lo, old_hi]);

                if !is_crossed {
                    // TODO: remove irq pending to delay
                    return;
                }

                self.read_at_pc(bus); // read at wrong pc
                let hi_fixer = if offset > 0 { 1 } else { -1 };
                self.reg.pc = u16::from_le_bytes([new_lo, old_hi.wrapping_add_signed(hi_fixer)]);
            }

            Mnemonic::Read(op) => {
                let operand = self.fetch_operand(bus, addressing);

                let m = match operand {
                    Operand::Accumulator => self.reg.a,
                    Operand::Immediate(m) => m,
                    Operand::FixedPage(addr) | Operand::SamePage(addr) => self.read(bus, addr),
                    Operand::DiffPage(addr) => {
                        self.read(bus, addr); // dummy read to fix ADH
                        self.read(bus, addr.wrapping_add(0x0100))
                    }
                    _ => panic!(),
                };

                match op {
                    Read::NOP => self.nop(m),
                    Read::ADC => self.adc(m),
                    Read::SBC => self.sbc(m),
                    Read::AND => self.and(m),
                    Read::ORA => self.ora(m),
                    Read::EOR => self.eor(m),
                    Read::BIT => self.bit(m),
                    Read::CMP => self.cmp(m),
                    Read::CPX => self.cpx(m),
                    Read::CPY => self.cpy(m),
                    Read::LDA => self.lda(m),
                    Read::LDX => self.ldx(m),
                    Read::LDY => self.ldy(m),
                    Read::LAX => self.lax(m),
                    Read::LXA => self.lxa(m),
                    Read::LAS => self.las(m),
                    Read::ALR => self.alr(m),
                    Read::ARR => self.arr(m),
                    Read::ANC => self.anc(m),
                    Read::AXS => self.axs(m),
                    Read::ANE => self.ane(m),
                }
            }

            Mnemonic::Modify(op) => {
                let operand = self.fetch_operand(bus, addressing);

                let addr = match operand {
                    Operand::Accumulator => 0, // Accumulator has no address
                    Operand::FixedPage(addr) => addr,
                    Operand::SamePage(addr) => {
                        self.read(bus, addr);
                        addr
                    }
                    Operand::DiffPage(addr) => {
                        self.read(bus, addr); // dummy read to fix ADH
                        addr.wrapping_add(0x0100)
                    }
                    _ => panic!(),
                };

                let mut m = self.read(bus, addr);
                self.write(bus, addr, m); // write back

                match op {
                    Modify::ASL => self.asl(&mut m),
                    Modify::LSR => self.lsr(&mut m),
                    Modify::ROL => self.rol(&mut m),
                    Modify::ROR => self.ror(&mut m),
                    Modify::INC => self.inc(&mut m),
                    Modify::DEC => self.dec(&mut m),
                    Modify::DCP => self.dcp(&mut m),
                    Modify::ISB => self.isb(&mut m),
                    Modify::RRA => self.rra(&mut m),
                    Modify::RLA => self.rla(&mut m),
                    Modify::SLO => self.slo(&mut m),
                    Modify::SRE => self.sre(&mut m),
                }

                if let Operand::Accumulator = operand {
                    self.reg.a = m;
                } else {
                    self.write(bus, addr, m);
                }
            }

            Mnemonic::Write(op) => {
                let operand = self.fetch_operand(bus, addressing);

                let addr = match operand {
                    Operand::FixedPage(addr) => addr,
                    Operand::SamePage(addr) => {
                        self.read(bus, addr);
                        addr
                    }
                    Operand::DiffPage(addr) => {
                        self.read(bus, addr); // dummy read to fix ADH
                        addr.wrapping_add(0x0100)
                    }
                    _ => panic!(),
                };

                let m = match op {
                    Write::STA => self.sta(),
                    Write::STX => self.stx(),
                    Write::STY => self.sty(),
                    Write::SAX => self.sax(),
                };

                self.write(bus, addr, m);
            }

            Mnemonic::UnstableWrite(op) => {
                let operand = self.fetch_operand(bus, addressing);

                let addr = match operand {
                    Operand::SamePage(addr) | Operand::DiffPage(addr) => {
                        self.read(bus, addr);
                        addr
                    }
                    _ => panic!(),
                };

                let [adl, adh] = addr.to_le_bytes();
                let m = match op {
                    UnstableWrite::SHA => self.sha(adh),
                    UnstableWrite::SHX => self.shx(adh),
                    UnstableWrite::SHY => self.shy(adh),
                    UnstableWrite::SHS => self.shs(adh),
                };

                if let Operand::SamePage(addr) = operand {
                    self.write(bus, addr, m);
                } else {
                    self.write(bus, u16::from_le_bytes([adl, m]), m);
                }
            }
        }
    }

    fn fetch_operand(&mut self, bus: &mut impl Bus, addressing: Addressing) -> Operand {
        match addressing {
            Addressing::Accumulator => {
                self.read_at_pc(bus);
                Operand::Accumulator
            }

            Addressing::Immediate => {
                let value = self.fetch(bus);
                Operand::Immediate(value)
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

            _ => unreachable!(),
        }
    }
}
