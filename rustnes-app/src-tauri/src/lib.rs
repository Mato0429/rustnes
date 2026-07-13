mod commands;
mod emulator;

use commands::{get_frame, get_screen_info, EmuState};
use emulator::{spawn_emulator_thread, FrameBuffer};
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let fb = Arc::new(FrameBuffer::new(256, 240));
    spawn_emulator_thread(fb.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(EmuState { fb: fb.clone() })
        .invoke_handler(tauri::generate_handler![get_screen_info, get_frame])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
