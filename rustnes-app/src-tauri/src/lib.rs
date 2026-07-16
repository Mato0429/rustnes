mod commands;
mod emulator;

use commands::{get_frame, load_cartridge, AppState};
use emulator::{spawn_emulator_thread, FrameBuffer};
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let fb = Arc::new(FrameBuffer::new(256, 240));
    let handle = spawn_emulator_thread(Arc::clone(&fb));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            fb: Arc::clone(&fb),
            emu_handle: handle,
        })
        .invoke_handler(tauri::generate_handler![get_frame, load_cartridge])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
