mod seq;

use crate::nes::bus::CpuBus;
use bitflags::bitflags;
use seq::Cycle;

const ZEROPAGE: u8 = 0x00;
const STACKPAGE: u8 = 0x01;

const RESET_VECTOR: u16 = 0xFFFC;
const NMI_VECTOR: u16 = 0xFFFA;
const BRK_VECTOR: u16 = 0xFFFE;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Status: u8 {
        const C = 0b0000_0001;
        const Z = 0b0000_0010;
        const I = 0b0000_0100;
        const D = 0b0000_1000;
        const V = 0b0100_0000;
        const N = 0b1000_0000;
    }
}

impl Status {
    const B: u8 = 0b0001_0000;
    const R: u8 = 0b0010_0000;

    pub const fn new(v: u8) -> Self {
        Self::from_bits_truncate(v)
    }

    pub fn as_byte(&self, b_flag: bool) -> u8 {
        let with_r = self.bits() | Self::R;
        if b_flag {
            with_r | Self::B
        } else {
            with_r
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackPtr {
    inner: u8,
}

impl StackPtr {
    pub const fn new(inner: u8) -> Self {
        Self { inner }
    }

    pub fn increment(&mut self) {
        self.inner = self.inner.wrapping_add(1);
    }

    pub fn decrement(&mut self) {
        self.inner = self.inner.wrapping_sub(1);
    }
}

impl From<u8> for StackPtr {
    fn from(val: u8) -> Self {
        StackPtr::new(val)
    }
}

impl From<StackPtr> for u8 {
    fn from(value: StackPtr) -> Self {
        value.inner
    }
}

impl From<StackPtr> for Word {
    fn from(value: StackPtr) -> Self {
        Word {
            lo: value.into(),
            hi: STACKPAGE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Word {
    pub lo: u8,
    pub hi: u8,
}

impl Word {
    pub const fn new(addr: u16) -> Self {
        let [lo, hi] = addr.to_le_bytes();
        Self { lo, hi }
    }

    pub fn carrying_inc(&mut self) {
        let (new_lo, carry) = self.lo.overflowing_add(1);
        self.lo = new_lo;
        if carry {
            self.hi = self.hi.wrapping_add(1);
        }
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Word::new(value)
    }
}

impl From<Word> for u16 {
    fn from(value: Word) -> Self {
        u16::from_le_bytes([value.lo, value.hi])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Reg {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: Status,
    pub sp: StackPtr,
    pub pc: Word,
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Calculation(fn(&mut InstrLogic)),
    Relative { flag: Status, condition: bool },
    SeqDependent,
}

#[derive(Debug, Clone, Copy)]
pub struct InstrLogic {
    reg: Reg,
    timing: u8,
    seq: &'static [Cycle],
    is_read: bool,
    addr: Word,
    addr_bus: Word,
    m: u8,
}

impl InstrLogic {
    fn tick(&mut self) {}

    fn read(&mut self, bus: &mut CpuBus) -> u8 {
        self.is_read = true;
        bus.read(self.addr_bus.into())
    }

    fn write(&mut self, bus: &mut CpuBus, data: u8) {
        self.is_read = false;
        bus.write(self.addr_bus.into(), data);
    }

    fn exec_op(&mut self, op: Operation) {
        match op {
            Operation::Calculation(calc) => calc(self),
            Operation::Relative { flag, condition } => {
                if self.reg.p.contains(flag) == condition {
                    self.timing += 2;
                }
            }
            Operation::SeqDependent => (),
        }
    }
}

impl InstrLogic {
    fn exec_cycle(&mut self, bus: &mut CpuBus, cycle: Cycle) {
        match cycle {
            Cycle::ReadAtPc => {
                self.addr_bus = self.reg.pc;
                self.read(bus);
            }

            Cycle::ReadAtPcInc => {
                self.addr_bus = self.reg.pc;
                self.reg.pc.carrying_inc();
                self.read(bus);
            }

            Cycle::ReadAtSp => {
                self.addr_bus = self.reg.sp.into();
                self.read(bus);
            }

            Cycle::ReadAtSpInc => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.increment();
                self.read(bus);
            }

            Cycle::ReadAtSpDec => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.decrement();
                self.read(bus);
            }

            Cycle::PushPchDec => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.decrement();
                self.write(bus, self.reg.pc.hi);
            }

            Cycle::PushPclDec => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.decrement();
                self.write(bus, self.reg.pc.lo);
            }

            Cycle::PushPSetBDec => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.decrement();
                self.write(bus, self.reg.p.as_byte(true));
            }

            Cycle::PushPDec => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.decrement();
                self.write(bus, self.reg.p.as_byte(false));
            }

            Cycle::PushADec => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.decrement();
                self.write(bus, self.reg.a);
            }

            Cycle::PullPInc => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.increment();
                self.reg.p = Status::new(self.read(bus));
            }

            Cycle::PullP => {
                self.addr_bus = self.reg.sp.into();
                self.reg.p = Status::new(self.read(bus));
            }

            Cycle::PullA => {
                self.addr_bus = self.reg.sp.into();
                self.reg.a = self.read(bus);
            }

            Cycle::PullPclInc => {
                self.addr_bus = self.reg.sp.into();
                self.reg.sp.increment();
                self.reg.pc.lo = self.read(bus);
            }

            Cycle::PullPch => {
                self.addr_bus = self.reg.sp.into();
                self.reg.pc.hi = self.read(bus);
            }

            Cycle::FetchAdlIncPc => {
                self.addr_bus = self.reg.pc;
                self.reg.pc.carrying_inc();
                self.addr.lo = self.read(bus);
            }

            Cycle::FetchAdhApplyPc => {
                self.addr_bus = self.reg.pc;
                self.addr.hi = self.read(bus);
                self.reg.pc = self.addr;
            }

            Cycle::FetchResVcl => {
                self.addr_bus = RESET_VECTOR.into();
                self.reg.pc.lo = self.read(bus);
            }

            Cycle::FetchResVch => {
                self.addr_bus = RESET_VECTOR.wrapping_add(1).into();
                self.reg.pc.hi = self.read(bus);
            }

            Cycle::FetchBrkVcl => {
                self.addr_bus = BRK_VECTOR.into();
                self.reg.pc.lo = self.read(bus);
            }

            Cycle::FetchBrkVch => {
                self.addr_bus = BRK_VECTOR.wrapping_add(1).into();
                self.reg.pc.hi = self.read(bus);
            }
        }
    }
}

