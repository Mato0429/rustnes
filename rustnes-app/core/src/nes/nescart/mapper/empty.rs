use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Empty;

impl MapperLogic for Empty {
    fn irq_active(&self) -> bool {
        false
    }

    fn cpu_step(&mut self) {}

    fn map_cpu_read(&mut self, _addr: u16) -> MappedCpuRead {
        MappedCpuRead::Openbus
    }

    fn map_cpu_write(&mut self, _addr: u16, _data: u8) -> MappedCpuWrite {
        MappedCpuWrite::Other
    }

    fn map_ppu_read(&mut self, _addr: u16) -> MappedPpuRead {
        MappedPpuRead::Openbus
    }

    fn map_ppu_write(&mut self, _addr: u16, _data: u8) -> MappedPpuWrite {
        MappedPpuWrite::Other
    }
}
