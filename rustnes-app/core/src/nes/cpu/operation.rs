use super::{Bus, Cpu, Status, BRK_VECTOR};

const LXA_CONST: u8 = 0xEE;
const ANE_CONST: u8 = 0xEE;

pub type Short = fn(&mut Cpu);
pub type Branch = fn(&Cpu) -> bool;
pub type Read = fn(&mut Cpu, u8);
pub type Modify = fn(&mut Cpu, &mut u8);
pub type Write = fn(&Cpu) -> u8;
pub type WrongWrite = fn(&mut Cpu, u8) -> u8;

#[derive(Debug, Clone, Copy)]
pub enum Unique {
    JamUndefined,
    BrkImplied,
    RtiImplied,
    RtsImplied,
    PhaImplied,
    PhpImplied,
    PlaImplied,
    PlpImplied,
    JsrAbsolute,
    JmpAbsolute,
    JmpIndirect,
    NopImplied,
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Unique(Unique),
    Short(Short),
    Branch(Branch),
    Read(Read),
    Modify(Modify),
    Write(Write),
    WrongWrite(WrongWrite),
}

impl Cpu {
    fn set_nz(&mut self, v: u8) {
        self.reg.p.set(Status::N, v & 0x80 != 0);
        self.reg.p.set(Status::Z, v == 0);
    }

    fn set_a(&mut self, v: u8) {
        self.reg.a = v;
        self.set_nz(v);
    }

    fn set_x(&mut self, v: u8) {
        self.reg.x = v;
        self.set_nz(v);
    }

    fn set_y(&mut self, v: u8) {
        self.reg.y = v;
        self.set_nz(v);
    }

    fn add_with_carry(&mut self, m: u8) {
        let old_a = self.reg.a;
        let res_u16 = old_a as u16 + m as u16 + self.reg.p.contains(Status::C) as u16;

        self.set_a(res_u16 as u8);
        let carry = 0xFF < res_u16;
        let overflow = (old_a ^ self.reg.a) & (m ^ self.reg.a) & 0x80 != 0;
        self.reg.p.set(Status::C, carry);
        self.reg.p.set(Status::V, overflow);
    }

    fn compare(&mut self, v: u8, m: u8) {
        let (res, borrow) = v.overflowing_sub(m);
        self.reg.p.set(Status::C, !borrow);
        self.set_nz(res);
    }
}

impl Cpu {
    pub(super) fn jam_undefined<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus); // TODO: unstable read
        self.is_jammed = true;
    }

    pub(super) fn nmi<B: Bus>(&mut self, bus: &mut B) {
        self.fetch(bus); // discard the padding byte
        let [pcl, pch] = self.reg.pc.to_le_bytes();
        self.push(bus, pch);
        self.push(bus, pcl);
        self.push(bus, self.reg.p.as_byte(true));
        self.reg.p.insert(Status::I);
        let lo = self.read(bus, 0xFFFA);
        let hi = self.read(bus, 0xFFFB);
        self.reg.pc = u16::from_le_bytes([lo, hi]);
    }

    pub(super) fn brk_implied<B: Bus>(&mut self, bus: &mut B) {
        self.fetch(bus); // discard the padding byte
        let [pcl, pch] = self.reg.pc.to_le_bytes();
        self.push(bus, pch);
        self.push(bus, pcl);
        self.push(bus, self.reg.p.as_byte(true));
        self.reg.p.insert(Status::I);
        let lo = self.read(bus, BRK_VECTOR);
        let hi = self.read(bus, BRK_VECTOR.wrapping_add(1));
        self.reg.pc = u16::from_le_bytes([lo, hi]);
    }

    pub(super) fn rti_implied<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus);
        self.read_at_sp(bus);
        self.reg.p = Status::from(self.pop(bus));
        let lo = self.pop(bus);
        let hi = self.pop(bus);
        self.reg.pc = u16::from_le_bytes([lo, hi]);
    }

    pub(super) fn rts_implied<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus);
        self.read_at_sp(bus);
        let lo = self.pop(bus);
        let hi = self.pop(bus);
        self.reg.pc = u16::from_le_bytes([lo, hi]);
        self.read_at_pc(bus);
        self.reg.pc = self.reg.pc.wrapping_add(1);
    }

    pub(super) fn pha_implied<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus);
        self.push(bus, self.reg.a);
    }

    pub(super) fn php_implied<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus);
        self.push(bus, self.reg.p.as_byte(true));
    }

    pub(super) fn pla_implied<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus);
        self.read_at_sp(bus);
        let popped = self.pop(bus);
        self.set_a(popped);
    }

    pub(super) fn plp_implied<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus);
        self.read_at_sp(bus);
        let popped = self.pop(bus);
        self.reg.p = Status::from(popped);
    }

    pub(super) fn jsr_absolute<B: Bus>(&mut self, bus: &mut B) {
        let new_lo = self.fetch(bus);
        self.read_at_sp(bus);
        let [old_lo, old_hi] = self.reg.pc.to_le_bytes();
        self.push(bus, old_hi);
        self.push(bus, old_lo);
        let new_hi = self.read(bus, self.reg.pc);
        self.reg.pc = u16::from_le_bytes([new_lo, new_hi]);
    }

    pub(super) fn jmp_absolute<B: Bus>(&mut self, bus: &mut B) {
        let new_lo = self.fetch(bus);
        let new_hi = self.read(bus, self.reg.pc);
        self.reg.pc = u16::from_le_bytes([new_lo, new_hi]);
    }

    pub(super) fn jmp_indirect<B: Bus>(&mut self, bus: &mut B) {
        let ptr_lo = self.fetch(bus);
        let ptr_hi = self.fetch(bus);
        let lo = self.read(bus, u16::from_le_bytes([ptr_lo, ptr_hi]));
        let hi = self.read(bus, u16::from_le_bytes([ptr_lo.wrapping_add(1), ptr_hi]));
        self.reg.pc = u16::from_le_bytes([lo, hi]);
    }

    pub(super) fn nop_implied<B: Bus>(&mut self, bus: &mut B) {
        self.read_at_pc(bus);
    }
}

