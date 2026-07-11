use enum_dispatch::enum_dispatch;

#[derive(Debug, Clone, Copy)]
pub enum MappedCpuRead {
    MapperChip(u8),
    PrgRom(u32),
    PrgRam(u32),
    Openbus,
}

#[derive(Debug, Clone, Copy)]
pub enum MappedCpuWrite {
    Other, // Handled by the mapper or ignored
    PrgRam(u32, u8),
}

#[derive(Debug, Clone, Copy)]
pub enum MappedPpuRead {
    MapperChip(u8),
    InternalVram(u16),
    ChrRom(u32),
    ChrRam(u32),
    Openbus,
}

#[derive(Debug, Clone, Copy)]
pub enum MappedPpuWrite {
    Other, //  Handled by the mapper or ignored
    InternalVram(u16, u8),
    ChrRam(u32, u8),
}

#[enum_dispatch]
pub trait MapperLogic {
    fn irq_active(&self) -> bool;
    fn cpu_step(&mut self);
    fn map_cpu_read(&mut self, addr: u16) -> MappedCpuRead;
    fn map_cpu_write(&mut self, addr: u16, data: u8) -> MappedCpuWrite;
    fn map_ppu_read(&mut self, addr: u16) -> MappedPpuRead;
    fn map_ppu_write(&mut self, addr: u16, data: u8) -> MappedPpuWrite;
}
