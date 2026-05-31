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