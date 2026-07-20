mod logic;

mod empty;
mod mmc1;
mod mmc3;
mod nrom;

use anyhow::{anyhow, Result};
use enum_dispatch::enum_dispatch;
pub(super) use logic::*;

use empty::Empty;
use mmc1::Mmc1;
use mmc3::Mmc3;
use nrom::Nrom;

#[derive(Debug, Clone)]
pub struct MapperArgs {
    pub vertical_nt: bool,
    pub alternative_nt: bool,
    pub prgrom_size: u32,
    pub chrrom_size: u32,
    pub prgram_size: u32,
    pub chrram_size: u32,
}

#[enum_dispatch(MapperLogic)]
#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    Empty,
    Nrom,
    Mmc1,
    Mmc3,
}

impl Mapper {
    pub fn empty() -> Self {
        Self::Empty(Empty)
    }

    pub fn new(mapper_id: u32, submapper: u8, args: MapperArgs) -> Result<Self> {
        let mapper = match (mapper_id, submapper) {
            (0, _) => Mapper::Nrom(Nrom::new(args)),
            (1, _) => Mapper::Mmc1(Mmc1::new(args)),
            (4, _) => Mapper::Mmc3(Mmc3::new(args)),

            _ => {
                return Err(anyhow!(
                    "unsupported mapper: id={mapper_id}, submapper={submapper}"
                ))
            }
        };

        Ok(mapper)
    }
}
