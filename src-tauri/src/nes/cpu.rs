mod opcode;
mod status;

use crate::nes::{
    cpu::{
        opcode::{Addressing, Branch, Implicit, UniqueCycle, OPCODE_TABLE},
        status::{Flags, StatusRegister},
    },
    types::*,
};
use opcode::Operator;

#[allow(clippy::upper_case_acronyms)]
enum Destination {
    Discard,
    OpcodeLatch,
    DataLatch,
    Execute,
    AddressLow,
    AddressHigh,
    PCLow,
    PCHigh,
}

enum Phase {
    FetchOpcode,
    ResolveAddressing(u8),
    ExcuteInstruction(u8),
    HandleInterrupt,
}

pub enum IoRequest {
    Read(u16),
    Write(u16, u8),
}

pub struct Cpu {
    is_jammed: bool,

    pc: CrossingAddr,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    p: StatusRegister,

    dst: Destination,
    ad: CrossingAddr,
    dl: u8,

    phase: Phase,
    operator: Operator,
    addressing: Addressing,
}

impl Cpu {
    pub fn tick(&mut self) -> IoRequest {
        match self.phase {
            Phase::FetchOpcode => self.fetch_opcode(),

            Phase::ResolveAddressing(c) => {
                let (res, done) = self.resolve_addressing(c);
                if done {
                    self.phase = Phase::ExcuteInstruction(1)
                } else {
                    self.phase = Phase::ResolveAddressing(c + 1);
                }
                res
            }

            Phase::ExcuteInstruction(c) => {
                let (res, done) = self.exec_instruction(c);
                if done {
                    self.phase = Phase::FetchOpcode
                } else {
                    self.phase = Phase::ExcuteInstruction(c + 1);
                }
                res
            }
        }
    }

    pub fn tock(&mut self, v: u8) {
        match self.dst {
            Destination::Discard => (),
            Destination::OpcodeLatch => {
                let (operator, addressing) = OPCODE_TABLE[v as usize];
                self.operator = operator;
                self.addressing = addressing
            }
            Destination::Execute => {
                if let Phase::ExcuteInstruction(c) = self.phase {
                    self.exec_instruction(c);
                } else {
                    unreachable!();
                }
            }
            Destination::DataLatch => self.dl = v,
            Destination::AddressLow => self.ad.set_low(v),
            Destination::AddressHigh => self.ad.set_high(v),
            Destination::PCLow => self.pc.set_low(v),
            Destination::PCHigh => self.pc.set_high(v),
        };
    }

    fn fetch_opcode(&mut self) -> IoRequest {
        self.phase = Phase::ResolveAddressing(1);
        let res = self.issue_read(*self.pc, Destination::OpcodeLatch);
        *self.pc = self.pc.wrapping_add(1);
        res
    }

    fn issue_read(&mut self, addr: u16, dst: Destination) -> IoRequest {
        self.dst = dst;
        IoRequest::Read(addr)
    }

    fn issue_write(&self, addr: u16, data: u8) -> IoRequest {
        IoRequest::Write(addr, data)
    }

    fn update_nz(&mut self, v: u8) {
        self.p.set(Flags::N, v & 0x80 != 0);
        self.p.set(Flags::Z, v == 0);
    }
}

impl Cpu {
    fn resolve_addressing(&mut self, cycle: u8) -> (IoRequest, bool) {
        match self.addressing {
            Addressing::Implied | Addressing::Accumulator => self.resolve_implicit(),
            Addressing::Immediate => self.resolve_immediate(),
            Addressing::ZeroPage => self.resolve_zeropage(),
            Addressing::ZeroPageX => self.resolve_zeropage_indexed(cycle, self.x),
            Addressing::ZeroPageY => self.resolve_zeropage_indexed(cycle, self.y),
            Addressing::Absolute => self.resolve_absolute(cycle),
            Addressing::AbsoluteX => self.resolve_absolute_indexed(cycle, self.x),
            Addressing::AbsoluteY => self.resolve_absolute_indexed(cycle, self.y),
            Addressing::Relative => self.resolve_relative(cycle),
        }
    }

    fn resolve_implicit(&mut self) -> (IoRequest, bool) {
        (self.issue_read(*self.pc, Destination::Discard), true)
    }

    fn resolve_immediate(&mut self) -> (IoRequest, bool) {
        let res = self.issue_read(*self.pc, Destination::DataLatch);
        *self.pc = self.pc.wrapping_add(1);
        (res, true)
    }

    fn resolve_zeropage(&mut self) -> (IoRequest, bool) {
        let res = self.issue_read(self.pc.with_high(0x00), Destination::AddressLow);
        *self.pc = self.pc.wrapping_add(1);
        (res, true)
    }

