mod header;
mod rawheader;

use crate::nes::{Mapper, MapperFactory};
use header::Header;
use rawheader::RawHeader;
use std::io::{self, BufReader, Read};

const HEADER_SIZE: usize = 0x10;
const TRAINER_SIZE: usize = 0x200;

#[derive(Debug)]
pub struct NesRom {
    pub header: Header,
    pub mapper: Box<dyn Mapper>,
    pub trainer: [u8; TRAINER_SIZE],
    pub prgrom: Vec<u8>,
    pub chrrom: Vec<u8>,
    pub prgram: Vec<u8>,
    pub chrram: Vec<u8>,
    // pub nvprgram: Vec<u8>,
    // pub nvchrram: Vec<u8>,
}

impl NesRom {
    pub fn create<R: Read>(data: R) -> io::Result<Self> {
        let mut buf = BufReader::new(data);

        // Read header info
        let mut header_data = [0u8; HEADER_SIZE];
        buf.read_exact(&mut header_data)?;
        let raw_header = RawHeader::create(&header_data);
        let header = Header::create(raw_header)?;

        // Construct a mapper
        let mapper = MapperFactory::create(header.mapper_info).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                format!("unsupported mapper: {:}", header.mapper_info.mapper_id),
            )
        })?;

        // Allocate Roms/Rams
        let mut trainer = [0u8; TRAINER_SIZE];
        if header.mapper_info.has_trainer {
            buf.read_exact(&mut trainer)?;
        }

        let mut prgrom = vec![0u8; header.mapper_info.prgrom_size as usize];
        buf.read_exact(&mut prgrom)?;
        let mut chrrom = vec![0u8; header.mapper_info.chrrom_size as usize];
        buf.read_exact(&mut chrrom)?;

        let prgram = vec![0u8; header.mapper_info.prgram_size as usize];
        let chrram = vec![0u8; header.mapper_info.chrram_size as usize];

        let cart = NesRom {
            header,
            mapper,
            trainer,
            prgrom,
            chrrom,
            prgram,
            chrram,
        };

        Ok(cart)
    }
}
