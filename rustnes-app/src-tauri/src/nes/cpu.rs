mod instr;
mod opcode;
mod regs;

use crate::nes::{bus::CpuBus, NesBus};
use opcode::*;
use regs::{Register, StackPtr, Status, Word};

const RESET_VECTOR: u16 = 0xFFFC;
const BRK_VECTOR: u16 = 0xFFFE;

const DEFAULT_A: u8 = 0x00;
const DEFAULT_X: u8 = 0x00;
const DEFAULT_Y: u8 = 0x00;
const DEFAULT_P: u8 = 0x34;
const DEFAULT_SP: u8 = 0x00;
const DEFAULT_PC: u16 = 0x8000; // This value is a placeholder. PC is set by the RESET VECTOR

macro_rules! cpubus {
    ($nesbus:expr) => {
        CpuBus {
            openbus: &mut $nesbus.cpu_openbus,
            wram: &mut $nesbus.wram,
            ppu: &mut $nesbus.ppu,
            nesrom: &mut $nesbus.nesrom,
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemAddrKind {
    PageSafe,
    NotCrossed,
    RequireFix,
}

#[derive(Debug, Clone, Copy)]
pub struct Cpu {
    pub is_jammed: bool,
    pub reg: Register,
}

impl Cpu {
    #[allow(clippy::new_without_default)]
    // TODO: Enable to override initial registers(include PC), wram, and etc...
    pub fn new() -> Self {
        Self {
            is_jammed: false,
            reg: Register {
                a: DEFAULT_A,
                x: DEFAULT_X,
                y: DEFAULT_Y,
                p: Status::from(DEFAULT_P),
                sp: StackPtr::from(DEFAULT_SP),
                pc: Word::from(DEFAULT_PC),
            },
        }
    }

    pub fn reset(&mut self, bus: &mut NesBus) {
        self.is_jammed = false;
        self.read_at_pc(bus);
        self.read_at_pc(bus);
        for _ in 0..3 {
            self.read_at_sp(bus);
            self.reg.sp.inner = self.reg.sp.inner.wrapping_sub(1);
        }
        self.reg.pc.lo = self.read(bus, RESET_VECTOR);
        self.reg.p.set_interrupt(true);
        self.reg.pc.hi = self.read(bus, RESET_VECTOR.wrapping_add(1));
    }

    pub fn step(&mut self, bus: &mut NesBus) {
        // TODO: Handle interrupt here
        let opcode_byte = self.fetch(bus);
        let opcode = OPCODE_TABLE[opcode_byte as usize];
        print!("{:?} {:?} ", opcode.mnemonic, opcode.addressing);
        self.exec_opcode(bus, opcode.mnemonic, opcode.addressing);
    }

    fn read(&mut self, bus: &mut NesBus, addr: impl Into<u16>) -> u8 {
        // TODO: Handle dma like: fn process_dma(&mut self, last_addr: impl Into<u16>, page: u8)
        bus.sync_with_cpu();
        cpubus!(bus).read(addr.into())
    }

    fn write(&mut self, bus: &mut NesBus, addr: impl Into<u16>, data: u8) {
        // TODO: Hook dma here
        bus.sync_with_cpu();
        cpubus!(bus).write(addr.into(), data);
    }

    fn read_at_pc(&mut self, bus: &mut NesBus) -> u8 {
        self.read(bus, self.reg.pc)
    }

    fn read_at_sp(&mut self, bus: &mut NesBus) -> u8 {
        self.read(bus, self.reg.sp)
    }

    fn advance_pc(&mut self) {
        let (idxed_lo, crossed) = self.reg.pc.lo.overflowing_add(1);
        self.reg.pc.lo = idxed_lo;
        if crossed {
            self.reg.pc.hi = self.reg.pc.hi.wrapping_add(1);
        }
    }

    fn fetch(&mut self, bus: &mut NesBus) -> u8 {
        let byte = self.read_at_pc(bus);
        self.advance_pc();
        byte
    }

    fn push(&mut self, bus: &mut NesBus, data: u8) {
        self.write(bus, self.reg.sp, data);
        self.reg.sp.inner = self.reg.sp.inner.wrapping_sub(1);
    }

    fn pop(&mut self, bus: &mut NesBus) -> u8 {
        self.reg.sp.inner = self.reg.sp.inner.wrapping_add(1);
        self.read_at_sp(bus)
    }

    fn exec_opcode(&mut self, bus: &mut NesBus, mnem: Mnemonic, addr: Addressing) {
        match (mnem, addr) {
            (Mnemonic::Read(Read::NOP), Addressing::Implied) => (),
            (Mnemonic::Unique(unique), _) => self.exec_unique(bus, unique, addr),
            (Mnemonic::Branch(branch), Addressing::Relative) => self.exec_relative(bus, branch),
            (Mnemonic::Short(short), Addressing::Implied) => self.exec_short(bus, short),
            (Mnemonic::Modify(modify), Addressing::Accumulator) => self.exec_accum(bus, modify),
            (Mnemonic::Read(read), Addressing::Immediate) => self.exec_immediate(bus, read),
            (Mnemonic::Read(read), _) => self.exec_mem_read(bus, read, addr),
            (Mnemonic::Modify(modify), _) => self.exec_mem_modify(bus, modify, addr),
            (Mnemonic::Write(write), _) => self.exec_mem_write(bus, write, addr),
            _ => todo!(),
        }
    }

    fn exec_relative(&mut self, bus: &mut NesBus, mnem: Branch) {
        let offset = self.fetch(bus) as i8;
        let is_branched = self.operate_branch(mnem);

        if !is_branched {
            return;
        }

        self.read_at_pc(bus);
        let (idxed_lo, crossed) = self.reg.pc.lo.overflowing_add_signed(offset);
        self.reg.pc.lo = idxed_lo;

        if !crossed {
            // TODO: Clear polled irq interrupt to ignore
            return;
        }

        self.read_at_pc(bus);
        let hi_fixer = if offset > 0 { 1 } else { -1 };
        self.reg.pc.hi = self.reg.pc.hi.wrapping_add_signed(hi_fixer);
    }

    fn exec_short(&mut self, bus: &mut NesBus, mnem: Short) {
        self.read_at_pc(bus); // Operate after read to delay interrupt
        self.operate_short(mnem);
    }

    fn exec_accum(&mut self, bus: &mut NesBus, mnem: Modify) {
        self.read_at_pc(bus);
        let mut tmp_a = self.reg.a;
        self.operate_modify(mnem, &mut tmp_a);
        self.reg.a = tmp_a;
    }

    fn exec_immediate(&mut self, bus: &mut NesBus, mnem: Read) {
        let m = self.fetch(bus);
        self.operate_read(mnem, m);
    }

    fn exec_mem_read(&mut self, bus: &mut NesBus, mnem: Read, addr: Addressing) {
        let (unchecked_addr, kind) = self.resolve_mem_addr(bus, addr);
        let mem_addr = if kind == MemAddrKind::RequireFix {
            self.read(bus, unchecked_addr); // Dummy read to fix ADH
            Word::from_le_bytes([unchecked_addr.lo, unchecked_addr.hi.wrapping_add(1)])
        } else {
            unchecked_addr
        };

        let m = self.read(bus, mem_addr);
        self.operate_read(mnem, m);
    }

    fn exec_mem_modify(&mut self, bus: &mut NesBus, mnem: Modify, addr: Addressing) {
        let (unchecked_addr, kind) = self.resolve_mem_addr(bus, addr);
        let mem_addr = if kind == MemAddrKind::PageSafe {
            unchecked_addr
        } else {
            self.read(bus, unchecked_addr); // Dummy read to guarantee ADH
            if kind == MemAddrKind::RequireFix {
                Word::from_le_bytes([unchecked_addr.lo, unchecked_addr.hi.wrapping_add(1)])
            } else {
                unchecked_addr
            }
        };

        let mut m = self.read(bus, mem_addr);
        self.write(bus, mem_addr, m); // Dummy write
        self.operate_modify(mnem, &mut m); // Operate on the value
        self.write(bus, mem_addr, m); // Write the value back
    }

    fn exec_mem_write(&mut self, bus: &mut NesBus, mnem: Write, addr: Addressing) {
        let (unchecked_addr, kind) = self.resolve_mem_addr(bus, addr);
        let mut mem_addr = if kind == MemAddrKind::PageSafe {
            unchecked_addr
        } else {
            self.read(bus, unchecked_addr); // Dummy read to guarantee ADH
            if kind == MemAddrKind::RequireFix {
                Word::from_le_bytes([unchecked_addr.lo, unchecked_addr.hi.wrapping_add(1)])
            } else {
                unchecked_addr
            }
        };

        let m = self.operate_write(mnem, &mut mem_addr);
        self.write(bus, mem_addr, m); // Write the value
    }

    fn resolve_mem_addr(&mut self, bus: &mut NesBus, addr: Addressing) -> (Word, MemAddrKind) {
        match addr {
            Addressing::Zeropage => self.zeropage(bus),
            Addressing::ZeropageX => self.zeropage_indexed(bus, self.reg.x),
            Addressing::ZeropageY => self.zeropage_indexed(bus, self.reg.y),
            Addressing::Absolute => self.absolute(bus),
            Addressing::AbsoluteX => self.absolute_indexed(bus, self.reg.x),
            Addressing::AbsoluteY => self.absolute_indexed(bus, self.reg.y),
            Addressing::XIdxedInd => self.x_indexed_indirect(bus),
            Addressing::IndYIdxed => self.indirect_y_indexed(bus),
            _ => panic!(),
        }
    }

    fn zeropage(&mut self, bus: &mut NesBus) -> (Word, MemAddrKind) {
        (Word::from_zeropage(self.fetch(bus)), MemAddrKind::PageSafe)
    }

    fn zeropage_indexed(&mut self, bus: &mut NesBus, indexer: u8) -> (Word, MemAddrKind) {
        let lo = self.fetch(bus);
        self.read(bus, Word::from_zeropage(lo));
        let lo = lo.wrapping_add(indexer);
        (Word::from_zeropage(lo), MemAddrKind::PageSafe)
    }

    fn absolute(&mut self, bus: &mut NesBus) -> (Word, MemAddrKind) {
        let lo = self.fetch(bus);
        let hi = self.fetch(bus);
        (Word::from_le_bytes([lo, hi]), MemAddrKind::PageSafe)
    }

    fn absolute_indexed(&mut self, bus: &mut NesBus, indexer: u8) -> (Word, MemAddrKind) {
        let lo = self.fetch(bus);
        let hi = self.fetch(bus);
        let (lo, is_crossed) = lo.overflowing_add(indexer);
        let addr = Word::from_le_bytes([lo, hi]);

        if is_crossed {
            (addr, MemAddrKind::RequireFix)
        } else {
            (addr, MemAddrKind::NotCrossed)
        }
    }

    fn x_indexed_indirect(&mut self, bus: &mut NesBus) -> (Word, MemAddrKind) {
        let ptr = self.fetch(bus);
        self.read(bus, Word::from_zeropage(ptr));
        let ptr = ptr.wrapping_add(self.reg.x);
        let lo = self.read(bus, Word::from_zeropage(ptr));
        let hi = self.read(bus, Word::from_zeropage(ptr.wrapping_add(1)));
        (Word::from_le_bytes([lo, hi]), MemAddrKind::PageSafe)
    }

    fn indirect_y_indexed(&mut self, bus: &mut NesBus) -> (Word, MemAddrKind) {
        let ptr = self.fetch(bus);
        let lo = self.read(bus, Word::from_zeropage(ptr));
        let hi = self.read(bus, Word::from_zeropage(ptr.wrapping_add(1)));
        let (lo, is_crossed) = lo.overflowing_add(self.reg.y);
        let addr = Word::from_le_bytes([lo, hi]);

        if is_crossed {
            (addr, MemAddrKind::RequireFix)
        } else {
            (addr, MemAddrKind::NotCrossed)
        }
    }
}

impl Cpu {
    fn update_nz(&mut self, v: u8) {
        self.reg.p.set_negative(v & 0x80 != 0);
        self.reg.p.set_zero(v == 0);
    }

    fn add_with_carry(&mut self, v: u8) {
        let old_a = self.reg.a;
        let res_u16 = old_a as u16 + v as u16 + self.reg.p.carry() as u16;
        let res = res_u16 as u8;

        let carry = 0xFF < res_u16;
        let overflow = (old_a ^ res) & (v ^ res) & 0x80 != 0;
        self.reg.p.set_carry(carry);
        self.reg.p.set_overflow(overflow);
        self.update_nz(res);
        self.reg.a = res;
    }

    fn compare(&mut self, v: u8, w: u8) {
        let (res, borrow) = v.overflowing_sub(w);
        self.reg.p.set_carry(!borrow);
        self.update_nz(res);
    }

    fn exec_unique(&mut self, bus: &mut NesBus, mnem: Unique, addr: Addressing) {
        match (mnem, addr) {
            (Unique::JAM, Addressing::Undefined) => {
                // TODO: destroyed read
                self.read_at_pc(bus);
                self.is_jammed = true;
            }

            // TODO: Interrupt vector hijack
            (Unique::BRK, Addressing::Implied) => {
                self.fetch(bus);
                self.push(bus, self.reg.pc.hi);
                self.push(bus, self.reg.pc.lo);
                self.push(bus, self.reg.p.as_byte(true));
                self.reg.pc.lo = self.read(bus, BRK_VECTOR);
                self.reg.p.set_interrupt(true);
                self.reg.pc.hi = self.read(bus, BRK_VECTOR.wrapping_add(1));
            }

            (Unique::RTI, Addressing::Implied) => {
                self.read_at_pc(bus);
                self.read_at_sp(bus);
                self.reg.p = Status::from(self.pop(bus));
                self.reg.pc.lo = self.pop(bus);
                self.reg.pc.hi = self.pop(bus);
            }

            (Unique::RTS, Addressing::Implied) => {
                self.read_at_pc(bus);
                self.read_at_sp(bus);
                self.reg.pc.lo = self.pop(bus);
                self.reg.pc.hi = self.pop(bus);
                self.advance_pc();
            }

            (Unique::PHA, Addressing::Implied) => {
                self.read_at_pc(bus);
                self.push(bus, self.reg.a);
            }

            (Unique::PHP, Addressing::Implied) => {
                self.read_at_pc(bus);
                self.push(bus, self.reg.p.as_byte(false));
            }

            (Unique::PLA, Addressing::Implied) => {
                self.read_at_pc(bus);
                self.read_at_sp(bus);
                self.reg.a = self.pop(bus);
                self.update_nz(self.reg.a);
            }

            (Unique::PLP, Addressing::Implied) => {
                self.read_at_pc(bus);
                self.read_at_sp(bus);
                self.reg.p = Status::from(self.pop(bus));
            }

            (Unique::JSR, Addressing::Absolute) => {
                let lo = self.fetch(bus);
                self.read_at_sp(bus);
                self.push(bus, self.reg.pc.hi);
                self.push(bus, self.reg.pc.lo);
                self.reg.pc.hi = self.fetch(bus);
                self.reg.pc.lo = lo;
            }

            (Unique::JMP, Addressing::Absolute) => {
                let lo = self.fetch(bus);
                self.reg.pc.hi = self.fetch(bus);
                self.reg.pc.lo = lo;
            }

            (Unique::JMP, Addressing::AbsoluteInd) => {
                let mut ptr = Word::from(0);
                ptr.lo = self.fetch(bus);
                ptr.hi = self.fetch(bus);
                self.reg.pc.lo = self.read(bus, ptr);
                ptr.lo = ptr.lo.wrapping_add(1);
                self.reg.pc.hi = self.read(bus, ptr);
            }
            _ => panic!(),
        }
    }

    fn operate_branch(&mut self, mnem: Branch) -> bool {
        match mnem {
            Branch::BCS => self.reg.p.carry(),
            Branch::BCC => !self.reg.p.carry(),
            Branch::BEQ => self.reg.p.zero(),
            Branch::BNE => !self.reg.p.zero(),
            Branch::BVS => self.reg.p.overflow(),
            Branch::BVC => !self.reg.p.overflow(),
            Branch::BMI => self.reg.p.negative(),
            Branch::BPL => !self.reg.p.negative(),
        }
    }

    fn operate_short(&mut self, mnem: Short) {
        match mnem {
            Short::INX => {
                self.reg.x = self.reg.x.wrapping_add(1);
                self.update_nz(self.reg.x);
            }

            Short::INY => {
                self.reg.y = self.reg.y.wrapping_add(1);
                self.update_nz(self.reg.y);
            }

            Short::DEX => {
                self.reg.x = self.reg.x.wrapping_sub(1);
                self.update_nz(self.reg.x);
            }

            Short::DEY => {
                self.reg.y = self.reg.y.wrapping_sub(1);
                self.update_nz(self.reg.y);
            }

            Short::SEC => self.reg.p.set_carry(true),
            Short::SED => self.reg.p.set_decimal(true),
            Short::SEI => self.reg.p.set_interrupt(true),

            Short::CLC => self.reg.p.set_carry(false),
            Short::CLD => self.reg.p.set_decimal(false),
            Short::CLI => self.reg.p.set_interrupt(false),
            Short::CLV => self.reg.p.set_overflow(false),

            Short::TAX => {
                self.reg.x = self.reg.a;
                self.update_nz(self.reg.x);
            }

            Short::TAY => {
                self.reg.y = self.reg.a;
                self.update_nz(self.reg.y);
            }

            Short::TXA => {
                self.reg.a = self.reg.x;
                self.update_nz(self.reg.a);
            }

            Short::TYA => {
                self.reg.a = self.reg.y;
                self.update_nz(self.reg.a);
            }

            Short::TSX => {
                self.reg.x = self.reg.sp.into();
                self.update_nz(self.reg.x);
            }

            Short::TXS => self.reg.sp = self.reg.x.into(),
        };
    }

    fn operate_read(&mut self, mnem: Read, m: u8) {
        match mnem {
            Read::NOP => (),

            Read::ADC => self.add_with_carry(m),
            Read::SBC => self.add_with_carry(!m),

            Read::AND => {
                self.reg.a &= m;
                self.update_nz(self.reg.a);
            }

            Read::ORA => {
                self.reg.a |= m;
                self.update_nz(self.reg.a);
            }

            Read::EOR => {
                self.reg.a ^= m;
                self.update_nz(self.reg.a);
            }

            Read::BIT => {
                self.reg.p.set_negative(m & 0x80 != 0);
                self.reg.p.set_overflow(m & 0x40 != 0);
                self.reg.p.set_zero(self.reg.a & m == 0);
            }

            Read::CMP => self.compare(self.reg.a, m),
            Read::CPX => self.compare(self.reg.x, m),
            Read::CPY => self.compare(self.reg.y, m),

            Read::LDA => {
                self.reg.a = m;
                self.update_nz(self.reg.a);
            }

            Read::LDX => {
                self.reg.x = m;
                self.update_nz(self.reg.x);
            }

            Read::LDY => {
                self.reg.y = m;
                self.update_nz(self.reg.y);
            }

            Read::LAX => (),
            Read::LXA => (),
            Read::LAS => (),
            Read::ALR => (),
            Read::ARR => (),
            Read::ANC => (),
            Read::AXS => (),
            Read::ANE => (),
        }
    }

    fn operate_modify(&mut self, mnem: Modify, m: &mut u8) {
        match mnem {
            Modify::ASL => {
                self.reg.p.set_carry(*m & 0x80 != 0);
                *m <<= 1;
                self.update_nz(*m);
            }

            Modify::LSR => {
                self.reg.p.set_carry(*m & 0x01 != 0);
                *m >>= 1;
                self.update_nz(*m);
            }

            Modify::ROL => {
                let old_c = self.reg.p.carry();
                self.reg.p.set_carry(*m & 0x80 != 0);
                *m = (*m << 1) | if old_c { 0x01 } else { 0 };
                self.update_nz(*m);
            }

            Modify::ROR => {
                let old_c = self.reg.p.carry();
                self.reg.p.set_carry(*m & 0x01 != 0);
                *m = (*m >> 1) | if old_c { 0x80 } else { 0 };
                self.update_nz(*m);
            }

            Modify::INC => {
                *m = m.wrapping_add(1);
                self.update_nz(*m);
            }

            Modify::DEC => {
                *m = m.wrapping_sub(1);
                self.update_nz(*m);
            }

            Modify::DCP => (),
            Modify::ISB => (),
            Modify::RRA => (),
            Modify::RLA => (),
            Modify::SLO => (),
            Modify::SRE => (),
        }
    }

    fn operate_write(&mut self, mnem: Write, _addr: &mut Word) -> u8 {
        match mnem {
            Write::STA => self.reg.a,
            Write::STX => self.reg.x,
            Write::STY => self.reg.y,

            Write::SAX => 0x00,
            Write::SBX => 0x00,
            Write::SHS => 0x00,
            Write::SHA => 0x00,
            Write::SHX => 0x00,
            Write::SHY => 0x00,
        }
    }
}
