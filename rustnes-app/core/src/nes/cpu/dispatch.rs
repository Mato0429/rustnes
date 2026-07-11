use super::{
    opcode::Targetter,
    operation::{Branch, Modify, Operation, Read, Short, Unique, Write, WrongWrite},
    {Bus, Cpu, ZERO_PAGE},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Target {
    Accumulator,
    Immediate(u8),
    FixedPage(u16),
    SamePage(u16),
    DiffPage(u16),
}

impl Cpu {
    pub(super) fn exec_opcode<B: Bus>(&mut self, bus: &mut B, opcode: (Operation, Targetter)) {
        let (operation, targetter) = opcode;

        if let Targetter::Depend = targetter {
            match operation {
                Operation::Unique(unique) => self.dispatch_unique(bus, unique),
                Operation::Short(short) => self.dispatch_short(bus, short),
                Operation::Branch(branch) => self.dispatch_branch(bus, branch),
                _ => panic!(),
            };
        } else {
            let target = self.resolve_target(bus, targetter);
            match operation {
                Operation::Read(read_fn) => self.dispatch_read(bus, read_fn, target),
                Operation::Modify(modify_fn) => self.dispatch_modify(bus, modify_fn, target),
                Operation::Write(write_fn) => self.dispatch_write(bus, write_fn, target),
                Operation::WrongWrite(wrongwrite_fn) => {
                    self.dispatch_wrongwrite(bus, wrongwrite_fn, target)
                }
                _ => unreachable!(),
            };
        }
    }

    fn resolve_target(&mut self, bus: &mut impl Bus, target: Targetter) -> Target {
        match target {
            Targetter::Accumulator => {
                self.read_at_pc(bus);
                Target::Accumulator
            }

            Targetter::Immediate => {
                let value = self.fetch(bus);
                Target::Immediate(value)
            }

            Targetter::Zeropage => {
                let lo = self.fetch(bus);
                let base = u16::from_le_bytes([lo, ZERO_PAGE]);
                Target::FixedPage(base)
            }

            Targetter::ZeropageX => {
                let base = self.fetch(bus);
                self.read(bus, u16::from_le_bytes([base, ZERO_PAGE]));
                let indexed = u16::from_le_bytes([base.wrapping_add(self.reg.x), ZERO_PAGE]);
                Target::FixedPage(indexed)
            }

            Targetter::ZeropageY => {
                let base = self.fetch(bus);
                self.read(bus, u16::from_le_bytes([base, ZERO_PAGE]));
                let indexed = u16::from_le_bytes([base.wrapping_add(self.reg.y), ZERO_PAGE]);
                Target::FixedPage(indexed)
            }

            Targetter::Absolute => {
                let lo = self.fetch(bus);
                let hi = self.fetch(bus);
                let base = u16::from_le_bytes([lo, hi]);
                Target::FixedPage(base)
            }

            Targetter::AbsoluteX => {
                let lo = self.fetch(bus);
                let hi = self.fetch(bus);

                let (lo, is_crossed) = lo.overflowing_add(self.reg.x);
                let indexed = u16::from_le_bytes([lo, hi]);
                if is_crossed {
                    Target::DiffPage(indexed)
                } else {
                    Target::SamePage(indexed)
                }
            }

            Targetter::AbsoluteY => {
                let lo = self.fetch(bus);
                let hi = self.fetch(bus);

                let (lo, is_crossed) = lo.overflowing_add(self.reg.y);
                let indexed = u16::from_le_bytes([lo, hi]);
                if is_crossed {
                    Target::DiffPage(indexed)
                } else {
                    Target::SamePage(indexed)
                }
            }

            Targetter::XIdxedInd => {
                let ptr = self.fetch(bus);
                let idxed_ptr = ptr.wrapping_add(self.reg.x);
                self.read(bus, u16::from_le_bytes([ptr, ZERO_PAGE])); // dummy read to index

                let vec_lo = u16::from_le_bytes([idxed_ptr, ZERO_PAGE]);
                let vec_hi = u16::from_le_bytes([idxed_ptr.wrapping_add(1), ZERO_PAGE]);
                let lo = self.read(bus, vec_lo);
                let hi = self.read(bus, vec_hi);

                let addr = u16::from_le_bytes([lo, hi]);
                Target::FixedPage(addr)
            }

            Targetter::IndYIdxed => {
                let ptr = self.fetch(bus);

                let vec_lo = u16::from_le_bytes([ptr, ZERO_PAGE]);
                let vec_hi = u16::from_le_bytes([ptr.wrapping_add(1), ZERO_PAGE]);
                let lo = self.read(bus, vec_lo);
                let hi = self.read(bus, vec_hi);

                let (idxed_lo, is_crossed) = lo.overflowing_add(self.reg.y);
                let indexed = u16::from_le_bytes([idxed_lo, hi]);
                if is_crossed {
                    Target::DiffPage(indexed)
                } else {
                    Target::SamePage(indexed)
                }
            }

            Targetter::Depend => panic!(),
        }
    }

    fn dispatch_unique(&mut self, bus: &mut impl Bus, unique: Unique) {
        match unique {
            Unique::JamUndefined => self.jam_undefined(bus),
            Unique::BrkImplied => self.brk_implied(bus),
            Unique::RtiImplied => self.rti_implied(bus),
            Unique::RtsImplied => self.rts_implied(bus),
            Unique::PhaImplied => self.pha_implied(bus),
            Unique::PhpImplied => self.php_implied(bus),
            Unique::PlaImplied => self.pla_implied(bus),
            Unique::PlpImplied => self.plp_implied(bus),
            Unique::JsrAbsolute => self.jsr_absolute(bus),
            Unique::JmpAbsolute => self.jmp_absolute(bus),
            Unique::JmpIndirect => self.jmp_indirect(bus),
            Unique::NopImplied => self.nop_implied(bus),
        };
    }

    fn dispatch_short(&mut self, bus: &mut impl Bus, short: Short) {
        self.read_at_pc(bus);
        short(self)
    }

    fn dispatch_branch(&mut self, bus: &mut impl Bus, branch: Branch) {
        let offset = self.fetch(bus) as i8;

        let is_branched = branch(self);
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

    fn dispatch_read(&mut self, bus: &mut impl Bus, read_fn: Read, target: Target) {
        let m = match target {
            Target::Accumulator => self.reg.a,
            Target::Immediate(m) => m,
            Target::FixedPage(addr) | Target::SamePage(addr) => self.read(bus, addr),
            Target::DiffPage(addr) => {
                self.read(bus, addr); // dummy read to fix ADH
                self.read(bus, addr.wrapping_add(0x0100)) // fix ADH
            }
        };

        read_fn(self, m);
    }

    fn dispatch_modify(&mut self, bus: &mut impl Bus, modify_fn: Modify, target: Target) {
        let addr = match target {
            Target::Accumulator => {
                let mut tmp_a = self.reg.a;
                modify_fn(self, &mut tmp_a);
                self.reg.a = tmp_a;
                return;
            }
            Target::FixedPage(addr) => addr,
            Target::SamePage(addr) => {
                self.read(bus, addr); // dummy read
                addr
            }
            Target::DiffPage(addr) => {
                self.read(bus, addr); // dummy read to fix ADH
                addr.wrapping_add(0x0100) // fix ADH
            }
            Target::Immediate(_) => panic!(),
        };

        let mut m = self.read(bus, addr);
        self.write(bus, addr, m); // write back
        modify_fn(self, &mut m); // operate
        self.write(bus, addr, m);
    }

    fn dispatch_write(&mut self, bus: &mut impl Bus, write_fn: Write, target: Target) {
        let addr = match target {
            Target::FixedPage(addr) => addr,
            Target::SamePage(addr) => {
                self.read(bus, addr); // dummy read
                addr
            }
            Target::DiffPage(addr) => {
                self.read(bus, addr); // dummy read to fix ADH
                addr.wrapping_add(0x0100) // fix ADH
            }
            Target::Accumulator | Target::Immediate(_) => panic!(),
        };

        let m = write_fn(self);
        self.write(bus, addr, m);
    }

    fn dispatch_wrongwrite(
        &mut self,
        bus: &mut impl Bus,
        wrongwrite_fn: WrongWrite,
        target: Target,
    ) {
        let addr = match target {
            Target::SamePage(addr) | Target::DiffPage(addr) => {
                self.read(bus, addr);
                addr
            }
            _ => panic!(),
        };

        let [lo, hi] = addr.to_le_bytes();
        let m = wrongwrite_fn(self, hi);

        if let Target::SamePage(_) = target {
            self.write(bus, addr, m);
        } else {
            let addr = u16::from_le_bytes([lo, m]);
            self.write(bus, addr, m);
        }
    }
}
