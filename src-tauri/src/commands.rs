use crate::engine::book::OpeningBook;
use crate::engine::classify::{
    classify, ClassifyArgs,
};
use crate::engine::see::{
    get_target_square, is_losing_material,
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
use shakmaty::san::San;
use shakmaty::uci::UciMove;
use std::io::Cursor;
use tauri::{AppHandle, Emitter};

use shakmaty::{
    fen::Fen, CastlingMode, Chess, Position,
};

#[tauri::command]
pub fn analyze_game(
    app: AppHandle,
    pgn: String,
    target_depth: Option<u8>,
) -> Result<(), String> {
    // The search depth for the UCI chess engine
    let depth = target_depth.unwrap_or(15);

    // Spawn a dedicated background thread for the blocking UCI engine
    std::thread::spawn(move || {
        if let Err(e) = run_analysis_pipeline(
            app.clone(),
            pgn,
            depth,
        ) {
            let _ =
                app.emit("analysis-error", &e);
        }
    });

    Ok(())
}

fn run_analysis_pipeline(
    app: AppHandle,
    pgn: String,
    depth: u8,
) -> Result<(), String> {
    app.emit("analysis-started", ())
        .map_err(|e| e.to_string())?;

    // Visitor impl used to construct the game metadata and track positions
    let mut visitor = PgnVisitor::new();

    // Streams the raw PGN string bytes
    let mut reader =
        Reader::new(Cursor::new(pgn.as_bytes()));

    // Parses game headers
    // Maps the sequence of SAN moves to their FEN positions
    let (metadata, positions) =
        match reader.read_game(&mut visitor) {
            Ok(Some(p)) => p,
            _ => {
                return Err("Failed to parse PGN"
                    .to_string())
            }
        };

    // temp fix
    // Bypass Tauri's resource and externalBin resolver for now.
    let target_triple =
        "x86_64-unknown-linux-gnu";

    let engine_path = std::env::current_dir()
        .unwrap()
        .join(format!(
            "core/engine/theoria-{}",
            target_triple
        ));

    let book_path = std::env::current_dir()
        .unwrap()
        .join("core/database/book.bin");

    // Loads the opening database
    let book = OpeningBook::new(
        book_path.to_str().unwrap_or(""),
    );

    // Loads the engine
    let mut engine = UciEngine::new(
        engine_path.to_str().unwrap(),
    );

    // Starting position
    let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // Loads Starting position
    let (
        initial_eval,
        initial_best_move,
        _,
        initial_multi_pv_evals,
    ) = engine
        .analyze_position(initial_fen, depth);

    // The evaluation score from the position immediately preceding the move the player is about to make.
    let mut prev_eval = match initial_eval {
        Evaluation::Cp(cp) => cp,
        Evaluation::Mate(m) => {
            if m > 0 {
                10000
            } else {
                -10000
            }
        }
    };

    // The SAN and FEN of the previous half-move.
    // Is used for context comparison against the current move.
    let mut prev_fen = initial_fen.to_string();
    let mut prev_san = String::new();

    // total half-moves
    let mut ply_count = 1;

    // The drop in win probability caused by the previous move
    let mut prev_win_loss = 0.0;

    // The best engine move that the player should've played
    let mut prev_best_move_uci =
        initial_best_move;

    // These are the top-3 engine choices the CURRENT player had available
    // before choosing their move.
    let mut prev_multi_pv_evals =
        initial_multi_pv_evals;

    // The cumulative mathematical disadvantage accumulated by each player.
    // Is used to calculate their final accuracy CAPS scores.
    let mut white_win_loss = 0.0;
    let mut black_win_loss = 0.0;

    // Total moves per side
    // Is used as the denominator for accuracy calculations.
    let mut white_moves = 0;
    let mut black_moves = 0;

    // Aggregated tally of moves made by each side for each classification
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

    // Iterates over the moves and analyzes them
    for (san, fen) in positions {
        // Get engine evaluation
        let (
            played_eval,
            opponent_best_move,
            pv,
            multi_pv_evals,
        ) = engine.analyze_position(&fen, depth);

        // Get evaluation for the played move
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

        // Normalized means it is from white's perspective,
        // white goes towards positive and black goes towards negative.
        let normalized_cp = if ply_count % 2 != 0
        {
            -played_cp
        } else {
            played_cp
        };

        // Delegate to isolated logic helper.
        let (
            classification,
            current_win_loss,
            best_move_san,
        ) = evaluate_move_context(
            &san,
            &fen,
            &prev_san,
            &prev_fen,
            &prev_best_move_uci,
            prev_eval,
            normalized_cp,
            prev_win_loss,
            &prev_multi_pv_evals,
            ply_count,
            &book,
        );

        //
        let positive_loss =
            current_win_loss.max(0.0);

        // Increments move counter
        if ply_count % 2 != 0 {
            white_win_loss += positive_loss;
            white_moves += 1;
        } else {
            black_win_loss += positive_loss;
            black_moves += 1;
        }

        // Increments the move badge tally
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

        // Construct AnalyzedMove data
        let analyzed_move = AnalyzedMove {
            ply: ply_count,
            san: san.clone(),
            fen: fen.clone(),
            played_eval: normalized_cp,
            prev_best_eval: prev_eval,
            best_move_san,
            classification,
            principal_variation: pv,
        };

        // Emit analyzed move
        app.emit("batch-tick", &analyzed_move)
            .map_err(|e| e.to_string())?;

        // Update iteration state
        prev_eval = normalized_cp;
        prev_fen = fen;
        prev_san = san.clone();
        prev_win_loss = current_win_loss;
        prev_best_move_uci = opponent_best_move;
        prev_multi_pv_evals = multi_pv_evals;
        ply_count += 1;
    }

    engine.quit();

    // Construct Analysis
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

    // Emit analysis completion
    app.emit("analysis-complete", &summary)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn evaluate_move_context(
    san: &str,
    fen: &str,
    prev_san: &str,
    prev_fen: &str,
    prev_best_move_uci: &str,
    prev_eval: i32,
    normalized_cp: i32,
    prev_win_loss: f64,
    multi_pv_evals: &[i32],
    ply_count: u32,
    book: &OpeningBook,
) -> (MoveBadge, f64, String) {
    // A multiplier to convert White-normalized scores into the moving player's perspective.
    let pov_multiplier =
        if ply_count % 2 != 0 { 1 } else { -1 };

    // Centipawn engine values to be passed to classify function
    let class_prev_eval =
        prev_eval * pov_multiplier;
    let class_played_eval =
        normalized_cp * pov_multiplier;

    // multi_pv_evals[0] is the engine's top-line score
    // for the position before the move.
    let class_best_eval = multi_pv_evals
        .first()
        .copied()
        .unwrap_or(class_prev_eval);

    // Parsed board state before the move.
    // Is used to get forced moves, legal moves, and book lookups.
    let prev_pos =
        Fen::from_ascii(prev_fen.as_bytes())
            .ok()
            .and_then(|f| {
                f.into_position::<Chess>(
                    CastlingMode::Standard,
                )
                .ok()
            });

    // Parsed board state after the move.
    // Is used to check for sacrifices.
    let current_pos_opt =
        Fen::from_ascii(fen.as_bytes())
            .ok()
            .and_then(|f| {
                f.into_position::<Chess>(
                    CastlingMode::Standard,
                )
                .ok()
            });

    // Checks if its the only legal move
    let is_forced_move = prev_pos
        .as_ref()
        .map(|pos| pos.legal_moves().len() == 1)
        .unwrap_or(false);

    // Checks if the move is a sacrifice
    let is_losing_material_flag =
        if let Some(ref pos) = current_pos_opt {
            let color = if ply_count % 2 != 0 {
                shakmaty::Color::White
            } else {
                shakmaty::Color::Black
            };
            is_losing_material(pos, color)
        } else {
            false
        };

    let prev_target = get_target_square(prev_san);
    let current_target = get_target_square(san);

    // During a queen to queen capture, taking their Queen back is mathematically the
    // "Best" move (prevents a -9.0 drop) and in most cases it is technicaly a "Great"
    // move because not recapturing usually just loses.
    // However, this requires zero calculation from the player. We explicitly flag
    // immediate recaptures on the same square so the classifier degrades them
    // from "Great" down to "Best"
    // Basically just a check to avoid awarding "Great" moves for an obvious recapture.
    let is_obvious_recapture = prev_target
        .is_some()
        && current_target.is_some()
        && prev_target == current_target
        && san.contains('x');

    // Pass the position from before the move so the book checks if
    // the played san is a theory move from given previous pos.
    let is_book_flag = prev_pos
        .as_ref()
        .map(|pos| {
            book.is_book_move(pos, prev_fen, san)
        })
        .unwrap_or(false);

    // Engine best move converted to SAN
    let best_move_san = if let Some(pos) =
        &prev_pos
    {
        if let Ok(uci) = UciMove::from_ascii(
            prev_best_move_uci.as_bytes(),
        ) {
            if let Ok(m) = uci.to_move(pos) {
                San::from_move(pos, m).to_string()
            } else {
                prev_best_move_uci.to_string()
            }
        } else {
            prev_best_move_uci.to_string()
        }
    } else {
        prev_best_move_uci.to_string()
    };

    let is_best_engine_move =
        san == best_move_san;

    // Construct classification arguments
    let classify_args = ClassifyArgs {
        is_book: is_book_flag,
        prev_eval: class_prev_eval,
        played_eval: class_played_eval,
        prev_best_eval: class_best_eval,
        multi_pv_evals,
        is_losing_material:
            is_losing_material_flag,
        is_obvious_recapture,
        prev_win_loss,
        is_forced_move,
        is_best_engine_move,
    };

    // Classify move
    let (classification, current_win_loss) =
        classify(classify_args);

    (
        classification,
        current_win_loss,
        best_move_san,
    )
}
