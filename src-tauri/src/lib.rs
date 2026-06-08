mod commands;
mod data;
mod heuristics;
mod models;
mod pipeline;
mod uci;

use crate::commands::{
    analyze_game, analyze_live_position,
    fetch_games, get_player_profile, parse_pgn,
    stop_live_analysis, toggle_live_engine,
};
use crate::data::book::OpeningBook;
use crate::uci::live_manager::init_live_manager;
use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub engine_path: String,
    pub opening_book: Arc<OpeningBook>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let current_dir = std::env::current_dir().unwrap();

            let engine_path = current_dir
                .join("core/engine/stockfish-ubuntu-x86-64-bmi2")
                .to_string_lossy()
                .to_string();

            let book_path = current_dir
                .join("core/database/book.bin")
                .to_string_lossy()
                .to_string();

            let opening_book = Arc::new(OpeningBook::new(&book_path));

            app.manage(AppState {
                engine_path,
                opening_book,
            });

            let manager = init_live_manager(app.handle().clone());
            app.manage(manager);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            analyze_game,
            fetch_games,
            parse_pgn,
            get_player_profile,
            toggle_live_engine,
            stop_live_analysis,
            analyze_live_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
