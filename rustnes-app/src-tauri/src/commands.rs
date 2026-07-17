use std::sync::atomic::Ordering;

use crate::emulator::{EmuCommand, EmuHandle};

use rustnes_core::nes::nescart::NesCart;
use tauri::{ipc::Response, State};

pub struct AppState {
    pub emu_handle: EmuHandle,
}

#[tauri::command]
pub async fn update_joypad1(state: State<'_, AppState>, v: u8) -> Result<(), String> {
    state.emu_handle.jp1.store(v, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn get_frame(state: State<'_, AppState>) -> Result<Response, String> {
    Ok(Response::new(state.emu_handle.fb.lock().unwrap().clone()))
}

#[tauri::command]
pub async fn load_cart(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let cart = NesCart::open(path).map_err(|v| v.to_string())?;

    state
        .emu_handle
        .cmd_tx
        .send(EmuCommand::Load(cart))
        .map_err(|e| e.to_string())?;

    Ok(())
}
