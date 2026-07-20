use super::*;
use crate::nes::nescart::mirroring::{Mirroring, VramTarget};

const PRGBANK_SIZE: u32 = 0x2000;
const CHRBANK_SIZE: u32 = 0x400;

#[derive(Debug, Clone, Copy)]
pub struct Mmc3 {
    mirroring: Mirroring,
    is_fourscreen: bool,
    prgram_size: u32,
    prgrom_size: u32,
    chrrom_size: u32,
    chrram_size: u32,

    bankselect: u8,
    prgswap: bool,
    chrswap: bool,
    r: [u8; 8],
    prgram_denywrite: bool,
    prgram_enable: bool,
    irq_latch: u8,
    irq_counter: u8,
    irq_enable: bool,
    require_reload: bool,
    a12_streak: u8,
}

impl Mmc3 {
    pub fn new(args: MapperArgs) -> Self {
        let mirroring = if args.alternative_nt {
            Mirroring::FourScreen
        } else if args.vertical_nt {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        Self {
            mirroring,
            is_fourscreen: args.alternative_nt,
            prgram_size: args.prgram_size,
            prgrom_size: args.prgrom_size,
            chrrom_size: args.chrrom_size,
            chrram_size: args.chrram_size,
            bankselect: 0x00,
            prgswap: false,
            chrswap: false,
            r: [0x00; 8],
            prgram_denywrite: true,
            prgram_enable: false,
            irq_latch: 0x00,
            irq_counter: 0x00,
            irq_enable: false,
            require_reload: false,
            a12_streak: 0,
        }
    }

    fn control_by_cpu(&mut self, addr: u16, data: u8) {
        match (addr & 0x8000 != 0, addr & 0x4000 != 0, addr & 0x0001 != 0) {
            // Bank select
            (false, false, false) => {
                self.bankselect = data & 0x7;
                self.prgswap = data & 0x40 != 0;
                self.chrswap = data & 0x80 != 0;
            }

            // Bank data
            (false, false, true) => self.r[self.bankselect as usize] = data,

            // Nametable arrangement
            (false, true, false) => {
                if !self.is_fourscreen {
                    self.mirroring = if data & 0x01 != 0 {
                        Mirroring::Horizontal
                    } else {
                        Mirroring::Vertical
                    };
                }
            }

            // PrgRam protect
            (false, true, true) => {
                self.prgram_denywrite = data & 0x40 != 0;
                self.prgram_enable = data & 0x80 != 0;
            }

            // IRQ latch
            (true, false, false) => self.irq_latch = data,

            // IRQ reload
            (true, false, true) => {
                self.irq_counter = 0;
                self.require_reload = true
            }

            // IRQ disable
            (true, true, false) => self.irq_enable = false,

            // IRQ enable
            (true, true, true) => self.irq_enable = true,
        };
    }

    fn observe_ppu(&mut self, addr: u16) {
        let a12_active = addr & 0x1000 != 0;

        // Valid edge
        if 3 <= self.a12_streak && a12_active {
            if self.irq_counter == 0 || self.require_reload {
                self.irq_counter = self.irq_latch;
                self.require_reload = false;
            } else {
                self.irq_counter = self.irq_counter.wrapping_sub(1);
            }
        }

        if a12_active {
            self.a12_streak = 0;
        } else {
            self.a12_streak = self.a12_streak.saturating_add(1);
        }
    }

    fn prg_addr(&self, addr: u16) -> u32 {
        let banks = (self.prgrom_size / PRGBANK_SIZE) as u8;

        let bank = if !self.prgswap {
            match addr {
                0x8000..=0x9FFF => self.r[6],
                0xA000..=0xBFFF => self.r[7],
                0xC000..=0xDFFF => banks - 2,
                0xE000..=0xFFFF => banks - 1,
                _ => unreachable!(),
            }
        } else {
            match addr {
                0x8000..=0x9FFF => self.r[7],
                0xA000..=0xBFFF => banks - 2,
                0xC000..=0xDFFF => self.r[6],
                0xE000..=0xFFFF => banks - 1,
                _ => unreachable!(),
            }
        };

        bank as u32 * PRGBANK_SIZE + (addr as u32 & 0x1FFF)
    }

    fn chr_addr(&self, addr: u16) -> u32 {
        let bank = if !self.chrswap {
            match addr {
                0x0000..=0x07FF => self.r[0] & 0xFE,
                0x0800..=0x0FFF => self.r[1] & 0xFE,
                0x1000..=0x13FF => self.r[2],
                0x1400..=0x17FF => self.r[3],
                0x1800..=0x1BFF => self.r[4],
                0x1C00..=0x1FFF => self.r[5],
                _ => unreachable!(),
            }
        } else {
            match addr {
                0x0000..=0x07FF => self.r[2],
                0x0800..=0x0FFF => self.r[3],
                0x1000..=0x13FF => self.r[4],
                0x1400..=0x17FF => self.r[5],
                0x1800..=0x1BFF => self.r[0] & 0xFE,
                0x1C00..=0x1FFF => self.r[1] & 0xFE,
                _ => unreachable!(),
            }
        };

        let is_2kb = !self.chrswap && (0x0000..=0x0FFF).contains(&addr)
            || self.chrswap && (0x1800..=0x1FFF).contains(&addr);

        if is_2kb {
            bank as u32 * CHRBANK_SIZE + (addr as u32 & 0x7FF)
        } else {
            bank as u32 * CHRBANK_SIZE + (addr as u32 & 0x3FF)
        }
    }
}

impl MapperLogic for Mmc3 {
    fn irq_active(&self) -> bool {
        self.irq_counter == 0 && self.irq_enable
    }

