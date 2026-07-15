mod ines;
mod nes2;

use std::io::{self, Read, Seek, SeekFrom};

pub use ines::InesParser;
pub use nes2::Nes2Parser;

// TODO: Support console details
#[derive(Debug, Clone, Copy)]
pub enum ConsoleType {
    NesOrFamicom,
    VsSystem { ppu: u8, hw: u8 },
    PlayChoise10,
    Extended { console: u8 },
}

#[derive(Debug, Clone, Copy)]
pub enum NesRegion {
    Ntsc,
    Pal,
    Multi,
    Dendy,
}

#[derive(Debug, Clone)]
pub struct EmuFile {
    pub console_type: ConsoleType,
    pub nes_region: NesRegion,
    pub other_roms: u8,
    pub expansion_device: u8,

    pub mapper_id: u32,
    pub submapper: u8,
    pub vertical_nt: bool,
    pub alternative_nt: bool,
    pub prgrom: Vec<u8>,
    pub chrrom: Vec<u8>,
    pub prgram_size: u32,
    pub chrram_size: u32,
}

pub trait EmuFileParser {
    fn parse<R: Read>(reader: &mut R) -> io::Result<EmuFile>;
}

pub fn parse_emufile<R: Read + Seek>(mut reader: R) -> io::Result<EmuFile> {
    let head = reader.stream_position()?;

    Nes2Parser::parse(&mut reader).or_else(|_| {
        reader.seek(SeekFrom::Start(head))?;
        InesParser::parse(&mut reader)
    })
}
