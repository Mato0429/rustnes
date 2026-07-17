mod commands;
mod emulator;

use commands::{get_frame, load_cart, update_joypad1, AppState};
use emulator::spawn_emulator_thread;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let emu_handle = spawn_emulator_thread();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { emu_handle })
        .invoke_handler(tauri::generate_handler![
            get_frame,
            load_cart,
            update_joypad1
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
