use crate::nes::mapper::MapperCtx;

use super::*;
use std::io::{self, Read};

const HEADER_SIZE: usize = 0x10;
const FILE_IDENTIFIER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const NES2_IDENTIFIER: u8 = 0x02;

const TRAINER_SIZE: usize = 0x200;
const PRGROM_CHUNK_SIZE: u32 = 0x4000;
const CHRROM_CHUNK_SIZE: u32 = 0x2000;
const PRGRAM_CHUNK_SIZE: u32 = 0x2000;
const CHRRAM_PLACEHOLDER_SIZE: u32 = 0x2000;

#[derive(Debug, Clone, Copy)]
struct RawHeader {
    pub file_identifier: [u8; 4],
    pub prgrom_size: u8,
    pub chrrom_size: u8,
    pub horizontal_nt: bool,
    pub _has_battery: bool,
    pub has_trainer: bool,
    pub alternative_nt: bool,
    pub mapper_nibble0: u8,
    pub console_type: u8,
    pub mapper_nibble1: u8,
    pub prgram_size: u8,
    pub is_pal_tv: bool,
}

impl RawHeader {
    pub fn new(data: &[u8; HEADER_SIZE]) -> Self {
        Self {
            file_identifier: data[0..4].try_into().unwrap(),
            prgrom_size: data[4],
            chrrom_size: data[5],
            horizontal_nt: data[6] & 0x01 != 0,
            _has_battery: data[6] & 0x02 != 0,
            has_trainer: data[6] & 0x04 != 0,
            alternative_nt: data[6] & 0x08 != 0,
            mapper_nibble0: (data[6] & 0xF0) >> 4,
            console_type: data[7] & 0x03,
            mapper_nibble1: (data[7] & 0xF0) >> 4,
            prgram_size: data[8],
            is_pal_tv: data[9] & 0x01 != 0,
        }
    }
}

pub struct InesLoader;

impl EmuFileLoader for InesLoader {
    fn parse<R: Read>(reader: &mut R) -> io::Result<EmuFile> {
        let mut header_data = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_data)?;
        let rawheader = RawHeader::new(&header_data);

        if rawheader.file_identifier != FILE_IDENTIFIER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the specified file is not .nes format",
            ));
        }

        let mapper_id =
            u16::from_le_bytes([rawheader.mapper_nibble0, rawheader.mapper_nibble1]) as u32;

        let hardwired_nt = if rawheader.horizontal_nt {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        let prgrom_size = PRGROM_CHUNK_SIZE * rawheader.prgrom_size as u32;
        let chrrom_size = CHRROM_CHUNK_SIZE * rawheader.chrrom_size as u32;
        let prgram_size = PRGRAM_CHUNK_SIZE * rawheader.prgram_size as u32;

        let console_type = match rawheader.console_type {
            0 => ConsoleType::NesOrFamicom,
            1 => ConsoleType::VsSystem { ppu: 0, hw: 0 },
            2 => ConsoleType::PlayChoise10,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "invalid console type (Flag7): {:#010b}",
                        rawheader.console_type
                    ),
                ))
            }
        };

        let cpu_ppu_timing = if rawheader.is_pal_tv {
            NesTiming::PalNes
        } else {
            NesTiming::NtscNes
        };

        let mut _trainer = vec![0u8; TRAINER_SIZE];
        if rawheader.has_trainer {
            reader.read_exact(&mut _trainer)?;
        }

        let mut prgrom = vec![0u8; prgrom_size as usize];
        reader.read_exact(&mut prgrom)?;

        let mut chrrom = vec![0u8; chrrom_size as usize];
        reader.read_exact(&mut chrrom)?;

        let mapper_ctx = MapperCtx {
            mapper_id,
            submapper: 0,
            hardwired_nt,
            alternative_nt: rawheader.alternative_nt,
            prgrom,
            chrrom,
            prgram_size,
            chrram_size: CHRRAM_PLACEHOLDER_SIZE,
        };

        let nesrom = mapper_ctx.create();

        let env_info = EnvInfo {
            console_type,
            cpu_ppu_timing,
            other_roms: 0,
            expansion_device: 0,
        };

        Ok(EmuFile { nesrom, env_info })
    }
}
