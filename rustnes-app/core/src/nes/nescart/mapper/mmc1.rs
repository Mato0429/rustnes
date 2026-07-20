use super::*;
use crate::nes::nescart::mirroring::{Mirroring, VramTarget};

const PRGBANK_SIZE: u32 = 0x4000;
const CHRBANK_SIZE: u32 = 0x1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrgBankMode {
    Flexed,
    FirstAndFlexed,
    FlexedAndLast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChrBankMode {
    Combined,
    Splitted,
}

#[derive(Debug, Clone, Copy)]
pub struct Mmc1 {
    mirroring: Mirroring,
    prgram_size: u32,
    prgrom_size: u32,
    chrrom_size: u32,
    chrram_size: u32,

    load_seek: u8,
    load_buf: u8,
    chrbank0: u8,
    chrbank1: u8,
    prgbank: u8,
    write_lock: bool,
    prgbank_mode: PrgBankMode,
    chrbank_mode: ChrBankMode,
}

impl Mmc1 {
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
            chrram_size: args.chrram_size,
            load_seek: 0x0,
            load_buf: 0x00,
            chrbank0: 0x00,
            chrbank1: 0x00,
            prgbank: 0x00,
            write_lock: false,
            prgbank_mode: PrgBankMode::Flexed,
            chrbank_mode: ChrBankMode::Combined,
        }
    }

    fn prgrom_addr(&self, addr: u16) -> u32 {
        let banks = (self.prgrom_size / PRGBANK_SIZE) as u8;
        let bank = match self.prgbank_mode {
            PrgBankMode::Flexed => self.prgbank & 0xFE,
            PrgBankMode::FirstAndFlexed => {
                if addr < 0xC000 {
                    0
                } else {
                    self.prgbank
                }
            }
            PrgBankMode::FlexedAndLast => {
                if addr < 0xC000 {
                    self.prgbank
                } else {
                    banks - 1
                }
            }
        };

        bank as u32 * PRGBANK_SIZE + (addr as u32 & 0x7FFF)
    }

    fn chr_addr(&self, addr: u16) -> u32 {
        match self.chrbank_mode {
            ChrBankMode::Combined => {
                let bank = (self.chrbank0 & 0xFE) as u32;
                bank * CHRBANK_SIZE + (addr as u32 & 0x1FFF)
            }
            ChrBankMode::Splitted => {
                let bank = if addr < 0x1000 {
                    self.chrbank0
                } else {
                    self.chrbank1
                };
                bank as u32 * CHRBANK_SIZE + (addr as u32 & 0x0FFF)
            }
        }
    }

    fn initiate(&mut self) {
        self.load_buf = 0x00;
        self.load_seek = 0;
    }

    fn write_control(&mut self, data: u8) {
        self.mirroring = match data & 0x3 {
            0 => Mirroring::SingleScreen0,
            1 => Mirroring::SingleScreen1,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        };

        self.prgbank_mode = match (data & 0x0C) >> 2 {
            0 | 1 => PrgBankMode::Flexed,
            2 => PrgBankMode::FirstAndFlexed,
            3 => PrgBankMode::FlexedAndLast,
            _ => unreachable!(),
        };

        self.chrbank_mode = if data & 0x10 != 0 {
            ChrBankMode::Splitted
        } else {
            ChrBankMode::Combined
        };
    }

    fn serial_write(&mut self, addr: u16, data: u8) {
        if data & 0x80 != 0 {
            self.prgbank_mode = PrgBankMode::FlexedAndLast;
            self.write_lock = false;
            self.initiate();
            return;
        }

        if self.write_lock {
            return;
        }
        self.write_lock = true;

        self.load_buf |= (data & 0x01) << self.load_seek;
        self.load_seek += 1;

        if self.load_seek == 5 {
            match addr {
                0x8000..=0x9FFF => self.write_control(self.load_buf),
                0xA000..=0xBFFF => self.chrbank0 = self.load_buf,
                0xC000..=0xDFFF => self.chrbank1 = self.load_buf,
                0xE000..=u16::MAX => self.prgbank = self.load_buf,
                _ => unreachable!(),
            }

            self.initiate();
        }
    }
}

impl MapperLogic for Mmc1 {
    fn irq_active(&self) -> bool {
        false
    }

    fn cpu_step(&mut self) {
        self.write_lock = false;
    }

    fn map_cpu_read(&mut self, addr: u16) -> MappedCpuRead {
        match addr {
            // Unmapped
            0x4020..=0x5FFF => MappedCpuRead::Openbus,

            // PrgRam
            0x6000..=0x7FFF => {
                let offset = (addr & 0x1FFF) as u32;
                MappedCpuRead::PrgRam(offset % self.prgram_size)
            }

            // PrgRom
            0x8000..=u16::MAX => {
                let offset = self.prgrom_addr(addr) % self.prgrom_size;
                MappedCpuRead::PrgRom(offset)
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
                MappedCpuWrite::PrgRam(offset % self.prgram_size, data)
            }

            // PrgRom
            0x8000..=u16::MAX => {
                self.serial_write(addr, data);
                MappedCpuWrite::Other
            }

            _ => panic!("address out of range: {addr:#06X}"),
        }
    }

    fn map_ppu_read(&mut self, addr: u16) -> MappedPpuRead {
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
