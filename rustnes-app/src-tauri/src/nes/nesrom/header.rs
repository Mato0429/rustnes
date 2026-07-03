use crate::nes::mapper::{MapperInfo, Mirroring};
use crate::nes::nesrom::RawHeader;
use crate::nes::{ConsoleType, EnvInfo, NesTiming};
use std::io;

const FILE_IDENTIFIER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const NES2_IDENTIFIER: u8 = 0x02;

const PRGROM_CHUNK_SIZE: u32 = 0x4000;
const CHRROM_CHUNK_SIZE: u32 = 0x2000;
const SHIFT_BASE: u32 = 0x40;

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub mapper_info: MapperInfo,
    pub env_info: EnvInfo,
}

impl Header {
    pub fn create(raw: RawHeader) -> io::Result<Self> {
        if raw.file_identifier != FILE_IDENTIFIER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the specified file is not .nes format",
            ));
        }

        /*
        if raw.nes2_identifier != NES2_IDENTIFIER {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "ines format is unsupported",
            ));
        }
        */

        let mapper_id = u32::from_le_bytes([
            raw.mapper_nibble0,
            raw.mapper_nibble1,
            raw.mapper_nibble2,
            0x00,
        ]);

        let hardwired_nt = if raw.horizontal_nt {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        let prgrom_size =
            Self::calculate_rom_size(raw.prgrom_lsb, raw.prgrom_msb, PRGROM_CHUNK_SIZE);

        let chrrom_size =
            Self::calculate_rom_size(raw.chrrom_lsb, raw.chrrom_msb, CHRROM_CHUNK_SIZE);

        let prgram_size = SHIFT_BASE << raw.prgram_sc;
        let chrram_size = SHIFT_BASE << raw.chrram_sc;
        let nvprgram_size = SHIFT_BASE << raw.nvprgram_sc;
        let nvchrram_size = SHIFT_BASE << raw.nvchrram_sc;

        let console_type = match raw.console_type {
            0 => ConsoleType::NesOrFamicom,
            1 => {
                let ppu = raw.console_detail & 0x0F;
                let hw = (raw.console_detail & 0xF0) >> 4;
                ConsoleType::VsSystem { ppu, hw }
            }
            2 => ConsoleType::PlayChoise10,
            3 => {
                let console = raw.console_detail & 0x0F;
                ConsoleType::Extended { console }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid console type (Flag7): {:#010b}", raw.console_type),
                ))
            }
        };

        let cpu_ppu_timing = match raw.cpu_ppu_timing {
            0 => NesTiming::NtscNes,
            1 => NesTiming::PalNes,
            2 => NesTiming::MultiRegion,
            3 => NesTiming::Dendy,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid timing type (Byte12): {:#010b}", raw.cpu_ppu_timing),
                ))
            }
        };

        let header = Self {
            mapper_info: MapperInfo {
                mapper_id,
                submapper: raw.submapper,
                hardwired_nt,
                alternative_nt: raw.alternative_nt,
                has_battery: raw.has_battery,
                has_trainer: raw.has_trainer,
                prgrom_size,
                chrrom_size,
                prgram_size,
                chrram_size,
                nvprgram_size,
                nvchrram_size,
            },
            env_info: EnvInfo {
                console_type,
                cpu_ppu_timing,
                other_roms: raw.other_roms,
                expansion_device: raw.expansion_device,
            },
        };

        Ok(header)
    }

    fn calculate_rom_size(lsb: u8, msb: u8, chunk_size: u32) -> u32 {
        if lsb == 0xFF {
            let exp = ((lsb & 0xFC) >> 2) as u32;
            let m = (lsb & 0x03) as u32;
            2u32.pow(exp) * (m * 2 + 1)
        } else {
            chunk_size * u16::from_le_bytes([lsb, msb]) as u32
        }
    }
}