    fn cpu_step(&mut self) {}

    fn map_cpu_read(&mut self, addr: u16) -> MappedCpuRead {
        match addr {
            // Unmapped
            0x4020..=0x5FFF => MappedCpuRead::Openbus,

            // PrgRam
            0x6000..=0x7FFF => {
                if self.prgram_enable {
                    let offset = (addr & 0x1FFF) as u32;
                    if offset < self.prgram_size {
                        return MappedCpuRead::PrgRam(offset);
                    }
                }

                MappedCpuRead::Openbus
            }

            // MMC3 Controls
            // 0x8000 | 0x8001 | 0xA000 | 0xA001 | 0xE000 | 0xE001 => MappedCpuRead::Openbus,

            // PrgRom
            0x8000..=u16::MAX => {
                let offset = self.prg_addr(addr);
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
                if !self.prgram_denywrite {
                    let offset = (addr & 0x1FFF) as u32;
                    if offset < self.prgram_size {
                        return MappedCpuWrite::PrgRam(offset, data);
                    }
                }

                MappedCpuWrite::Other
            }

            0x8000..=u16::MAX => {
                self.control_by_cpu(addr, data);
                MappedCpuWrite::Other
            }

            _ => panic!("address out of range: {addr:#06X}"),
        }
    }

    fn map_ppu_read(&mut self, addr: u16) -> MappedPpuRead {
        self.observe_ppu(addr);

        match addr {
            // ChrRom/Ram
            0x0000..=0x1FFF => {
                let offset = self.chr_addr(addr);
                if self.chrrom_size == 0 {
                    MappedPpuRead::ChrRam(offset % self.chrram_size)
                } else {
                    MappedPpuRead::ChrRom(offset % self.chrrom_size)
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
        self.observe_ppu(addr);

        match addr {
            // ChrRom/Ram
            0x0000..=0x1FFF => {
                if self.chrram_size != 0 {
                    let offset = self.chr_addr(addr);
                    MappedPpuWrite::ChrRam(offset % self.chrram_size, data)
                } else {
                    MappedPpuWrite::Other
                }
            }

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
