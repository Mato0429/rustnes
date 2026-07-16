use std::{fs::File, io::BufReader, sync::Arc};

use crate::emulator::{EmuCommand, EmuHandle};

use super::emulator::FrameBuffer;
use rustnes_core::nes::{
    emufile::parse_emufile,
    nescart::{
        mapper::{Mapper, MapperArgs},
        NesCart,
    },
};
use tauri::{ipc::Response, State};

pub struct AppState {
    pub emu_handle: EmuHandle,
    pub fb: Arc<FrameBuffer>,
}

#[tauri::command]
pub async fn get_frame(state: State<'_, AppState>) -> Result<Response, String> {
    let frame = state.fb.snapshot();
    Ok(Response::new(frame.as_ref().clone()))
}

#[tauri::command]
pub async fn load_cartridge(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let emufile = parse_emufile(BufReader::new(file)).map_err(|e| e.to_string())?;

    let args = MapperArgs {
        vertical_nt: emufile.vertical_nt,
        alternative_nt: emufile.alternative_nt,
        chrram_size: emufile.chrram_size,
        chrrom_size: emufile.chrrom.len() as u32,
        prgram_size: emufile.prgram_size,
        prgrom_size: emufile.prgrom.len() as u32,
    };

    let cart = NesCart {
        mapper: Mapper::new(emufile.mapper_id, emufile.submapper, args)
            .map_err(|e| e.to_string())?,
        chrram: vec![0u8; emufile.chrram_size as usize],
        chrrom: emufile.chrrom,
        prgram: vec![0u8; emufile.prgram_size as usize],
        prgrom: emufile.prgrom,
    };

    state
        .emu_handle
        .cmd_tx
        .send(EmuCommand::LoadCart(cart))
        .map_err(|e| e.to_string())?;

    Ok(())
}
