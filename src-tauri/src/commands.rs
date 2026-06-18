use crate::data::fetcher::{
    fetch_chesscom_games, fetch_chesscom_profile,
    fetch_lichess_games, fetch_lichess_profile,
};
use crate::models::fetch::{
    ChessComCursor, FetchResult, PlayerProfile,
};
use crate::pipeline::analyzer::run_analysis_pipeline;

use crate::data::pgn::PgnVisitor;
#[allow(unused_imports)]
use crate::models::game::{
    AnalysisProgress, AnalysisSummary,
    AnalyzedMove, GameMetadata, MoveBadge,
    MoveCounts,
};
use crate::uci::live_manager::{
    LiveCommand, LiveEngineManager,
};

use crate::AppState;
use pgn_reader::Reader;
use std::io::Cursor;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn analyze_game(
    app: AppHandle,
    pgn: String,
    target_time_ms: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // The search time for the UCI chess engine
    let time_ms = target_time_ms.unwrap_or(1500);

    // Clone the paths/arcs for the background thread
    let engine_path = state.engine_path.clone();
    let book = state.opening_book.clone();

    // Reset the flag to false before starting a new analysis
    state
        .cancel_analysis_flag
        .store(false, Ordering::Relaxed);

    // Clone the flag to pass to the detached thread
    let cancel_flag =
        state.cancel_analysis_flag.clone();

    std::thread::spawn(move || {
        if let Err(e) = run_analysis_pipeline(
            app.clone(),
            pgn,
            time_ms,
            engine_path,
            book,
            cancel_flag,
        ) {
            let _ =
                app.emit("analysis-error", &e);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn fetch_games(
    username: String,
    platform: String,
    cursor: Option<ChessComCursor>,
) -> Result<FetchResult, String> {
    match platform.as_str() {
        "chesscom" => {
            fetch_chesscom_games(
                &username, cursor,
            )
            .await
        }
        "lichess" => {
            fetch_lichess_games(&username, cursor)
                .await
        }
        _ => Err(format!(
            "Unsupported platform: {}",
            platform
        )),
    }
}

#[tauri::command]
pub async fn get_player_profile(
    username: String,
    platform: Option<String>,
) -> Result<PlayerProfile, String> {
    let client = reqwest::Client::new();
    let target_platform = platform
        .unwrap_or_else(|| {
            "chesscom".to_string()
        });

    match target_platform.as_str() {
        "lichess" => {
            fetch_lichess_profile(
                &client, &username,
            )
            .await
        }
        _ => {
            fetch_chesscom_profile(
                &client, &username,
            )
            .await
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewMove {
    pub ply: u32,
    pub san: String,
    pub fen: String,
    pub uci: String,
}

#[tauri::command]
pub fn parse_pgn(
    pgn: String,
) -> Result<Vec<PreviewMove>, String> {
    let mut visitor = PgnVisitor::new();
    let mut reader =
        Reader::new(Cursor::new(pgn.as_bytes()));

    let (_metadata, positions) = match reader
        .read_game(&mut visitor)
    {
        Ok(Some(p)) => p,
        _ => return Err(
            "Failed to parse PGN game structure"
                .to_string(),
        ),
    };

    let preview_moves = positions
        .into_iter()
        .enumerate()
        .map(|(i, (san, fen, uci))| PreviewMove {
            ply: (i + 1) as u32,
            san,
            fen,
            uci,
        })
        .collect();

    Ok(preview_moves)
}

/// Starts or fully terminates the live engine process.
///
/// `start = true`  -> spawns the engine binary and performs the UCI handshake.
/// `start = false` -> sends "quit" to the engine and tears down all I/O threads.
///
/// Use `stop_live_analysis` to merely pause searching without killing the process.
/// `binary_path` is ignored when `start = false`.
#[tauri::command]
pub async fn toggle_live_engine(
    start: bool,
    state: State<'_, LiveEngineManager>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let cmd = if start {
        LiveCommand::Start {
            binary_path: app_state
                .engine_path
                .clone(),
        }
    } else {
        LiveCommand::Terminate
    };

    state.tx.send(cmd).map_err(|e| e.to_string())
}

/// Starts an infinite analysis of the given position.
///
/// Automatically interrupts any search already in progress before
/// loading the new FEN, so it is safe to call repeatedly as the
/// user steps through moves.
#[tauri::command]
pub async fn analyze_live_position(
    fen: String,
    multipv: u8,
    state: State<'_, LiveEngineManager>,
) -> Result<(), String> {
    state
        .tx
        .send(LiveCommand::Analyze {
            fen,
            multipv,
        })
        .map_err(|e| e.to_string())
}

/// Stops the current search without terminating the engine process.
///
/// Call this when the user navigates away from the board or closes
/// the analysis panel. To fully shut down the engine, use
/// `toggle_live_engine(false, ...)` instead.
#[tauri::command]
pub async fn stop_live_analysis(
    state: State<'_, LiveEngineManager>,
) -> Result<(), String> {
    state
        .tx
        .send(LiveCommand::Stop)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_analysis(
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Flip the flag to true. The analysis pipeline will check this flag.
    state
        .cancel_analysis_flag
        .store(true, Ordering::Relaxed);
    Ok(())
}
