mod commands;
mod engine;
mod models;
mod pgn;

use crate::commands::analyze_game;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![analyze_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
