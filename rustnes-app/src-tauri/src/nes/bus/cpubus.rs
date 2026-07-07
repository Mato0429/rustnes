use crate::nes::{bus::BusData, mapper::MapperLogic, NesRom, Ppu};

pub const WRAM_SIZE: usize = 0x800;

/// # Mapping
/// | Address          | Size     | Device            | Description       |
/// |:-----------------|:---------|:------------------|:------------------|
/// | `0x0000..0x0800` | `0x0800` | WRAM              |                   |
/// | `0x0800..0x2000` |          | WRAM(Mirror)      | `0x0000..0x0800`  |
/// | `0x2000..0x2008` | `0x0008` | PPUReg            |                   |
/// | `0x2008..0x4000` |          | PPUReg(Mirror)    | `0x2000..0x2008`  |
/// | `0x4000..0x4018` | `0x0018` | APU, DMA, I/O Reg |                   |
/// | `0x4018..0x401B` | `0x0003` | CPU Test Mode     | [^1] Disabled     |
/// | `0x401B..0x4020` | `0x0005` | Interval Timer    | [^2] Disconnected |
/// | `0x4020..`       | `0xBFE0` | Cartridge(Mapper) |                   |
///
/// [^1]: [NesDev - CPU Test Mode](https://www.nesdev.org/wiki/CPU_Test_Mode)
/// [^2]: [NesDev - RP2A03 Programmable Interval Timer](https://www.nesdev.org/wiki/RP2A03_Programmable_Interval_Timer)
#[derive(Debug)]
pub struct CpuBus<'a> {
    pub openbus: &'a mut u8,
    pub wram: &'a mut [u8; WRAM_SIZE],
    pub ppu: &'a mut Ppu,
    pub nesrom: &'a mut NesRom,
}

impl<'a> CpuBus<'a> {
    pub fn read(&mut self, addr: u16) -> u8 {
        let busdata = match addr {
            0x0000..0x2000 => BusData::new(self.wram[(addr & 0x7FF) as usize], 0xFF),
            0x2000..0x4000 => self.ppu.read_register((addr & 0x07) as u8),
            0x4000..0x4020 => BusData::new(0, 0x00), // TODO: APU and others
            0x4020..=u16::MAX => self.nesrom.cpu_read(addr),
        };

        *self.openbus = busdata.composite(*self.openbus);
        *self.openbus
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        *self.openbus = data;

        match addr {
            0x0000..0x2000 => self.wram[(addr & 0x7FF) as usize] = data,
            0x2000..0x4000 => self.ppu.write_register((addr & 0x07) as u8, data),
            0x4000..0x4020 => (), // TODO: APU and others
            0x4020..=u16::MAX => self.nesrom.cpu_write(addr, data),
        }
    }
}