impl InstrLogic {
    fn update_nz(&mut self, v: u8) {
        self.reg.p.set(Status::N, v & 0x80 != 0);
        self.reg.p.set(Status::Z, v == 0);
    }

    fn _adc(&mut self, v: u8) {
        let old_a = self.reg.a;
        let res_u16 = old_a as u16 + v as u16 + self.reg.p.contains(Status::C) as u16;
        let res = res_u16 as u8;

        let carry = 0xFF < res_u16;
        let overflow = (old_a ^ res) & (v ^ res) & 0x80 != 0;
        self.reg.p.set(Status::C, carry);
        self.reg.p.set(Status::V, overflow);
        self.update_nz(res);
        self.reg.a = res;
    }

    fn _cmp(&mut self, a: u8, b: u8) {
        let (res, borrow) = a.overflowing_sub(b);
        self.reg.p.set(Status::C, !borrow);
        self.update_nz(res);
    }

    fn sec(&mut self) {
        self.reg.p.insert(Status::C);
    }

    fn sed(&mut self) {
        self.reg.p.insert(Status::D);
    }

    fn sei(&mut self) {
        self.reg.p.insert(Status::I);
    }

    fn clc(&mut self) {
        self.reg.p.remove(Status::C);
    }

    fn cld(&mut self) {
        self.reg.p.remove(Status::D);
    }

    fn cli(&mut self) {
        self.reg.p.remove(Status::I);
    }

    fn clv(&mut self) {
        self.reg.p.remove(Status::V);
    }

    fn inx(&mut self) {
        self.reg.x = self.reg.x.wrapping_add(1);
        self.update_nz(self.reg.x);
    }

    fn iny(&mut self) {
        self.reg.y = self.reg.y.wrapping_add(1);
        self.update_nz(self.reg.y);
    }

    fn dex(&mut self) {
        self.reg.x = self.reg.x.wrapping_sub(1);
        self.update_nz(self.reg.x);
    }

    fn dey(&mut self) {
        self.reg.y = self.reg.y.wrapping_sub(1);
        self.update_nz(self.reg.y);
    }

    fn tax(&mut self) {
        self.reg.x = self.reg.a;
        self.update_nz(self.reg.x);
    }

    fn tay(&mut self) {
        self.reg.y = self.reg.a;
        self.update_nz(self.reg.y);
    }

    fn txa(&mut self) {
        self.reg.a = self.reg.x;
        self.update_nz(self.reg.a);
    }

    fn tya(&mut self) {
        self.reg.a = self.reg.y;
        self.update_nz(self.reg.a);
    }

    fn tsx(&mut self) {
        self.reg.x = self.reg.sp.into();
        self.update_nz(self.reg.x);
    }

