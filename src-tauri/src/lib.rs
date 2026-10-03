mod commands;
mod data;
mod heuristics;
mod models;
mod pipeline;
mod resources;
mod uci;

use crate::commands::{
    analyze_game, analyze_live_position,
    cancel_analysis, configure_engine,
    fetch_games, get_engine_config,
    get_player_profile, parse_pgn,
    stop_live_analysis, toggle_live_engine,
};
use crate::data::book::OpeningBook;
use crate::models::engine_config::EngineConfig;
use crate::resources::store::ResourceStore;
use crate::uci::live_manager::init_live_manager;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub store: Arc<ResourceStore>,
    pub opening_book: Arc<OpeningBook>,
    pub cancel_analysis_flag: Arc<AtomicBool>,
    /// Single source of truth for engine settings.
    /// Written by `configure_engine`, read by the analysis pipeline
    /// and by `init_live_manager` whenever the engine is (re)started.
    pub engine_config: Arc<Mutex<EngineConfig>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let store = Arc::new(ResourceStore::open(app.handle())?);
            #[cfg(debug_assertions)]
            store.seed_dev_resources();

            let book_path = store
                .active_book_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let opening_book = Arc::new(OpeningBook::new(&book_path));

            let engine_config = Arc::new(Mutex::new(
                EngineConfig::default(),
            ));

            app.manage(AppState {
                store,
                opening_book,
                cancel_analysis_flag: Arc::new(
                    AtomicBool::new(false),
                ),
                engine_config: engine_config.clone(),
            });

            // Pass the shared config so the live manager always starts
            // the engine with whatever settings the user last saved.
            let manager = init_live_manager(
                app.handle().clone(),
                engine_config,
            );
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
            cancel_analysis,
            configure_engine,
            get_engine_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
