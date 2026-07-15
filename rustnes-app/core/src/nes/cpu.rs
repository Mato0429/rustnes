mod dispatch;
mod opcode;
mod operation;
mod registers;

use opcode::*;
use registers::{Register, Status};

const ZERO_PAGE: u8 = 0x00;
const STACK_PAGE: u8 = 0x01;

const OAMADDR: u16 = 0x2004;
const OAMDMA_ADDR: u16 = 0x4014;

const RESET_VECTOR: u16 = 0xFFFC;
const NMI_VECTOR: u16 = 0xFFFA;
const IRQ_VECTOR: u16 = 0xFFFE;

const DEFAULT_A: u8 = 0x00;
const DEFAULT_X: u8 = 0x00;
const DEFAULT_Y: u8 = 0x00;
const DEFAULT_P: u8 = 0x34;
const DEFAULT_SP: u8 = 0x00;
const DEFAULT_PC: u16 = 0x8000; // This value is a placeholder. PC is set by the RESET VECTOR

pub trait Bus {
    fn nmi_active(&self) -> bool;
    fn irq_active(&self) -> bool;
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Debug, Clone, Copy)]
enum OamDmaStat {
    Disabled,
    Pending { page: u8 },
    Active,
}

#[derive(Debug, Clone, Copy)]
pub struct Cpu {
    pub is_jammed: bool,
    pub total_cycle: u128,
    pub prev_nmi: bool,

    pub reg: Register,

    oamdma: OamDmaStat,
    is_put_cyc: bool,
}

impl Cpu {
    #[allow(clippy::new_without_default)]
    // TODO: Enable to override initial registers(include PC), wram, and etc...
    pub fn new() -> Self {
        Self {
            is_jammed: false,
            total_cycle: 0,
            prev_nmi: false,

            reg: Register {
                a: DEFAULT_A,
                x: DEFAULT_X,
                y: DEFAULT_Y,
                p: Status::from(DEFAULT_P),
                sp: DEFAULT_SP,
                pc: DEFAULT_PC,
            },

            oamdma: OamDmaStat::Disabled,
            is_put_cyc: false,
        }
    }

    pub fn reset(&mut self, bus: &mut impl Bus) {
        self.is_jammed = false;
        self.read_at_pc(bus);
        self.read_at_pc(bus);
        for _ in 0..3 {
            self.read_at_sp(bus);
            self.reg.sp = self.reg.sp.wrapping_sub(1);
        }
        self.reg.p.set(Status::I, true);
        let lo = self.read(bus, RESET_VECTOR);
        let hi = self.read(bus, RESET_VECTOR.wrapping_add(1));
        self.reg.pc = u16::from_le_bytes([lo, hi]);
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        // TODO: On the actual nes, interrupts is polled before the final cycle of steps
        // NMI edge detection
        if !self.prev_nmi && bus.nmi_active() {
            self.read_at_pc(bus);
            self.handle_interrupt(bus, NMI_VECTOR, false);
        }
        self.prev_nmi = bus.nmi_active();

        // IRQ level detection
        if bus.irq_active() && !self.reg.p.contains(Status::I) {
            self.read_at_pc(bus);
            self.handle_interrupt(bus, IRQ_VECTOR, false);
        }

        let opcode = self.fetch(bus);

        /*
        println!(
            "{:04X} OP:{:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:}",
            self.reg.pc,
            opcode,
            self.reg.a,
            self.reg.x,
            self.reg.y,
            self.reg.p,
            self.reg.sp,
            self.total_cycle
        );
        */

        let opcode = OPCODE_TABLE[opcode as usize];
        self.exec_opcode(bus, opcode);
    }

    fn stackpointer(&self) -> u16 {
        u16::from_le_bytes([self.reg.sp, STACK_PAGE])
    }

    fn sync_internal(&mut self) {
        self.total_cycle += 1;
        self.is_put_cyc ^= true;
    }

    fn read(&mut self, bus: &mut impl Bus, addr: u16) -> u8 {
        self.sync_internal();
        let byte = bus.read(addr);

        if let OamDmaStat::Pending { page } = self.oamdma {
            self.transfer_sprites(bus, page, addr);
        }

        byte
    }

    fn write(&mut self, bus: &mut impl Bus, addr: u16, data: u8) {
        self.sync_internal();

        if addr == OAMDMA_ADDR {
            // Modify instructions can override the page
            if let OamDmaStat::Disabled | OamDmaStat::Pending { page: _ } = self.oamdma {
                self.oamdma = OamDmaStat::Pending { page: data };
            };
        }

        bus.write(addr, data);
    }

    fn read_at_pc(&mut self, bus: &mut impl Bus) {
        self.read(bus, self.reg.pc);
    }

    fn read_at_sp(&mut self, bus: &mut impl Bus) {
        self.read(bus, self.stackpointer());
    }

    fn fetch(&mut self, bus: &mut impl Bus) -> u8 {
        let byte = self.read(bus, self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        byte
    }

    fn push(&mut self, bus: &mut impl Bus, data: u8) {
        self.write(bus, self.stackpointer(), data);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
    }

    fn pop(&mut self, bus: &mut impl Bus) -> u8 {
        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.read(bus, self.stackpointer())
    }

    fn transfer_sprites(&mut self, bus: &mut impl Bus, page: u8, last_addr: u16) {
        self.oamdma = OamDmaStat::Active;

        if self.is_put_cyc {
            self.read(bus, last_addr); // dummy read
        }

        for idx in 0..=255 {
            let addr = u16::from_le_bytes([idx, page]);
            let data = self.read(bus, addr);
            self.write(bus, OAMADDR, data);
        }
    }
}
