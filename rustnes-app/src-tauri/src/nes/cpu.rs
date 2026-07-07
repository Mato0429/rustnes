mod opcode;

use crate::nes::{bus::CpuBus, NesBus};
use modular_bitfield::prelude::*;
use opcode::Relative;

const ZEROPAGE: u8 = 0x00;
const STACKPAGE: u8 = 0x01;

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

#[derive(Debug, Clone, Copy)]
pub struct Word {
    lo: u8,
    hi: u8,
}

impl Word {
    fn from_le_bytes(bytes: [u8; 2]) -> Self {
        Self::from(u16::from_le_bytes(bytes))
    }

    fn carrying_inc(&mut self) {
        let (idxed_lo, crossed) = self.lo.overflowing_add(1);
        self.lo = idxed_lo;
        if crossed {
            self.hi = self.hi.wrapping_add(1);
        }
    }
}

impl From<Word> for u16 {
    fn from(value: Word) -> Self {
        Self::from_le_bytes([value.lo, value.hi])
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        let [lo, hi] = value.to_le_bytes();
        Self { lo, hi }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackPtr {
    inner: u8,
}

impl StackPtr {
    fn inc(&mut self) {
        self.inner = self.inner.wrapping_add(1);
    }

    fn dec(&mut self) {
        self.inner = self.inner.wrapping_sub(1);
    }
}

impl From<StackPtr> for u8 {
    fn from(value: StackPtr) -> Self {
        value.inner
    }
}

impl From<u8> for StackPtr {
    fn from(value: u8) -> Self {
        Self { inner: value }
    }
}

impl From<StackPtr> for u16 {
    fn from(value: StackPtr) -> Self {
        Self::from_le_bytes([value.into(), STACKPAGE])
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct Status {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    #[skip] // Break (bit4) depends
    __: B2, // Reserved(bit5) is always 1
    pub overflow: bool,
    pub negative: bool,
}

impl Status {
    const B: u8 = 0b0001_0000;
    const R: u8 = 0b0010_0000;

    fn as_byte(&self, b_flag: bool) -> u8 {
        let with_r = self.into_bytes()[0] | Self::R;
        if b_flag {
            with_r | Self::B
        } else {
            with_r
        }
    }
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Status::from_bytes([value])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    a: u8,
    x: u8,
    y: u8,
    p: Status,
    sp: StackPtr,
    pc: Word,
}

#[derive(Debug, Clone, Copy)]
pub struct Cpu {
    reg: Register,
}

impl Cpu {
    #[allow(clippy::new_without_default)]
    // TODO: Enable to override initial registers(include PC), wram, and etc...
    pub fn new() -> Self {
        Self {
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
        self.read(bus, self.reg.pc);
        for _ in 0..3 {
            self.read(bus, self.reg.sp);
            self.reg.sp.dec();
        }
        self.reg.pc.lo = self.read(bus, RESET_VECTOR);
        self.reg.p.set_interrupt(true);
        self.reg.pc.hi = self.read(bus, RESET_VECTOR.wrapping_add(1));
    }

    pub fn step(&mut self, bus: &mut NesBus) {
        // TODO: Handle interrupt here
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

    fn fetch(&mut self, bus: &mut NesBus) -> u8 {
        let res = self.read(bus, self.reg.pc);
        self.reg.pc.carrying_inc();
        res
    }

    fn push(&mut self, bus: &mut NesBus, data: u8) {
        self.write(bus, self.reg.sp, data);
        self.reg.sp.dec();
    }

    fn pop(&mut self, bus: &mut NesBus) -> u8 {
        self.reg.sp.inc();
        self.read(bus, self.reg.sp)
    }

    fn zeropage(&mut self, bus: &mut NesBus) -> (Word, bool) {
        (Word::from_le_bytes([self.fetch(bus), ZEROPAGE]), false)
    }

    fn zeropage_indexed(&mut self, bus: &mut NesBus, indexer: u8) -> (Word, bool) {
        let lo = self.fetch(bus);
        self.read(bus, Word::from_le_bytes([lo, ZEROPAGE]));
        let lo = lo.wrapping_add(indexer);
        (Word::from_le_bytes([lo, ZEROPAGE]), false)
    }

    fn zeropage_x(&mut self, bus: &mut NesBus) -> (Word, bool) {
        self.zeropage_indexed(bus, self.reg.x)
    }

    fn zeropage_y(&mut self, bus: &mut NesBus) -> (Word, bool) {
        self.zeropage_indexed(bus, self.reg.y)
    }

    fn absolute(&mut self, bus: &mut NesBus) -> (Word, bool) {
        let lo = self.fetch(bus);
        let hi = self.fetch(bus);
        (Word::from_le_bytes([lo, hi]), false)
    }

    fn absolute_indexed(&mut self, bus: &mut NesBus, indexer: u8) -> (Word, bool) {
        let lo = self.fetch(bus);
        let hi = self.fetch(bus);
        let (lo, is_crossed) = lo.overflowing_add(indexer);
        let addr = Word::from_le_bytes([lo, hi]);
        (addr, is_crossed)
    }

    fn absolute_x(&mut self, bus: &mut NesBus) -> (Word, bool) {
        self.absolute_indexed(bus, self.reg.x)
    }

    fn absolute_y(&mut self, bus: &mut NesBus) -> (Word, bool) {
        self.absolute_indexed(bus, self.reg.y)
    }

    fn pre_indexed_indirect(&mut self, bus: &mut NesBus) -> Word {
        let ptr = self.fetch(bus);
        self.read(bus, Word::from_le_bytes([ptr, ZEROPAGE]));
        let ptr = ptr.wrapping_add(self.reg.x);
        let lo = self.read(bus, Word::from_le_bytes([ptr, ZEROPAGE]));
        let hi = self.read(bus, Word::from_le_bytes([ptr.wrapping_add(1), ZEROPAGE]));
        Word::from_le_bytes([lo, hi])
    }

    fn post_indexed_indirect(&mut self, bus: &mut NesBus) -> (Word, bool) {
        let ptr = self.fetch(bus);
        let lo = self.read(bus, Word::from_le_bytes([ptr, ZEROPAGE]));
        let hi = self.read(bus, Word::from_le_bytes([ptr.wrapping_add(1), ZEROPAGE]));
        let (lo, is_crossed) = lo.overflowing_add(self.reg.y);
        let addr = Word::from_le_bytes([lo, hi]);
        (addr, is_crossed)
    }

    // fn exec_opcode(&mut self, bus: &mut NesBus, opcode: Opcode) {}

    fn exec_relative(&mut self, bus: &mut NesBus, flag_callback: Relative) {
        let offset = self.fetch(bus) as i8;
        let is_branched = flag_callback(&self.reg.p);

        if !is_branched {
            return;
        }

        self.read(bus, self.reg.pc);
        let (idxed_lo, crossed) = self.reg.pc.lo.overflowing_add_signed(offset);
        self.reg.pc.lo = idxed_lo;

        if !crossed {
            return;
        }

        self.read(bus, self.reg.pc);
        let hi_fixer = if offset > 0 { 1 } else { -1 };
        self.reg.pc.hi = self.reg.pc.hi.wrapping_add_signed(hi_fixer);
    }
}

impl Cpu {
    fn update_nz(p: &mut Status, v: u8) {
        p.set_carry(v & 0x80 != 0);
        p.set_zero(v == 0);
    }

    fn _adc(reg: &mut Register, v: u8) {
        let old_a = reg.a;
        let res_u16 = old_a as u16 + v as u16 + reg.p.carry() as u16;
        let res = res_u16 as u8;

        let carry = 0xFF < res_u16;
        let overflow = (old_a ^ res) & (v ^ res) & 0x80 != 0;
        reg.p.set_carry(carry);
        reg.p.set_overflow(overflow);
        Self::update_nz(&mut reg.p, res);
        reg.a = res;
    }

    fn _cmp(p: &mut Status, v: u8, w: u8) {
        let (res, borrow) = v.overflowing_sub(w);
        p.set_carry(!borrow);
        Self::update_nz(p, res);
    }

    // TODO: Interrupt vector hijack
    fn brk_implied(&mut self, bus: &mut NesBus) {
        self.fetch(bus);
        self.push(bus, self.reg.pc.hi);
        self.push(bus, self.reg.pc.lo);
        self.push(bus, self.reg.p.as_byte(true));
        self.reg.pc.lo = self.read(bus, BRK_VECTOR);
        self.reg.p.set_interrupt(true);
        self.reg.pc.hi = self.read(bus, BRK_VECTOR.wrapping_add(1));
    }

    fn rti_implied(&mut self, bus: &mut NesBus) {
        self.read(bus, self.reg.pc);
        self.read(bus, self.reg.sp);
        self.reg.p = Status::from(self.pop(bus));
        self.reg.pc.lo = self.pop(bus);
        self.reg.pc.hi = self.pop(bus);
    }

    fn rts_implied(&mut self, bus: &mut NesBus) {
        self.read(bus, self.reg.pc);
        self.read(bus, self.reg.sp);
        self.reg.pc.lo = self.pop(bus);
        self.reg.pc.hi = self.pop(bus);
        self.reg.pc.carrying_inc();
    }

    fn pha_implied(&mut self, bus: &mut NesBus) {
        self.read(bus, self.reg.pc);
        self.push(bus, self.reg.a);
    }

    fn php_implied(&mut self, bus: &mut NesBus) {
        self.read(bus, self.reg.pc);
        self.push(bus, self.reg.p.as_byte(false));
    }

    fn pla_implied(&mut self, bus: &mut NesBus) {
        self.read(bus, self.reg.pc);
        self.read(bus, self.reg.sp);
        self.reg.a = self.pop(bus);
    }

    fn plp_implied(&mut self, bus: &mut NesBus) {
        self.read(bus, self.reg.pc);
        self.read(bus, self.reg.sp);
        self.reg.p = Status::from(self.pop(bus));
    }

    fn jsr_absolute(&mut self, bus: &mut NesBus) {
        let lo = self.fetch(bus);
        self.read(bus, self.reg.sp);
        self.push(bus, self.reg.pc.hi);
        self.push(bus, self.reg.pc.lo);
        self.reg.pc.hi = self.fetch(bus);
        self.reg.pc.lo = lo;
    }

    fn jmp_absolute(&mut self, bus: &mut NesBus) {
        let lo = self.fetch(bus);
        self.reg.pc.hi = self.fetch(bus);
        self.reg.pc.lo = lo;
    }

    fn jmp_indirect(&mut self, bus: &mut NesBus) {
        let mut ptr = Word::from(0);
        ptr.lo = self.fetch(bus);
        ptr.hi = self.fetch(bus);
        self.reg.pc.lo = self.read(bus, ptr);
        ptr.lo = ptr.lo.wrapping_add(1);
        self.reg.pc.hi = self.read(bus, ptr);
    }

    fn bcs_relative(status: &Status) -> bool {
        status.carry()
    }

    fn bcc_relative(status: &Status) -> bool {
        !status.carry()
    }

    fn beq_relative(status: &Status) -> bool {
        status.zero()
    }

    fn bne_relative(status: &Status) -> bool {
        !status.zero()
    }

    fn bvs_relative(status: &Status) -> bool {
        status.overflow()
    }

    fn bvc_relative(status: &Status) -> bool {
        !status.overflow()
    }

    fn bmi_relative(status: &Status) -> bool {
        status.negative()
    }

    fn bpl_relative(status: &Status) -> bool {
        !status.negative()
    }

    fn inx(reg: &mut Register) {
        reg.x = reg.x.wrapping_add(1);
        Self::update_nz(&mut reg.p, reg.x);
    }

    fn iny(reg: &mut Register) {
        reg.y = reg.y.wrapping_add(1);
        Self::update_nz(&mut reg.p, reg.y);
    }

    fn dex(reg: &mut Register) {
        reg.x = reg.x.wrapping_sub(1);
        Self::update_nz(&mut reg.p, reg.x);
    }

    fn dey(reg: &mut Register) {
        reg.y = reg.y.wrapping_sub(1);
        Self::update_nz(&mut reg.p, reg.y);
    }

    fn sec(reg: &mut Register) {
        reg.p.set_carry(true);
    }

    fn sed(reg: &mut Register) {
        reg.p.set_decimal(true);
    }

    fn sei(reg: &mut Register) {
        reg.p.set_interrupt(true);
    }

    fn clc(reg: &mut Register) {
        reg.p.set_carry(false);
    }

    fn cld(reg: &mut Register) {
        reg.p.set_decimal(false);
    }

    fn cli(reg: &mut Register) {
        reg.p.set_interrupt(false);
    }

    fn clv(reg: &mut Register) {
        reg.p.set_overflow(false);
    }

    fn tax(reg: &mut Register) {
        reg.x = reg.a;
        Self::update_nz(&mut reg.p, reg.x);
    }

    fn tay(reg: &mut Register) {
        reg.y = reg.a;
        Self::update_nz(&mut reg.p, reg.y);
    }

    fn txa(reg: &mut Register) {
        reg.a = reg.x;
        Self::update_nz(&mut reg.p, reg.a);
    }

    fn tya(reg: &mut Register) {
        reg.a = reg.y;
        Self::update_nz(&mut reg.p, reg.a);
    }

    fn tsx(reg: &mut Register) {
        reg.x = reg.sp.into();
        Self::update_nz(&mut reg.p, reg.x);
    }

    fn txs(reg: &mut Register) {
        reg.sp = reg.x.into();
    }

    fn nop(_reg: &mut Register, _m: u8) {}

    fn adc(reg: &mut Register, m: u8) {
        Self::_adc(reg, m);
    }

    fn sbc(reg: &mut Register, m: u8) {
        Self::_adc(reg, !m);
    }

    fn and(reg: &mut Register, m: u8) {
        reg.a &= m;
        Self::update_nz(&mut reg.p, reg.a);
    }

    fn ora(reg: &mut Register, m: u8) {
        reg.a |= m;
        Self::update_nz(&mut reg.p, reg.a);
    }

    fn eor(reg: &mut Register, m: u8) {
        reg.a ^= m;
        Self::update_nz(&mut reg.p, reg.a);
    }

    fn bit(reg: &mut Register, m: u8) {
        reg.p.set_negative(m & 0x80 != 0);
        reg.p.set_overflow(m & 0x40 != 0);
        reg.p.set_zero(reg.a & m == 0);
    }

    fn cmp(reg: &mut Register, m: u8) {
        Self::_cmp(&mut reg.p, reg.a, m);
    }

    fn cpx(reg: &mut Register, m: u8) {
        Self::_cmp(&mut reg.p, reg.x, m);
    }

    fn cpy(reg: &mut Register, m: u8) {
        Self::_cmp(&mut reg.p, reg.y, m);
    }

    fn lda(reg: &mut Register, m: u8) {
        reg.a = m;
        Self::update_nz(&mut reg.p, reg.a);
    }

    fn ldx(reg: &mut Register, m: u8) {
        reg.x = m;
        Self::update_nz(&mut reg.p, reg.x);
    }

    fn ldy(reg: &mut Register, m: u8) {
        reg.y = m;
        Self::update_nz(&mut reg.p, reg.y);
    }

    fn lax(reg: &mut Register, m: u8) {
        Self::lda(reg, m);
        Self::ldx(reg, m);
    }

    fn las(reg: &mut Register, m: u8) {
        let res = u8::from(reg.sp) & m;
        reg.a = res;
        reg.x = res;
        reg.sp = res.into();
        Self::update_nz(&mut reg.p, res);
    }

    fn alr(reg: &mut Register, m: u8) {
        Self::and(reg, m);
        let mut tmp_a = reg.a;
        Self::lsr(reg, &mut tmp_a);
        reg.a = tmp_a;
    }

    fn arr(reg: &mut Register, m: u8) {
        Self::and(reg, m);
        let mut tmp_a = reg.a;
        Self::ror(reg, &mut tmp_a);
        reg.a = tmp_a;

        let spec_bit = (reg.a ^ (reg.a << 1)) & 0x40;
        reg.p.set_overflow(spec_bit != 0);
        reg.p.set_carry(reg.a & 0x40 != 0);
    }

    fn anc(reg: &mut Register, m: u8) {
        Self::and(reg, m);
        reg.p.set_carry(reg.a & 0x80 != 0);
    }

    fn axs(reg: &mut Register, m: u8) {
        let ax = reg.a & reg.x;
        let (res, borrow) = ax.overflowing_sub(m);
        reg.x = res;
        reg.p.set_carry(!borrow);
        Self::update_nz(&mut reg.p, res);
    }

    fn asl(reg: &mut Register, m: &mut u8) {
        reg.p.set_carry(*m & 0x80 != 0);
        *m <<= 1;
        Self::update_nz(&mut reg.p, *m);
    }

    fn lsr(reg: &mut Register, m: &mut u8) {
        reg.p.set_carry(*m & 0x01 != 0);
        *m >>= 1;
        Self::update_nz(&mut reg.p, *m);
    }

    fn rol(reg: &mut Register, m: &mut u8) {
        let old_c = reg.p.carry();
        reg.p.set_carry(*m & 0x80 != 0);
        *m = (*m << 1) | if old_c { 0x01 } else { 0 };
        Self::update_nz(&mut reg.p, *m);
    }

    fn ror(reg: &mut Register, m: &mut u8) {
        let old_c = reg.p.carry();
        reg.p.set_carry(*m & 0x01 != 0);
        *m = (*m >> 1) | if old_c { 0x80 } else { 0 };
        Self::update_nz(&mut reg.p, *m);
    }

    fn inc(reg: &mut Register, m: &mut u8) {
        *m = m.wrapping_add(1);
        Self::update_nz(&mut reg.p, *m);
    }

    fn dec(reg: &mut Register, m: &mut u8) {
        *m = m.wrapping_sub(1);
        Self::update_nz(&mut reg.p, *m);
    }

    fn dcp(reg: &mut Register, m: &mut u8) {
        Self::dec(reg, m);
        Self::cmp(reg, *m);
    }

    fn isb(reg: &mut Register, m: &mut u8) {
        Self::inc(reg, m);
        Self::sbc(reg, *m);
    }

    fn rra(reg: &mut Register, m: &mut u8) {
        Self::ror(reg, m);
        Self::adc(reg, *m);
    }

    fn rla(reg: &mut Register, m: &mut u8) {
        Self::rol(reg, m);
        Self::and(reg, *m);
    }

    fn slo(reg: &mut Register, m: &mut u8) {
        Self::asl(reg, m);
        Self::ora(reg, *m);
    }

    fn sre(reg: &mut Register, m: &mut u8) {
        Self::lsr(reg, m);
        Self::eor(reg, *m);
    }

    fn sta(reg: &Register) -> u8 {
        reg.a
    }

    fn stx(reg: &Register) -> u8 {
        reg.x
    }

    fn sty(reg: &Register) -> u8 {
        reg.y
    }

    fn sax(reg: &Register) -> u8 {
        reg.a & reg.x
    }
}
