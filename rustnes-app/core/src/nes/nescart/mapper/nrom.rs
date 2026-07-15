use super::*;
use crate::nes::nescart::mirroring::{Mirroring, VramTarget};

#[derive(Debug, Clone, Copy)]
pub struct Nrom {
    mirroring: Mirroring,
    prgram_size: u32,
    prgrom_size: u32,
    chrrom_size: u32,
}

impl Nrom {
    pub fn new(args: MapperArgs) -> Self {
        let mirroring = if args.vertical_nt {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        Self {
            mirroring,
            prgram_size: args.prgram_size,
            prgrom_size: args.prgrom_size,
            chrrom_size: args.chrrom_size,
        }
    }
}

impl MapperLogic for Nrom {
    fn irq_active(&self) -> bool {
        false
    }

    fn cpu_step(&mut self) {}

    fn map_cpu_read(&mut self, addr: u16) -> MappedCpuRead {
        match addr {
            // Unmapped
            0x4020..=0x5FFF => MappedCpuRead::Openbus,

            // PrgRam
            0x6000..=0x7FFF => {
                let offset = (addr & 0x1FFF) as u32;
                if offset < self.prgram_size {
                    MappedCpuRead::PrgRam(offset)
                } else {
                    MappedCpuRead::Openbus
                }
            }

            // PrgRom
            0x8000..=u16::MAX => {
                let mask = if self.prgrom_size <= 0x4000 {
                    0x3FFF
                } else {
                    0x7FFF
                };
                let offset = (addr as u32) & mask;

                if offset < self.prgrom_size {
                    MappedCpuRead::PrgRom(offset)
                } else {
                    MappedCpuRead::Openbus
                }
            }

            _ => panic!("address out of range: {addr:#06X}"),
        }
    }

    fn map_cpu_write(&mut self, addr: u16, data: u8) -> MappedCpuWrite {
        match addr {
            // Unmapped
            0x4020..=0x5FFF => MappedCpuWrite::Other,

            // PrgRam
            0x6000..=0x7FFF => {
                let offset = (addr & 0x1FFF) as u32;
                if offset < self.prgram_size {
                    MappedCpuWrite::PrgRam(offset, data)
                } else {
                    MappedCpuWrite::Other
                }
            }

            // PrgRom
            0x8000..=u16::MAX => MappedCpuWrite::Other,

            _ => panic!("address out of range: {addr:#06X}"),
        }
    }

    fn map_ppu_read(&mut self, addr: u16) -> MappedPpuRead {
        match addr {
            // ChrRom
            0x0000..=0x1FFF => {
                let offset = addr as u32;
                if offset < self.chrrom_size {
                    MappedPpuRead::ChrRom(offset)
                } else {
                    MappedPpuRead::Openbus
                }
            }

            // Graphics
            0x2000..=0x2FFF => {
                let (addr, target) = self.mirroring.resolve(addr);
                match target {
                    VramTarget::Internal => MappedPpuRead::InternalVram(addr),
                    VramTarget::External => unreachable!(),
                }
            }

            _ => panic!("address out of range: {addr:#06X}"),
        }
    }

    fn map_ppu_write(&mut self, addr: u16, data: u8) -> MappedPpuWrite {
        match addr {
            // ChrRom
            0x0000..=0x1FFF => MappedPpuWrite::Other,

            // Graphics
            0x2000..=0x2FFF => {
                let (addr, target) = self.mirroring.resolve(addr);
                match target {
                    VramTarget::Internal => MappedPpuWrite::InternalVram(addr, data),
                    VramTarget::External => unreachable!(),
                }
            }

            _ => panic!("address out of range: {addr:#06X}"),
        }
    }
}
