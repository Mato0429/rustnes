use crate::nes::mapper::Mirroring;

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
pub struct RomInfo {
    pub mapper_id: u32,
    pub submapper: u8,
    pub hardwired_nt: Mirroring,
    pub alternative_nt: bool,
    pub prgrom: Vec<u8>,
    pub chrrom: Vec<u8>,
    pub prgram_size: u32,
    pub chrram_size: u32,
}

#[derive(Debug, Clone)]
pub struct EmuFile {
    pub rom_info: RomInfo,
    pub env_info: EnvInfo,
}