impl Cpu {
    pub(super) fn inx(&mut self) {
        self.set_x(self.reg.x.wrapping_add(1));
    }

    pub(super) fn iny(&mut self) {
        self.set_y(self.reg.y.wrapping_add(1));
    }

    pub(super) fn dex(&mut self) {
        self.set_x(self.reg.x.wrapping_sub(1));
    }

    pub(super) fn dey(&mut self) {
        self.set_y(self.reg.y.wrapping_sub(1));
    }

    pub(super) fn clc(&mut self) {
        self.reg.p.remove(Status::C);
    }

    pub(super) fn cld(&mut self) {
        self.reg.p.remove(Status::D);
    }

    pub(super) fn cli(&mut self) {
        self.reg.p.remove(Status::I);
    }

    pub(super) fn clv(&mut self) {
        self.reg.p.remove(Status::V);
    }

    pub(super) fn sec(&mut self) {
        self.reg.p.insert(Status::C);
    }

    pub(super) fn sed(&mut self) {
        self.reg.p.insert(Status::D);
    }

    pub(super) fn sei(&mut self) {
        self.reg.p.insert(Status::I);
    }

    pub(super) fn tax(&mut self) {
        self.set_x(self.reg.a);
    }

    pub(super) fn tay(&mut self) {
        self.set_y(self.reg.a);
    }

    pub(super) fn txa(&mut self) {
        self.set_a(self.reg.x);
    }

    pub(super) fn tya(&mut self) {
        self.set_a(self.reg.y);
    }

    pub(super) fn tsx(&mut self) {
        self.set_x(self.reg.sp);
    }

    pub(super) fn txs(&mut self) {
        self.reg.sp = self.reg.x;
    }
}

impl Cpu {
    pub(super) fn bcs(&self) -> bool {
        self.reg.p.contains(Status::C)
    }

    pub(super) fn bcc(&self) -> bool {
        !self.reg.p.contains(Status::C)
    }

    pub(super) fn beq(&self) -> bool {
        self.reg.p.contains(Status::Z)
    }

    pub(super) fn bne(&self) -> bool {
        !self.reg.p.contains(Status::Z)
    }

    pub(super) fn bvs(&self) -> bool {
        self.reg.p.contains(Status::V)
    }

    pub(super) fn bvc(&self) -> bool {
        !self.reg.p.contains(Status::V)
    }

    pub(super) fn bmi(&self) -> bool {
        self.reg.p.contains(Status::N)
    }

    pub(super) fn bpl(&self) -> bool {
        !self.reg.p.contains(Status::N)
    }
}

impl Cpu {
    pub(super) fn nop(&mut self, _m: u8) {}

    pub(super) fn adc(&mut self, m: u8) {
        self.add_with_carry(m);
    }

    pub(super) fn sbc(&mut self, m: u8) {
        self.add_with_carry(!m);
    }

    pub(super) fn and(&mut self, m: u8) {
        self.set_a(self.reg.a & m);
    }

    pub(super) fn ora(&mut self, m: u8) {
        self.set_a(self.reg.a | m);
    }

    pub(super) fn eor(&mut self, m: u8) {
        self.set_a(self.reg.a ^ m);
    }

    pub(super) fn cmp(&mut self, m: u8) {
        self.compare(self.reg.a, m);
    }

    pub(super) fn cpx(&mut self, m: u8) {
        self.compare(self.reg.x, m);
    }

    pub(super) fn cpy(&mut self, m: u8) {
        self.compare(self.reg.y, m);
    }

    pub(super) fn bit(&mut self, m: u8) {
        self.reg.p.set(Status::N, m & 0x80 != 0);
        self.reg.p.set(Status::V, m & 0x40 != 0);
        self.reg.p.set(Status::Z, self.reg.a & m == 0);
    }

