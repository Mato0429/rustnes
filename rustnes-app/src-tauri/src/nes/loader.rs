mod ines;
mod nes2;

use crate::nes::mapper::{Mapper, Mirroring};
use std::io::{self, Read, Seek, SeekFrom};

pub use ines::InesLoader;
pub use nes2::Nes2Loader;

pub type NesRom = Mapper;

// TODO: Support console details
#[derive(Debug, Clone, Copy)]
pub enum ConsoleType {
    NesOrFamicom,
    VsSystem { ppu: u8, hw: u8 },
    PlayChoise10,
    Extended { console: u8 },
}

#[derive(Debug, Clone, Copy)]
pub enum NesTiming {
    NtscNes,
    PalNes,
    MultiRegion,
    Dendy,
}

#[derive(Debug, Clone, Copy)]
pub struct EnvInfo {
    pub console_type: ConsoleType,
    pub cpu_ppu_timing: NesTiming,
    pub other_roms: u8,
    pub expansion_device: u8, // TODO: Support expansion devices
}

#[derive(Debug, Clone)]
pub struct EmuFile {
    pub nesrom: NesRom,
    pub env_info: EnvInfo,
}

pub trait EmuFileLoader {
    fn parse<R: Read>(reader: &mut R) -> io::Result<EmuFile>;
}

pub fn parse_emufile<R: Read + Seek>(mut reader: R) -> io::Result<EmuFile> {
    let head = reader.stream_position()?;

    Nes2Loader::parse(&mut reader).or_else(|_| {
        reader.seek(SeekFrom::Start(head))?;
        InesLoader::parse(&mut reader)
    })
}
