use crate::nes::nesrom::HEADER_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct RawHeader {
    pub file_identifier: [u8; 4],
    pub prgrom_lsb: u8,
    pub chrrom_lsb: u8,
    pub horizontal_nt: bool,
    pub has_battery: bool,
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
    pub nvprgram_sc: u8,
    pub chrram_sc: u8,
    pub nvchrram_sc: u8,
    pub cpu_ppu_timing: u8,
    pub console_detail: u8,
    pub other_roms: u8,
    pub expansion_device: u8,
}

impl RawHeader {
    pub fn create(data: &[u8; HEADER_SIZE]) -> Self {
        Self {
            file_identifier: data[0..4].try_into().unwrap(),
            prgrom_lsb: data[4],
            chrrom_lsb: data[5],
            horizontal_nt: data[6] & 0x01 != 0,
            has_battery: data[6] & 0x02 != 0,
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
            nvprgram_sc: (data[10] & 0xF0) >> 4,
            chrram_sc: data[11] & 0x0F,
            nvchrram_sc: (data[11] & 0xF0) >> 4,
            cpu_ppu_timing: data[12] & 0x03,
            console_detail: data[13],
            other_roms: data[14],
            expansion_device: data[15],
        }
    }
}
