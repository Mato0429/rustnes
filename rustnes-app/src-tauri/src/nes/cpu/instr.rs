use super::Cpu;

pub type UniqueFn = fn(&mut Cpu);

#[derive(Debug, Clone, Copy)]
pub enum Instr {
    Unique(UniqueFn),
}
