use crate::data::book::OpeningBook;
use crate::data::pgn::PgnVisitor;
use crate::heuristics::accuracy::calculate_accuracy;
use crate::heuristics::context::evaluate_move_context;
use crate::models::engine_config::EngineConfig;
use crate::models::game::{
    AnalysisProgress, AnalysisSummary,
    AnalyzedMove, MoveCounts,
};
use crate::uci::evaluation::{
    bare_san, engine_to_white_pov, extract_cp,
    extract_mate_white_pov, normalize_multi_pv,
    uci_to_san, white_to_moving_pov,
};
use crate::uci::uci_engine::{
    Evaluation, UciEngine,
};

use pgn_reader::Reader;
use shakmaty::san::San;
use shakmaty::uci::UciMove;
use shakmaty::{
    fen::Fen, CastlingMode, Chess, Position,
};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub fn run_analysis_pipeline(
    app: AppHandle,
    pgn: String,
    time_ms: u32,
    engine_path: String,
    book: Arc<OpeningBook>,
    cancel_flag: Arc<AtomicBool>,
    config: Arc<std::sync::Mutex<EngineConfig>>,
) -> Result<(), String> {
    app.emit("analysis-started", ())
        .map_err(|e| e.to_string())?;

    // Snapshot the config once for the lifetime of this pipeline run.
    // Changes made via configure_engine during analysis take effect on the next run.
    let config = config.lock().unwrap().clone();

    // Deep re-analysis budget: ~2.5× the normal per-move time,
    // clamped to a minimum of 3 s so fast configs still investigate properly.
    let deep_time_ms = ((time_ms as f32 * 2.5)
        as u32)
        .max(3_000);

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

    let mut engine =
        UciEngine::new(&engine_path, &config);

    // Seeds `prev_eval` / `prev_best_move_uci`.
    let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let (
        initial_eval,
        initial_best_move,
        _,
        initial_multi_pv_evals,
    ) = engine.analyze_position(
        "position startpos",
        &format!(
            "go depth 22 movetime {}",
            time_ms
        ),
    );

    // All running state is kept in Absolute White POV.
    let mut prev_eval = match initial_eval {
        Evaluation::Cp(cp) => {
            engine_to_white_pov(cp, true)
        }
        Evaluation::Mate(m) => {
            if m > 0 {
                10_000
            } else {
                -10_000
            }
        }
    };
    let mut prev_best_mate: Option<i32> = None;
    let mut prev_multi_pv_evals =
        normalize_multi_pv(
            initial_multi_pv_evals,
            true,
        );

    let mut prev_fen = initial_fen.to_string();
    let mut prev_san = String::new();
    // The drop in win probability caused by the previous move
    let mut prev_win_loss = 0.0_f64;

    // The best engine move that the player should've played
    let mut prev_best_move_uci =
        initial_best_move;

    // The cumulative mathematical disadvantage accumulated by each player.
    // Is used to calculate their final accuracy CAPS scores.
    let mut white_win_loss = 0.0_f64;
    let mut black_win_loss = 0.0_f64;

    // Total moves per side
    // Is used as the denominator for accuracy calculations.
    let mut white_moves = 0u32;
    let mut black_moves = 0u32;

    // Aggregated tally of moves made by each side for each classification
    let mut move_counts_white =
        MoveCounts::default();
    let mut move_counts_black =
        MoveCounts::default();

    // Total plies for progress tracking
    let total_plies = positions.len() as u32;
    let mut analyzed_moves_collection =
        Vec::with_capacity(positions.len());
    let mut uci_moves_history: Vec<String> =
        Vec::new();
    let mut board = Chess::default();
    let mut ply_count = 1u32;

    for (san, fen, _uci) in positions {
        if cancel_flag.load(Ordering::Relaxed) {
            engine.quit();
            return Ok(());
        }

        // Perspective flags for normalization
        let is_white_moving = ply_count % 2 != 0;
        let is_white_to_move_after_play =
            !is_white_moving;

        // Builds position commands
        let prev_pos_cmd =
            if uci_moves_history.is_empty() {
                "position startpos".to_string()
            } else {
                format!(
                    "position startpos moves {}",
                    uci_moves_history.join(" ")
                )
            };

        // Resolves the previous board state for book / forced-move checks
        let prev_pos =
            Fen::from_ascii(prev_fen.as_bytes())
                .ok()
                .and_then(|f| {
                    f.into_position::<Chess>(
                        CastlingMode::Standard,
                    )
                    .ok()
                });

        let is_forced_move = prev_pos
            .as_ref()
            .map(|pos| {
                pos.legal_moves().len() == 1
            })
            .unwrap_or(false);

        let is_book_flag = prev_pos
            .as_ref()
            .map(|pos| {
                book.is_book_move(
                    pos, &prev_fen, &san,
                )
            })
            .unwrap_or(false);

        // Engine search limits
        let go_cmd =
            if is_book_flag || is_forced_move {
                "go depth 12".to_string()
            } else {
                format!(
                    "go depth 22 movetime {}",
                    time_ms
                )
            };

        // Advance the board and record the played UCI move
        let parsed_san =
            San::from_ascii(san.as_bytes()).ok();
        let m = parsed_san
            .and_then(|s| s.to_move(&board).ok());

        let played_uci = m
            .as_ref()
            .map(|mv| {
                UciMove::from_move(
                    mv.clone(),
                    CastlingMode::Standard,
                )
                .to_string()
            })
            .unwrap_or_default();

        if let Some(valid_move) = m {
            board = board
                .clone()
                .play(valid_move)
                .unwrap_or(board);
        }

        uci_moves_history
            .push(played_uci.clone());
        let current_pos_cmd = format!(
            "position startpos moves {}",
            uci_moves_history.join(" ")
        );

        // Evaluate position after the played move
        let (
            mut played_eval,
            mut opponent_best_move,
            mut pv,
            mut multi_pv_evals,
        ) = engine.analyze_position(
            &current_pos_cmd,
            &go_cmd,
        );

        let mut normalized_cp =
            engine_to_white_pov(
                extract_cp(&played_eval),
                is_white_to_move_after_play,
            );

        multi_pv_evals = normalize_multi_pv(
            multi_pv_evals,
            is_white_to_move_after_play,
        );

        // Resolves best-move SAN from previous position
        let mut best_move_san =
            bare_san(&uci_to_san(
                &prev_best_move_uci,
                &prev_pos,
            ))
            .to_string();

        // Horizon-effect guard
        let eval_drop = white_to_moving_pov(
            prev_eval,
            is_white_moving,
        ) - white_to_moving_pov(
            normalized_cp,
            is_white_moving,
        );

        if bare_san(&san) == best_move_san
            && eval_drop > 60
        {
            let deep_cmd = format!(
                "go depth 24 movetime {}",
                deep_time_ms
            );

            let (
                deep_prev_eval,
                deep_prev_best,
                _,
                deep_prev_multi,
            ) = engine.analyze_position(
                &prev_pos_cmd,
                &deep_cmd,
            );

            let (
                deep_played_eval,
                deep_opp_best,
                deep_pv,
                deep_multi,
            ) = engine.analyze_position(
                &current_pos_cmd,
                &deep_cmd,
            );

            // Updates previous-position state from the deeper search.
            prev_best_move_uci = deep_prev_best;
            prev_best_mate =
                extract_mate_white_pov(
                    &deep_prev_eval,
                    is_white_moving,
                );
            prev_eval = engine_to_white_pov(
                extract_cp(&deep_prev_eval),
                is_white_moving,
            );
            prev_multi_pv_evals =
                normalize_multi_pv(
                    deep_prev_multi,
                    is_white_moving,
                );

            best_move_san =
                bare_san(&uci_to_san(
                    &prev_best_move_uci,
                    &prev_pos,
                ))
                .to_string();

            // Updates current-position state from the deeper search.
            played_eval = deep_played_eval;
            opponent_best_move = deep_opp_best;
            pv = deep_pv;
            multi_pv_evals = normalize_multi_pv(
                deep_multi,
                is_white_to_move_after_play,
            );

            normalized_cp = engine_to_white_pov(
                extract_cp(&played_eval),
                is_white_to_move_after_play,
            );
        }

        // Classify the move
        let class_played_mate =
            extract_mate_white_pov(
                &played_eval,
                is_white_to_move_after_play,
            );

        let (classification, current_win_loss) =
            evaluate_move_context(
                bare_san(&san),
                &fen,
                bare_san(&prev_san),
                &prev_fen,
                &best_move_san,
                prev_eval,
                normalized_cp,
                prev_win_loss,
                &prev_multi_pv_evals,
                ply_count,
                is_book_flag,
                is_forced_move,
                prev_best_mate,
                class_played_mate,
            );

        let positive_loss =
            current_win_loss.max(0.0);

        if is_white_moving {
            white_win_loss += positive_loss;
            white_moves += 1;
            move_counts_white
                .tally(&classification);
        } else {
            black_win_loss += positive_loss;
            black_moves += 1;
            move_counts_black
                .tally(&classification);
        }

        analyzed_moves_collection.push(
            AnalyzedMove {
                ply: ply_count,
                san: san.clone(),
                fen: fen.clone(),
                uci: played_uci,
                played_eval: normalized_cp,
                prev_best_eval: prev_eval,
                best_move_san: best_move_san
                    .clone(),
                classification,
                principal_variation: pv,
                mate_in: class_played_mate,
                best_mate_in: prev_best_mate,
            },
        );

        app.emit(
            "analysis-progress",
            AnalysisProgress {
                current_ply: ply_count,
                total_plies,
            },
        )
        .map_err(|e| e.to_string())?;

        // Roll state forward for next ply
        prev_best_mate = extract_mate_white_pov(
            &played_eval,
            is_white_to_move_after_play,
        );
        prev_eval = normalized_cp;
        prev_fen = fen;
        prev_san = san; // preserve display form
        prev_win_loss = current_win_loss;
        prev_best_move_uci = opponent_best_move;
        prev_multi_pv_evals = multi_pv_evals;
        ply_count += 1;
    }

    engine.quit();

    app.emit(
        "analysis-complete",
        &AnalysisSummary {
            white_accuracy: calculate_accuracy(
                white_win_loss,
                white_moves,
            ),
            black_accuracy: calculate_accuracy(
                black_win_loss,
                black_moves,
            ),
            move_counts_black,
            move_counts_white,
            metadata,
            moves: analyzed_moves_collection,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
