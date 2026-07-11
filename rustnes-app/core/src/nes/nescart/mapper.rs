mod logic;
mod nrom;

use anyhow::{anyhow, Result};
use enum_dispatch::enum_dispatch;
pub(super) use logic::*;

use nrom::Nrom;

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen0,
    SingleScreen1,
    FourScreen,
}

#[derive(Debug, Clone)]
pub struct MapperArgs {
    pub horizontal_nt: bool,
    pub alternative_nt: bool,
    pub prgrom_size: u32,
    pub chrrom_size: u32,
    pub prgram_size: u32,
    pub chrram_size: u32,
}

#[enum_dispatch(MapperLogic)]
#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    Nrom,
}

impl Mapper {
    pub fn new(mapper_id: u32, submapper: u8, args: MapperArgs) -> Result<Self> {
        let mapper = match (mapper_id, submapper) {
            (0, _) => Mapper::Nrom(Nrom::new(args)),

            _ => {
                return Err(anyhow!(
                    "unsupported mapper: id={mapper_id}, submapper={submapper}"
                ))
            }
        };

        Ok(mapper)
    }
}
