use crate::data::fetcher::fetch_chesscom_games;
use crate::models::fetch::{
    ChessComCursor, FetchResult,
};
use crate::pipeline::analyzer::run_analysis_pipeline;

#[allow(unused_imports)]
use crate::models::game::{
    AnalysisProgress, AnalysisSummary,
    AnalyzedMove, GameMetadata, MoveBadge,
    MoveCounts,
};

use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn analyze_game(
    app: AppHandle,
    pgn: String,
    target_time_ms: Option<u32>,
) -> Result<(), String> {
    // The search time for the UCI chess engine
    let time_ms = target_time_ms.unwrap_or(1500);

    // Spawn a dedicated background thread for the blocking UCI engine
    std::thread::spawn(move || {
        if let Err(e) = run_analysis_pipeline(
            app.clone(),
            pgn,
            time_ms,
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
        _ => Err(format!(
            "Unsupported platform: {}",
            platform
        )),
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
pub fn parse_pgn(pgn: String) -> Result<Vec<PreviewMove>, String> {
    use pgn_reader::Reader;
    use crate::data::pgn::PgnVisitor;
    use std::io::Cursor;

    let mut visitor = PgnVisitor::new();
    let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));

    let (_metadata, positions) = match reader.read_game(&mut visitor) {
        Ok(Some(p)) => p,
        _ => return Err("Failed to parse PGN game structure".to_string()),
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