    fn resolve_zeropage_indexed(&mut self, cycle: u8, idx: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(*self.pc, Destination::AddressLow);
                *self.pc = self.pc.wrapping_add(1);
                (res, false)
            }
            2 => {
                let res = self.issue_read(self.ad.with_high(0x00), Destination::Discard);
                self.ad.apply_index(idx); // Zeropage does not cause a page crossing.
                (res, true)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_absolute(&mut self, cycle: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(*self.pc, Destination::AddressLow);
                *self.pc = self.pc.wrapping_add(1);
                (res, false)
            }
            2 => {
                let res = self.issue_read(*self.pc, Destination::AddressHigh);
                *self.pc = self.pc.wrapping_add(1);
                (res, false)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_absolute_indexed(&mut self, cycle: u8, idx: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(*self.pc, Destination::AddressLow);
                *self.pc = self.pc.wrapping_add(1);
                (res, false)
            }
            2 => {
                let res = self.issue_read(*self.pc, Destination::AddressHigh);
                self.ad.apply_index(idx); // Address may be invalid at here.
                *self.pc = self.pc.wrapping_add(1);
                (res, !self.ad.is_crossed())
            }
            3 => {
                let res = self.issue_read(*self.ad, Destination::Discard); // wrong read
                self.ad.fix_page();
                (res, true)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_relative(&mut self, cycle: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(*self.pc, Destination::DataLatch);
                *self.pc = self.pc.wrapping_add(1);
                (res, false)
            }
            2 => {
                let branched = if let Operator::Branch(o) = self.operator {
                    match o {
                        Branch::BCC => !self.p.contains(Flags::C),
                        Branch::BCS => self.p.contains(Flags::C),
                        Branch::BNE => !self.p.contains(Flags::Z),
                        Branch::BEQ => self.p.contains(Flags::Z),
                        Branch::BVC => !self.p.contains(Flags::V),
                        Branch::BVS => self.p.contains(Flags::V),
                        Branch::BPL => !self.p.contains(Flags::N),
                        Branch::BMI => self.p.contains(Flags::N),
                    }
                } else {
                    unreachable!()
                };

                if branched {
                    let res = self.issue_read(*self.pc, Destination::Discard);
                    self.pc.apply_index_signed(self.dl as i8);
                    (res, false)
                } else {
                    (self.fetch_opcode(), true)
                }
            }
            3 => {
                if self.pc.is_crossed() {
                    let res = self.issue_read(*self.pc, Destination::Discard);
                    self.pc.fix_page();
                    (res, true)
                } else {
                    (self.fetch_opcode(), true)
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Cpu {
    fn exec_instruction(&mut self, cycle: u8) -> (IoRequest, bool) {
        let res = match self.operator {
            Operator::Implicit(o) => self.exec_implicit(o),
            Operator::UniqueCycle(o) => return self.exec_unique_cycle(o, cycle),
        };

        (res, true)
    }

    fn exec_implicit(&mut self, operator: Implicit) -> IoRequest {
        match operator {
            Implicit::JAM => self.is_jammed = true,
            Implicit::INX => {
                self.x = self.x.wrapping_add(1);
                self.update_nz(self.x);
            }
            Implicit::INY => {
                self.y = self.y.wrapping_add(1);
                self.update_nz(self.y);
            }
            Implicit::DEX => {
                self.x = self.x.wrapping_sub(1);
                self.update_nz(self.y);
            }
            Implicit::DEY => {
                self.y = self.y.wrapping_sub(1);
                self.update_nz(self.y);
            }
            Implicit::CLC => self.p.remove(Flags::C),
            Implicit::CLD => self.p.remove(Flags::D),
            Implicit::CLI => self.p.remove(Flags::I),
            Implicit::CLV => self.p.remove(Flags::V),
            Implicit::SEC => self.p.insert(Flags::C),
            Implicit::SED => self.p.insert(Flags::D),
            Implicit::SEI => self.p.insert(Flags::I),
            Implicit::TAX => {
                self.x = self.a;
                self.update_nz(self.x);
            }
            Implicit::TAY => {
                self.y = self.a;
                self.update_nz(self.y);
            }
            Implicit::TSX => {
                self.sp = self.x;
                self.update_nz(self.sp);
            }
            Implicit::TXA => {
                self.a = self.x;
                self.update_nz(self.a);
            }
            Implicit::TYA => {
                self.a = self.y;
                self.update_nz(self.a);
            }
            Implicit::TXS => self.sp = self.x,
        }

        self.issue_read(*self.pc, Destination::Discard)
    }

    fn exec_unique_cycle(&mut self, operator: UniqueCycle, cycle: u8) -> (IoRequest, bool) {}
}

// Addressingはアドレス計算のみを行う。
// 実際の値の読み込み、適用はRead/Modify/Writeだけで行える。
