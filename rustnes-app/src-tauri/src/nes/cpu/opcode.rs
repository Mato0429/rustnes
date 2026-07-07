use super::{Cpu, Register, Status, Word};
use crate::nes::NesBus;

pub type Addresser = fn(&mut Cpu, bus: &mut NesBus) -> Word;

pub type Unique = fn(&mut Cpu, bus: &mut NesBus);
pub type Relative = fn(&Status) -> bool;
pub type Short = fn(&mut Register);
pub type Read = fn(&mut Register, u8);
pub type Modify = fn(&mut Register, &mut u8);
pub type Write = fn(&Register) -> u8;

#[derive(Debug, Clone, Copy)]
pub enum Bounding {
    Safe,
    Unsafe,
}

#[derive(Debug, Clone, Copy)]
pub enum Indexer {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
pub enum IndexType {
    Unindexed,
    Indexed(Bounding, Indexer),
}

#[derive(Debug, Clone, Copy)]
pub enum MemAccess {
    Read(Read),
    Modify(Modify),
    Write(Write),
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Unique(Unique),
    Relative(Relative),
    Short(Short),
    Accumulator(Modify),
    MemAccess(MemAccess, IndexType),
}

// Opcode
// + Unique
// + Relative
// + Short
// + Accumulator
// + MemAccess
//   + Unindexed
//   + Indexed(bounding, indexer)

/*
    Zeropage,
    ZeropageX,
    ZeropageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    XIndexedIndirect,
    IndirectYIndexed,
*/
