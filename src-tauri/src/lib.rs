mod commands;
mod data;
mod heuristics;
mod models;
mod pipeline;
mod uci;

use crate::commands::{
    analyze_game, fetch_games, parse_pgn,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![analyze_game, fetch_games, parse_pgn])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