    fn txs(&mut self) {
        self.reg.sp = self.reg.x.into();
    }

    fn nop(&mut self) {}

    fn adc(&mut self) {
        self._adc(self.m);
    }

    fn sbc(&mut self) {
        self._adc(!self.m);
    }

    fn and(&mut self) {
        self.reg.a &= self.m;
        self.update_nz(self.reg.a);
    }

    fn ora(&mut self) {
        self.reg.a |= self.m;
        self.update_nz(self.reg.a);
    }

    fn eor(&mut self) {
        self.reg.a ^= self.m;
        self.update_nz(self.reg.a);
    }

    fn lda(&mut self) {
        self.reg.a = self.m;
        self.update_nz(self.reg.a);
    }

    fn ldx(&mut self) {
        self.reg.x = self.m;
        self.update_nz(self.reg.x);
    }

    fn ldy(&mut self) {
        self.reg.y = self.m;
        self.update_nz(self.reg.y);
    }

    fn cmp(&mut self) {
        self._cmp(self.reg.a, self.m);
    }

    fn cpx(&mut self) {
        self._cmp(self.reg.x, self.m);
    }

    fn cpy(&mut self) {
        self._cmp(self.reg.y, self.m);
    }

    fn bit(&mut self) {
        self.reg.p.set(Status::N, self.m & 0x80 != 0);
        self.reg.p.set(Status::V, self.m & 0x40 != 0);
        self.reg.p.set(Status::Z, self.reg.a & self.m == 0);
    }

    fn lax(&mut self) {
        self.lda();
        self.ldx();
    }

    fn las(&mut self) {
        let res = u8::from(self.reg.sp) & self.m;
        self.reg.a = res;
        self.reg.x = res;
        self.reg.sp = res.into();
        self.update_nz(res);
    }

    fn alr(&mut self) {
        self.and();
        self.m = self.reg.a;
        self.lsr();
        self.reg.a = self.m;
    }

    fn arr(&mut self) {
        self.and();
        self.m = self.reg.a;
        self.ror();
        self.reg.a = self.m;

        let spec_bit = (self.m ^ (self.m << 1)) & 0x40;
        self.reg.p.set(Status::V, spec_bit != 0);
        self.reg.p.set(Status::C, self.m & 0x40 != 0);
    }

    fn anc(&mut self) {
        self.and();
        self.reg.p.set(Status::C, self.reg.a & 0x80 != 0);
    }

    fn axs(&mut self) {
        let ax = self.reg.a & self.reg.x;
        let (res, borrow) = ax.overflowing_sub(self.m);
        self.reg.x = res;
        self.reg.p.set(Status::C, !borrow);
        self.update_nz(res);
    }

    fn inc(&mut self) {
        self.m = self.m.wrapping_add(1);
        self.update_nz(self.m);
    }

    fn dec(&mut self) {
        self.m = self.m.wrapping_sub(1);
        self.update_nz(self.m);
    }

    fn asl(&mut self) {
        self.reg.p.set(Status::C, self.m & 0x80 != 0);
        self.m <<= 1;
        self.update_nz(self.m);
    }

    fn lsr(&mut self) {
        self.reg.p.set(Status::C, self.m & 0x01 != 0);
        self.m >>= 1;
        self.update_nz(self.m);
    }

    fn rol(&mut self) {
        let old_c = self.reg.p.contains(Status::C);
        self.reg.p.set(Status::C, self.m & 0x80 != 0);
        self.m = (self.m << 1) | if old_c { 0x01 } else { 0 };
        self.update_nz(self.m);
    }

    fn ror(&mut self) {
        let old_c = self.reg.p.contains(Status::C);
        self.reg.p.set(Status::C, self.m & 0x01 != 0);
        self.m = (self.m >> 1) | if old_c { 0x80 } else { 0 };
        self.update_nz(self.m);
    }

    fn dcp(&mut self) {
        self.dec();
        self.cmp();
    }

    fn isb(&mut self) {
        self.inc();
        self.sbc();
    }

    fn rla(&mut self) {
        self.rol();
        self.and();
    }

    fn rra(&mut self) {
        self.ror();
        self.adc();
    }

    fn slo(&mut self) {
        self.asl();
        self.ora();
    }

    fn sre(&mut self) {
        self.lsr();
        self.eor();
    }

    fn sta(&mut self) {
        self.m = self.reg.a;
    }

    fn stx(&mut self) {
        self.m = self.reg.x;
    }

    fn sty(&mut self) {
        self.m = self.reg.y;
    }

    fn sax(&mut self) {
        self.m = self.reg.a & self.reg.x;
    }
}