    pub(super) fn lda(&mut self, m: u8) {
        self.set_a(m);
    }

    pub(super) fn ldx(&mut self, m: u8) {
        self.set_x(m);
    }

    pub(super) fn ldy(&mut self, m: u8) {
        self.set_y(m);
    }

    pub(super) fn lax(&mut self, m: u8) {
        self.lda(m);
        self.ldx(m);
    }

    pub(super) fn lxa(&mut self, m: u8) {
        let m = m & (self.reg.a | LXA_CONST);
        self.lax(m);
    }

    pub(super) fn las(&mut self, m: u8) {
        let m = self.reg.sp & m;
        self.reg.sp = m;
        self.lax(m);
    }

    pub(super) fn anc(&mut self, m: u8) {
        self.and(m);
        self.reg.p.set(Status::C, self.reg.a & 0x80 != 0);
    }
}

impl Cpu {
    pub(super) fn alr(&mut self, m: u8) {
        self.and(m);
        let mut tmp_a = self.reg.a;
        self.lsr(&mut tmp_a);
        self.reg.a = tmp_a;
    }

    pub(super) fn arr(&mut self, m: u8) {
        self.and(m);
        let mut tmp_a = self.reg.a;
        self.ror(&mut tmp_a);

        let spec_bit = (tmp_a ^ (tmp_a << 1)) & 0x40;
        self.reg.p.set(Status::C, tmp_a & 0x40 != 0);
        self.reg.p.set(Status::V, spec_bit != 0);
        self.reg.a = tmp_a;
    }

    pub(super) fn axs(&mut self, m: u8) {
        let ax = self.reg.a & self.reg.x;
        self.reg.p.set(Status::C, ax >= m);
        self.set_x(ax.wrapping_sub(m));
    }

    pub(super) fn ane(&mut self, m: u8) {
        self.set_a((self.reg.a | ANE_CONST) & self.reg.x & m);
    }

    pub(super) fn inc(&mut self, m: &mut u8) {
        *m = m.wrapping_add(1);
        self.set_nz(*m);
    }

    pub(super) fn dec(&mut self, m: &mut u8) {
        *m = m.wrapping_sub(1);
        self.set_nz(*m);
    }

    pub(super) fn asl(&mut self, m: &mut u8) {
        self.reg.p.set(Status::C, *m & 0x80 != 0);
        *m <<= 1;
        self.set_nz(*m)
    }

    pub(super) fn lsr(&mut self, m: &mut u8) {
        self.reg.p.set(Status::C, *m & 0x01 != 0);
        *m >>= 1;
        self.set_nz(*m)
    }

    pub(super) fn rol(&mut self, m: &mut u8) {
        let old_c = self.reg.p.contains(Status::C);
        self.reg.p.set(Status::C, *m & 0x80 != 0);
        *m = *m << 1 | if old_c { 0x01 } else { 0 };
        self.set_nz(*m);
    }

    pub(super) fn ror(&mut self, m: &mut u8) {
        let old_c = self.reg.p.contains(Status::C);
        self.reg.p.set(Status::C, *m & 0x01 != 0);
        *m = *m >> 1 | if old_c { 0x80 } else { 0 };
        self.set_nz(*m);
    }

    pub(super) fn dcp(&mut self, m: &mut u8) {
        self.dec(m);
        self.cmp(*m);
    }

    pub(super) fn isb(&mut self, m: &mut u8) {
        self.inc(m);
        self.sbc(*m);
    }

    pub(super) fn rra(&mut self, m: &mut u8) {
        self.ror(m);
        self.adc(*m);
    }

    pub(super) fn rla(&mut self, m: &mut u8) {
        self.rol(m);
        self.and(*m);
    }

    pub(super) fn slo(&mut self, m: &mut u8) {
        self.asl(m);
        self.ora(*m);
    }

    pub(super) fn sre(&mut self, m: &mut u8) {
        self.lsr(m);
        self.eor(*m);
    }
}

impl Cpu {
    pub(super) fn sta(&self) -> u8 {
        self.reg.a
    }

    pub(super) fn stx(&self) -> u8 {
        self.reg.x
    }

    pub(super) fn sty(&self) -> u8 {
        self.reg.y
    }

    pub(super) fn sax(&self) -> u8 {
        self.reg.a & self.reg.x
    }
}

impl Cpu {
    pub(super) fn sha(&mut self, adh: u8) -> u8 {
        self.reg.a & self.reg.x & adh.wrapping_add(1)
    }

    pub(super) fn shx(&mut self, adh: u8) -> u8 {
        self.reg.x & adh.wrapping_add(1)
    }

    pub(super) fn shy(&mut self, adh: u8) -> u8 {
        self.reg.y & adh.wrapping_add(1)
    }

    pub(super) fn shs(&mut self, adh: u8) -> u8 {
        self.reg.sp = self.reg.a & self.reg.x;
        self.reg.sp & adh.wrapping_add(1)
    }
}
