use crate::engine::book::OpeningBook;
use crate::engine::classify::{
    classify, ClassifyArgs,
};
use crate::engine::see::{
    get_target_square, is_sacrifice,
};
use crate::engine::uci_engine::{
    Evaluation, UciEngine,
};
use crate::math::calculate_accuracy;

#[allow(unused_imports)]
use crate::models::game::{
    AnalysisSummary, AnalyzedMove, GameMetadata,
    MoveBadge, MoveCounts,
};
use crate::pgn::PgnVisitor;
use pgn_reader::Reader;
use std::io::Cursor;
use tauri::{AppHandle, Emitter, Manager};

use shakmaty::{
    fen::Fen, CastlingMode, Chess, Position,
};

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

    // Resolve the Chess Engine
    let target_triple = std::env::consts::ARCH
        .to_owned()
        + "-"
        + std::env::consts::OS;
    let engine_path = app
        .path()
        .resolve(
            format!(
                "core/engine/theoria-{}",
                target_triple
            ),
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|_| {
            "Failed to locate engine binary"
                .to_string()
        })?;

    // Resolve the Opening Book
    let book_path = app
        .path()
        .resolve(
            "core/database/book.bin",
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|_| {
            "Failed to locate opening book"
                .to_string()
        })?;

    // Load the database into memory once
    let book = OpeningBook::new(
        book_path.to_str().unwrap_or(""),
    );

    let binary_path_str =
        engine_path.to_str().unwrap();
    let mut engine =
        UciEngine::new(binary_path_str);
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

    // Accuracy trackers
    let mut white_win_loss = 0.0;
    let mut black_win_loss = 0.0;
    let mut white_moves = 0;
    let mut black_moves = 0;

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

        let best_eval = prev_eval;
        let normalized_cp = if ply_count % 2 != 0
        {
            -played_cp
        } else {
            played_cp
        };

        // Check for Forced Move
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

        // Check for Sacrifice using Static Exchange Evaluation
        let current_pos_opt =
            Fen::from_ascii(fen.as_bytes())
                .ok()
                .and_then(|f| {
                    f.into_position::<Chess>(
                        CastlingMode::Standard,
                    )
                    .ok()
                });

        let is_sacrifice_flag = if let Some(
            ref pos,
        ) =
            current_pos_opt
        {
            let player_color =
                if ply_count % 2 != 0 {
                    shakmaty::Color::White
                } else {
                    shakmaty::Color::Black
                };
            is_sacrifice(pos, player_color)
        } else {
            false
        };

        // 3. Check for Obvious Recapture
        let prev_target =
            get_target_square(&prev_san);
        let current_target =
            get_target_square(&san);

        // If the opponent takes our Queen, taking their Queen back is mathematically the
        // "Best" move (prevents a -9.0 drop).
        // However, this requires zero calculation from the player. We explicitly flag
        // immediate recaptures on the same square so the classifier degrades them
        // from "Great" down to "Best", preventing players from being rewarded for obvious moves.
        let is_obvious_recapture = prev_target
            .is_some()
            && current_target.is_some()
            && prev_target == current_target
            && san.contains('x');

        // 4. Check Opening Database
        let is_book_flag = current_pos_opt
            .as_ref()
            .map(|pos| {
                book.is_book_move(pos, &san)
            })
            .unwrap_or(false);

        let classify_args = ClassifyArgs {
            is_book: is_book_flag,
            prev_eval,
            played_eval: normalized_cp,
            best_move_eval: best_eval,
            multi_pv_evals: &multi_pv_evals,
            is_sacrifice: is_sacrifice_flag,
            is_obvious_recapture,
            prev_win_loss,
            is_forced_move,
        };

        // Classify the move
        let (classification, current_win_loss) =
            classify(classify_args);

        // Track loss for Accuracy CAPS Score
        let positive_loss =
            current_win_loss.max(0.0);
        if ply_count % 2 != 0 {
            white_win_loss += positive_loss;
            white_moves += 1;
        } else {
            black_win_loss += positive_loss;
            black_moves += 1;
        }

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
        white_accuracy: calculate_accuracy(
            white_win_loss,
            white_moves,
        ),
        black_accuracy: calculate_accuracy(
            black_win_loss,
            black_moves,
        ),
        move_counts,
        metadata,
    };

    app.emit("analysis-complete", &summary)
        .map_err(|e| e.to_string())?;

    Ok(())
}
