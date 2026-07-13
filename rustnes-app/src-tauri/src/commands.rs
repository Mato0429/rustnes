use super::emulator::FrameBuffer;
use serde::Serialize;
use std::sync::Arc;
use tauri::{ipc::Response, State};

pub struct EmuState {
    pub fb: Arc<FrameBuffer>,
}

#[derive(Serialize)]
pub struct ScreenInfo {
    width: u32,
    height: u32,
}

#[tauri::command]
pub fn get_screen_info(state: State<EmuState>) -> ScreenInfo {
    ScreenInfo {
        width: state.fb.width,
        height: state.fb.height,
    }
}

#[tauri::command]
pub async fn get_frame(state: State<'_, EmuState>) -> Result<Response, String> {
    let frame = state.fb.snapshot();
    Ok(Response::new(frame.as_ref().clone()))
}
