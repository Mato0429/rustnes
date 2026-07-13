use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Nrom {
    mirroring: Mirroring,
    prgram_size: u32,
    prgrom_size: u32,
    chrrom_size: u32,
}

impl Nrom {
    pub fn new(args: MapperArgs) -> Self {
        let mirroring = if args.horizontal_nt {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
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
            0x0000..0x4020 => unreachable!(),
            0x4020..0x6000 => MappedCpuRead::Openbus,

            0x6000..0x8000 => {
                let offset = (addr & 0x1FFF) as u32;
                if offset < self.prgram_size {
                    MappedCpuRead::PrgRam(offset)
                } else {
                    MappedCpuRead::Openbus
                }
            }

            0x8000..0xC000 => {
                let offset = (addr & 0x3FFF) as u32;
                if offset < self.prgrom_size {
                    MappedCpuRead::PrgRom(offset)
                } else {
                    MappedCpuRead::Openbus
                }
            }

            0xC000..=u16::MAX => {
                let offset = if self.prgrom_size <= 0x4000 {
                    (addr & 0x3FFF) as u32
                } else {
                    (addr & 0x7FFF) as u32
                };

                if offset < self.prgrom_size {
                    MappedCpuRead::PrgRom(offset)
                } else {
                    MappedCpuRead::Openbus
                }
            }
        }
    }

    fn map_cpu_write(&mut self, addr: u16, data: u8) -> MappedCpuWrite {
        match addr {
            0x0000..0x4020 => unreachable!(),
            0x4020..0x6000 => MappedCpuWrite::Other,

            0x6000..0x8000 => {
                let offset = (addr & 0x1FFF) as u32;
                if offset < self.prgram_size {
                    MappedCpuWrite::PrgRam(offset, data)
                } else {
                    MappedCpuWrite::Other
                }
            }

            0x8000..0xC000 => MappedCpuWrite::Other,
            0xC000..=u16::MAX => MappedCpuWrite::Other,
        }
    }

    fn map_ppu_read(&mut self, addr: u16) -> MappedPpuRead {
        match addr {
            0x0000..0x2000 => {
                let offset = (addr & 0x1FFF) as u32;
                if offset < self.chrrom_size {
                    MappedPpuRead::ChrRom(offset)
                } else {
                    MappedPpuRead::Openbus
                }
            }

            0x2000..0x3000 => {
                // TODO: Apply mirroring
                let offset = addr & 0x07FF;
                MappedPpuRead::InternalVram(offset)
            }

            0x3000..=u16::MAX => unreachable!(),
        }
    }

    fn map_ppu_write(&mut self, addr: u16, data: u8) -> MappedPpuWrite {
        match addr {
            0x0000..0x2000 => MappedPpuWrite::Other,

            0x2000..0x3000 => {
                // TODO: Apply mirroring
                let offset = addr & 0x07FF;
                MappedPpuWrite::InternalVram(offset, data)
            }

            0x3000..=u16::MAX => unreachable!(),
        }
    }
}
