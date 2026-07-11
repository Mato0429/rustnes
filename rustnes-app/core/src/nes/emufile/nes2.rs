use super::*;
use std::io::{self, Read};

const HEADER_SIZE: usize = 0x10;
const FILE_IDENTIFIER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const NES2_IDENTIFIER: u8 = 0x02;

const TRAINER_SIZE: usize = 0x200;
const PRGROM_CHUNK_SIZE: u32 = 0x4000;
const CHRROM_CHUNK_SIZE: u32 = 0x2000;
const SHIFT_BASE: u32 = 0x40;

#[derive(Debug, Clone, Copy)]
struct RawHeader {
    pub file_identifier: [u8; 4],
    pub prgrom_lsb: u8,
    pub chrrom_lsb: u8,
    pub horizontal_nt: bool,
    pub _has_battery: bool,
    pub has_trainer: bool,
    pub alternative_nt: bool,
    pub mapper_nibble0: u8,
    pub console_type: u8,
    pub nes2_identifier: u8,
    pub mapper_nibble1: u8,
    pub mapper_nibble2: u8,
    pub submapper: u8,
    pub prgrom_msb: u8,
    pub chrrom_msb: u8,
    pub prgram_sc: u8,
    pub _nvprgram_sc: u8,
    pub chrram_sc: u8,
    pub _nvchrram_sc: u8,
    pub nes_region: u8,
    pub console_detail: u8,
    pub other_roms: u8,
    pub expansion_device: u8,
}

impl RawHeader {
    pub fn new(data: &[u8; HEADER_SIZE]) -> Self {
        Self {
            file_identifier: data[0..4].try_into().unwrap(),
            prgrom_lsb: data[4],
            chrrom_lsb: data[5],
            horizontal_nt: data[6] & 0x01 != 0,
            _has_battery: data[6] & 0x02 != 0,
            has_trainer: data[6] & 0x04 != 0,
            alternative_nt: data[6] & 0x08 != 0,
            mapper_nibble0: (data[6] & 0xF0) >> 4,
            console_type: data[7] & 0x03,
            nes2_identifier: (data[7] & 0x0C) >> 2,
            mapper_nibble1: (data[7] & 0xF0) >> 4,
            mapper_nibble2: data[8] & 0x0F,
            submapper: (data[8] & 0xF0) >> 4,
            prgrom_msb: data[9] & 0x0F,
            chrrom_msb: (data[9] & 0xF0) >> 4,
            prgram_sc: data[10] & 0x0F,
            _nvprgram_sc: (data[10] & 0xF0) >> 4,
            chrram_sc: data[11] & 0x0F,
            _nvchrram_sc: (data[11] & 0xF0) >> 4,
            nes_region: data[12] & 0x03,
            console_detail: data[13],
            other_roms: data[14],
            expansion_device: data[15],
        }
    }
}

pub struct Nes2Parser;

impl Nes2Parser {
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

impl EmuFileParser for Nes2Parser {
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

        if rawheader.nes2_identifier != NES2_IDENTIFIER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported format for this parser (Nes2Parser)",
            ));
        }

        let mapper_id = u32::from_le_bytes([
            rawheader.mapper_nibble0,
            rawheader.mapper_nibble1,
            rawheader.mapper_nibble2,
            0x00,
        ]);

        let hardwired_nt = if rawheader.horizontal_nt {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        let prgrom_size = Self::calculate_rom_size(
            rawheader.prgrom_lsb,
            rawheader.prgrom_msb,
            PRGROM_CHUNK_SIZE,
        );

        let chrrom_size = Self::calculate_rom_size(
            rawheader.chrrom_lsb,
            rawheader.chrrom_msb,
            CHRROM_CHUNK_SIZE,
        );

        let prgram_size = SHIFT_BASE << rawheader.prgram_sc;
        let chrram_size = SHIFT_BASE << rawheader.chrram_sc;

        let console_type = match rawheader.console_type {
            0 => ConsoleType::NesOrFamicom,
            1 => {
                let ppu = rawheader.console_detail & 0x0F;
                let hw = (rawheader.console_detail & 0xF0) >> 4;
                ConsoleType::VsSystem { ppu, hw }
            }
            2 => ConsoleType::PlayChoise10,
            3 => {
                let console = rawheader.console_detail & 0x0F;
                ConsoleType::Extended { console }
            }
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

        let nes_region = match rawheader.nes_region {
            0 => NesRegion::Ntsc,
            1 => NesRegion::Pal,
            2 => NesRegion::Multi,
            3 => NesRegion::Dendy,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "invalid region type (Byte12): {:#010b}",
                        rawheader.nes_region
                    ),
                ))
            }
        };

        let mut _trainer = vec![0u8; TRAINER_SIZE];
        if rawheader.has_trainer {
            reader.read_exact(&mut _trainer)?;
        }

        let mut prgrom = vec![0u8; prgrom_size as usize];
        reader.read_exact(&mut prgrom)?;

        let mut chrrom = vec![0u8; chrrom_size as usize];
        reader.read_exact(&mut chrrom)?;

        let nesrom_info = NesRomInfo {
            mapper_id,
            submapper: rawheader.submapper,
            hardwired_nt,
            alternative_nt: rawheader.alternative_nt,
            prgrom,
            chrrom,
            prgram_size,
            chrram_size,
        };

        let emufile = EmuFile {
            console_type,
            nes_region,
            other_roms: 0,
            expansion_device: 0,
            nesrom_info,
        };

        Ok(emufile)
    }
}
