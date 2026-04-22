mod address;
mod opcode;
mod statusreg;

use self::{
    address::{Address, ZeropageAddress},
    opcode::{Addressing, Branch, Implicit, Operator, ReadOp, UniqueCycle, OPCODE_TABLE},
    statusreg::{Flags, StatusRegister},
};

#[allow(clippy::upper_case_acronyms)]
enum Destination {
    Discard,
    OpcodeLatch,
    DataLatch,
    AddressLow,
    AddressHigh,
    ZeropagePointer,
    PCLow,
    PCHigh,
    ApplyReadOp(ReadOp),
}

enum Phase {
    FetchOpcode,
    ResolveAddressing(u8),
    ExcuteInstruction(u8),
    HandleInterrupt(u8),
}

pub enum IoRequest {
    Read(u16),
    Write(u16, u8),
}

pub struct Cpu {
    is_jammed: bool,
    phase: Phase,
    operator: Operator,
    addressing: Addressing,

    pc: Address,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    p: StatusRegister,

    dst: Destination,
    ad: Address,
    zpg_ptr: u8,
    dl: u8,
    is_branched: bool,
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
            Destination::DataLatch => self.dl = v,
            Destination::AddressLow => self.ad.set_lo(v),
            Destination::AddressHigh => self.ad.set_hi(v),
            Destination::ZeropagePointer => self.zpg_ptr = v,
            Destination::PCLow => self.pc.set_lo(v),
            Destination::PCHigh => self.pc.set_hi(v),
            Destination::ApplyReadOp(o) => self.apply_readop(o),
        };
    }

    fn fetch_opcode(&mut self) -> IoRequest {
        self.phase = Phase::ResolveAddressing(1);
        let res = self.issue_read(self.pc, Destination::OpcodeLatch);
        self.pc.increment();
        res
    }

    fn issue_read(&mut self, addr: Address, dst: Destination) -> IoRequest {
        self.dst = dst;
        IoRequest::Read(addr.as_u16())
    }

    fn issue_write(&self, addr: Address, data: u8) -> IoRequest {
        IoRequest::Write(addr.as_u16(), data)
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
            Addressing::XIdxedInd => self.resolve_x_indexed_indirect(cycle),
        }
    }

    fn resolve_implicit(&mut self) -> (IoRequest, bool) {
        (self.issue_read(self.pc, Destination::Discard), true)
    }

    fn resolve_immediate(&mut self) -> (IoRequest, bool) {
        let res = self.issue_read(self.pc, Destination::DataLatch);
        self.pc.increment();
        (res, true)
    }

    fn resolve_zeropage(&mut self) -> (IoRequest, bool) {
        let res = self.issue_read(self.pc.as_zeropage(), Destination::AddressLow);
        self.pc.increment();
        (res, true)
    }

    fn resolve_zeropage_indexed(&mut self, cycle: u8, idx: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(self.pc, Destination::AddressLow);
                self.pc.increment();
                (res, false)
            }
            2 => {
                let res = self.issue_read(self.ad.as_zeropage(), Destination::Discard);
                self.ad.apply_index(idx); // Zeropage does not cause a page crossing.
                (res, true)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_absolute(&mut self, cycle: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(self.pc, Destination::AddressLow);
                self.pc.increment();
                (res, false)
            }
            2 => {
                let res = self.issue_read(self.pc, Destination::AddressHigh);
                self.pc.increment();
                (res, true)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_absolute_indexed(&mut self, cycle: u8, idx: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(self.pc, Destination::AddressLow);
                self.pc.increment();
                (res, false)
            }
            2 => {
                let res = self.issue_read(self.pc, Destination::AddressHigh);
                self.ad.apply_index(idx); // Address may be invalid at here.
                self.pc.increment();
                (res, true)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_relative(&mut self, cycle: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(self.pc, Destination::DataLatch);
                self.pc.increment();
                (res, true)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_x_indexed_indirect(&mut self, cycle: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(self.pc, Destination::ZeropagePointer);
                self.pc.increment();
                (res, false)
            }

            2 => {
                let res = self.issue_read(self.zpg_ptr.as_address(), Destination::Discard);
                self.zpg_ptr = self.zpg_ptr.wrapping_add(self.x);
                (res, false)
            }

            3 => (
                self.issue_read(self.zpg_ptr.as_address(), Destination::AddressLow),
                false,
            ),

            4 => {
                let res = self.issue_read(
                    self.zpg_ptr.wrapping_add(1).as_address(),
                    Destination::AddressHigh,
                );
                (res, true)
            }

            _ => unreachable!(),
        }
    }

    fn resolve_indirect_y_indexed(&mut self, cycle: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                let res = self.issue_read(self.pc, Destination::ZeropagePointer);
                self.pc.increment();
                (res, false)
            }

            2 => {
                let res = self.issue_read(self.zpg_ptr.as_address(), Destination::AddressLow);
                (res, false)
            }

            3 => {
                let res = self.issue_read(
                    self.zpg_ptr.wrapping_add(1).as_address(),
                    Destination::AddressHigh,
                );
                self.ad.apply_index(self.y);
                (res, true)
            }

            _ => unreachable!(),
        }
    }
}

impl Cpu {
    fn exec_instruction(&mut self, cycle: u8) -> (IoRequest, bool) {
        match self.operator {
            Operator::Implicit(o) => self.exec_implicit(o, cycle),
            Operator::Branch(o) => self.exec_branch(o, cycle),
            Operator::UniqueCycle(o) => return self.exec_unique_cycle(o, cycle),
        }
    }

    fn exec_implicit(&mut self, operator: Implicit, _cycle: u8) -> (IoRequest, bool) {
        self.apply_implicit(operator);
        (self.issue_read(self.pc, Destination::Discard), true)
    }

    fn exec_branch(&mut self, operator: Branch, cycle: u8) -> (IoRequest, bool) {
        match cycle {
            1 => {
                self.apply_branch(operator);

                if self.is_branched {
                    let res = self.issue_read(self.pc, Destination::Discard);
                    self.pc.apply_index_signed(self.dl as i8);
                    (res, false)
                } else {
                    (self.fetch_opcode(), true)
                }
            }
            2 => {
                if self.pc.is_crossed() {
                    let res = self.issue_read(self.pc, Destination::Discard);
                    self.pc.fix_page();
                    (res, true)
                } else {
                    (self.fetch_opcode(), true)
                }
            }
            _ => unreachable!(),
        }
    }

    fn exec_readop(&mut self, operator: ReadOp, cycle: u8) -> (IoRequest, bool) {}

    fn exec_unique_cycle(&mut self, operator: UniqueCycle, cycle: u8) -> (IoRequest, bool) {}
}

impl Cpu {
    fn apply_implicit(&mut self, operator: Implicit) {
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
    }

    fn apply_branch(&mut self, operator: Branch) {
        self.is_branched = match operator {
            Branch::BCC => !self.p.contains(Flags::C),
            Branch::BCS => self.p.contains(Flags::C),
            Branch::BNE => !self.p.contains(Flags::Z),
            Branch::BEQ => self.p.contains(Flags::Z),
            Branch::BVC => !self.p.contains(Flags::V),
            Branch::BVS => self.p.contains(Flags::V),
            Branch::BPL => !self.p.contains(Flags::N),
            Branch::BMI => self.p.contains(Flags::N),
        };
    }

    fn apply_readop(&mut self, operator: ReadOp) {}
}

// fetch フェッチ -> 共通固定サイクル
// resolve アドレッシング -> 独自固定サイクル
// exec 命令実行 -> 可変サイクル
// apply 命令適用 -> 共通固定サイクル
