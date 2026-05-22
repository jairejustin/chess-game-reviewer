use crate::engine::classify::classify;
use crate::engine::uci_engine::{
    Evaluation, UciEngine,
};
use crate::models::game::{
    AnalysisSummary, AnalyzedMove, GameMetadata,
    MoveBadge, MoveCounts,
};

use crate::pgn::PgnVisitor;
use pgn_reader::Reader;
use std::io::Cursor;
use tauri::{AppHandle, Emitter};

use shakmaty::{
    fen::Fen, CastlingMode, Chess, Position,
};

/// Extracts the 2-character destination square from a capture SAN.
fn capture_square(san: &str) -> Option<&str> {
    if let Some(x_idx) = san.find('x') {
        let square_start = x_idx + 1;
        let square_end = square_start + 2;
        if square_end <= san.len() {
            return Some(
                &san[square_start..square_end],
            );
        }
    }
    None
}

pub fn calculate_material_balance(
    fen: &str,
) -> i32 {
    let mut balance = 0;
    // Extract just the board layout part of the FEN
    for c in fen
        .split(' ')
        .next()
        .unwrap_or("")
        .chars()
    {
        match c {
            'P' => balance += 1,
            'N' | 'B' => balance += 3,
            'R' => balance += 5,
            'Q' => balance += 9,
            'p' => balance -= 1,
            'n' | 'b' => balance -= 3,
            'r' => balance -= 5,
            'q' => balance -= 9,
            _ => {}
        }
    }
    balance
}

#[tauri::command]
pub fn analyze_game(
    app: AppHandle,
    pgn: String,
) -> Result<(), String> {
    let mut visitor = PgnVisitor::new();
    let mut reader =
        Reader::new(Cursor::new(pgn.as_bytes()));

    let (metadata, positions) =
        match reader.read_game(&mut visitor) {
            Ok(Some(p)) => p,
            _ => {
                return Err("Failed to parse PGN"
                    .to_string())
            }
        };

    let binary_path = "/home/j/Documents/Personal Projects/chess-analyze/src-tauri/binaries/theoria-x86_64-unknown-linux-gnu";
    let mut engine = UciEngine::new(binary_path);
    let depth = 15;

    let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let initial = engine
        .analyze_position(initial_fen, depth);

    let mut prev_eval = match initial.0 {
        Evaluation::Cp(cp) => cp,
        Evaluation::Mate(m) => {
            if m > 0 {
                10000
            } else {
                -10000
            }
        }
    };

    let mut prev_fen = initial_fen.to_string();
    let mut ply_count = 1;
    let mut prev_san = String::new();
    let mut prev_win_loss = 0.0;

    let mut move_counts = MoveCounts {
        brilliant: 0,
        great: 0,
        best: 0,
        excellent: 0,
        good: 0,
        inaccuracy: 0,
        mistake: 0,
        blunder: 0,
        miss: 0,
        book: 0,
        forced: 0,
    };

    for (san, fen) in positions {
        let (
            played_eval,
            best_move,
            pv,
            multi_pv_evals,
        ) = engine.analyze_position(&fen, depth);

        let played_cp = match played_eval {
            Evaluation::Cp(cp) => cp,
            Evaluation::Mate(m) => {
                if m > 0 {
                    10000
                } else {
                    -10000
                }
            }
        };

        let prev_material =
            calculate_material_balance(&prev_fen);
        let current_material =
            calculate_material_balance(&fen);
        let raw_material_delta =
            current_material - prev_material;
        let material_delta = if ply_count % 2 != 0
        {
            raw_material_delta
        } else {
            -raw_material_delta
        };

        let best_eval = prev_eval;
        let normalized_cp = if ply_count % 2 != 0
        {
            -played_cp
        } else {
            played_cp
        };

        let is_forced_move =
            Fen::from_ascii(prev_fen.as_bytes())
                .ok()
                .and_then(|f| {
                    f.into_position::<Chess>(
                        CastlingMode::Standard,
                    )
                    .ok()
                })
                .map(|pos| {
                    pos.legal_moves().len() == 1
                })
                .unwrap_or(false);

        let prev_target =
            capture_square(&prev_san);
        let current_target = capture_square(&san);
        let is_obvious_recapture = prev_target
            .is_some()
            && current_target.is_some()
            && prev_target == current_target;

        let (classification, current_win_loss) = {
            classify(
                prev_eval,
                normalized_cp,
                best_eval,
                &multi_pv_evals,
                material_delta,
                is_obvious_recapture,
                prev_win_loss,
                is_forced_move,
            )
        };

        match classification {
            MoveBadge::Brilliant => {
                move_counts.brilliant += 1
            }
            MoveBadge::Great => {
                move_counts.great += 1
            }
            MoveBadge::Best => {
                move_counts.best += 1
            }
            MoveBadge::Excellent => {
                move_counts.excellent += 1
            }
            MoveBadge::Good => {
                move_counts.good += 1
            }
            MoveBadge::Inaccuracy => {
                move_counts.inaccuracy += 1
            }
            MoveBadge::Mistake => {
                move_counts.mistake += 1
            }
            MoveBadge::Blunder => {
                move_counts.blunder += 1
            }
            MoveBadge::Miss => {
                move_counts.miss += 1
            }
            MoveBadge::Book => {
                move_counts.book += 1
            }
            MoveBadge::Forced => {
                move_counts.forced += 1
            }
        }

        let analyzed_move = AnalyzedMove {
            ply: ply_count,
            san: san.clone(),
            fen: fen.clone(),
            played_eval: played_cp,
            best_move_eval: best_eval,
            best_move_san: best_move,
            classification,
            principal_variation: pv,
        };

        app.emit("batch-tick", &analyzed_move)
            .map_err(|e| e.to_string())?;

        prev_eval = normalized_cp;
        prev_fen = fen;
        prev_san = san.clone();
        prev_win_loss = current_win_loss;
        ply_count += 1;
    }

    engine.quit();

    let summary = AnalysisSummary {
        white_accuracy: 100.0,
        black_accuracy: 100.0,
        move_counts,
        metadata,
    };

    app.emit("analysis-complete", &summary)
        .map_err(|e| e.to_string())?;

    Ok(())
}
