mod dispatch;
mod opcode;
mod operation;
mod registers;

use opcode::*;
use registers::{Register, Status};

const ZERO_PAGE: u8 = 0x00;
const STACK_PAGE: u8 = 0x01;

const RESET_VECTOR: u16 = 0xFFFC;
const BRK_VECTOR: u16 = 0xFFFE;

const DEFAULT_A: u8 = 0x00;
const DEFAULT_X: u8 = 0x00;
const DEFAULT_Y: u8 = 0x00;
const DEFAULT_P: u8 = 0x34;
const DEFAULT_SP: u8 = 0x00;
const DEFAULT_PC: u16 = 0x8000; // This value is a placeholder. PC is set by the RESET VECTOR

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Debug, Clone, Copy)]
pub struct Cpu {
    pub is_jammed: bool,
    pub total_cycle: u128,

    pub reg: Register,
}

impl Cpu {
    #[allow(clippy::new_without_default)]
    // TODO: Enable to override initial registers(include PC), wram, and etc...
    pub fn new() -> Self {
        Self {
            is_jammed: false,
            total_cycle: 0,

            reg: Register {
                a: DEFAULT_A,
                x: DEFAULT_X,
                y: DEFAULT_Y,
                p: Status::from(DEFAULT_P),
                sp: DEFAULT_SP,
                pc: DEFAULT_PC,
            },
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
        // TODO: Handle interrupt here
        let opcode = self.fetch(bus);
        let opcode = OPCODE_TABLE[opcode as usize];
        self.exec_opcode(bus, opcode);
    }

    fn stackpointer(&self) -> u16 {
        u16::from_be_bytes([self.reg.sp, STACK_PAGE])
    }

    fn consume_cycle(&mut self) {
        self.total_cycle += 1;
    }

    fn read(&mut self, bus: &mut impl Bus, addr: u16) -> u8 {
        self.consume_cycle();

        // TODO: Activate dma like: fn process_dma(&mut self, last_addr: impl Into<u16>, page: u8)
        bus.read(addr)
    }

    fn write(&mut self, bus: &mut impl Bus, addr: u16, data: u8) {
        self.consume_cycle();

        // TODO: Hook dma here
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
}
