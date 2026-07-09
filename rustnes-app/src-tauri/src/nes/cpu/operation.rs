use super::{Bus, Cpu, Status};

impl Cpu {
    fn update_nz(&mut self, v: u8) {
        self.reg.p.set(Status::N, v & 0x80 != 0);
        self.reg.p.set(Status::Z, v == 0);
    }

    pub(super) fn brk_implied<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn rts_implied<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn rti_implied<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn pha_implied<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn php_implied<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn pla_implied<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn plp_implied<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn jsr_absolute<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn jmp_absolute<B: Bus>(&mut self, bus: &mut B) {}

    pub(super) fn jmp_indirect<B: Bus>(&mut self, bus: &mut B) {}
}
