mod ines;
mod nes2;

use crate::nes::emufile::{ConsoleType, EmuFile, EnvInfo, NesTiming, RomInfo};
use crate::nes::mapper::Mirroring;
use std::io::{self, BufReader, Read};

pub use ines::InesParser;
pub use nes2::Nes2Parser;

pub trait EmuFileParser {
    fn parse<R: Read>(data: R) -> io::Result<EmuFile>;
}

pub fn parse_emufile<R: Read>(data: R) -> io::Result<EmuFile> {
    let mut buf = Vec::<u8>::new();
    let mut reader = BufReader::new(data);
    reader.read_to_end(&mut buf)?;
    let buf_slice = buf.as_slice();

    Nes2Parser::parse(buf_slice).or_else(|_| InesParser::parse(buf_slice))
}
