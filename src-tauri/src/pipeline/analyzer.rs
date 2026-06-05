use crate::data::book::OpeningBook;
use crate::data::pgn::PgnVisitor;
use crate::heuristics::accuracy::calculate_accuracy;
use crate::heuristics::context::evaluate_move_context;
use crate::models::game::{
    AnalysisProgress, AnalysisSummary,
    AnalyzedMove, MoveCounts,
};
use crate::uci::evaluation::{
    engine_to_white_pov, extract_cp,
    extract_mate, white_to_moving_pov,
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
use tauri::{AppHandle, Emitter};

pub fn run_analysis_pipeline(
    app: AppHandle,
    pgn: String,
    time_ms: u32,
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
    let engine_path = std::env::current_dir()
        .unwrap()
        .join("core/engine/stockfish-ubuntu-x86-64-bmi2");

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
    let initial_pos_cmd = "position startpos";

    // Analyze starting position using hybrid limit
    let (
        initial_eval,
        initial_best_move,
        _,
        initial_multi_pv_evals,
    ) = engine.analyze_position(
        initial_pos_cmd,
        &format!(
            "go depth 22 movetime {}",
            time_ms
        ),
    );

    // Standardize the initial setup strictly to Absolute White POV.
    let mut prev_eval = match initial_eval {
        Evaluation::Cp(cp) => {
            engine_to_white_pov(cp, true)
        }
        Evaluation::Mate(m) => {
            if m > 0 {
                10000
            } else {
                -10000
            }
        }
    };

    // The mate-in-N from the best move analysis of the *previous* position,
    // kept in Absolute White POV. This is what feeds `class_best_mate` each
    // ply — it is distinct from `prev_eval_obj` (which was the played eval).
    // At the start the starting position has no forced mate.
    let mut prev_best_mate: Option<i32> = None;

    let mut prev_multi_pv_evals: Vec<i32> =
        initial_multi_pv_evals
            .into_iter()
            .map(|v| engine_to_white_pov(v, true))
            .collect();

    let mut prev_fen = initial_fen.to_string();
    let mut prev_san = String::new();

    // total half-moves
    let mut ply_count = 1;

    // The drop in win probability caused by the previous move
    let mut prev_win_loss = 0.0;

    // The best engine move that the player should've played
    let mut prev_best_move_uci =
        initial_best_move;

    // The cumulative mathematical disadvantage accumulated by each player.
    // Is used to calculate their final accuracy CAPS scores.
    let mut white_win_loss = 0.0;
    let mut black_win_loss = 0.0;

    // Total moves per side
    // Is used as the denominator for accuracy calculations.
    let mut white_moves = 0;
    let mut black_moves = 0;

    // Aggregated tally of moves made by each side for each classification
    let mut move_counts_white =
        MoveCounts::default();
    let mut move_counts_black =
        MoveCounts::default();

    // Total plies for progress tracking
    let total_plies = positions.len() as u32;
    let mut analyzed_moves_collection =
        Vec::with_capacity(positions.len());
    let mut uci_moves_history = Vec::new();
    let mut board = Chess::default();

    // Helper: converts a UCI string to SAN in the context of a given position
    let get_san = |uci_str: &str,
                   pos_opt: &Option<Chess>|
     -> String {
        if let Some(pos) = pos_opt {
            if let Ok(uci) = UciMove::from_ascii(
                uci_str.as_bytes(),
            ) {
                if let Ok(m) = uci.to_move(pos) {
                    return San::from_move(
                        pos, m,
                    )
                    .to_string();
                }
            }
        }
        uci_str.to_string()
    };

    for (san, fen, _uci) in positions {
        // Perspective flags for normalization
        let is_white_moving = ply_count % 2 != 0;
        let is_white_to_move_after_play =
            !is_white_moving;

        let prev_pos_cmd =
            if uci_moves_history.is_empty() {
                "position startpos".to_string()
            } else {
                format!(
                    "position startpos moves {}",
                    uci_moves_history.join(" ")
                )
            };

        // Parse previous board state for early checks
        let prev_pos =
            Fen::from_ascii(prev_fen.as_bytes())
                .ok()
                .and_then(|f| {
                    f.into_position::<Chess>(
                        CastlingMode::Standard,
                    )
                    .ok()
                });

        // Fast checks to evaluate book and forced moves before hitting the engine
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

        // Dynamic Engine Limits
        let go_cmd =
            if is_book_flag || is_forced_move {
                "go depth 12".to_string()
            } else {
                format!(
                    "go depth 22 movetime {}",
                    time_ms
                )
            };

        // Plays move on board and builds history command
        let parsed_san =
            San::from_ascii(san.as_bytes()).ok();
        let m = parsed_san
            .and_then(|s| s.to_move(&board).ok());

        let played_uci =
            if let Some(ref valid_move) = m {
                UciMove::from_move(
                    valid_move.clone(),
                    CastlingMode::Standard,
                )
                .to_string()
            } else {
                String::new()
            };

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

        // Evaluate the position after the played move
        let (
            mut played_eval,
            mut opponent_best_move,
            mut pv,
            mut multi_pv_evals,
        ) = engine.analyze_position(
            &current_pos_cmd,
            &go_cmd,
        );

        // Normalize evaluations to Absolute White POV
        let mut played_cp =
            extract_cp(&played_eval);
        let mut normalized_cp =
            engine_to_white_pov(
                played_cp,
                is_white_to_move_after_play,
            );

        multi_pv_evals = multi_pv_evals
            .into_iter()
            .map(|v| {
                engine_to_white_pov(
                    v,
                    is_white_to_move_after_play,
                )
            })
            .collect();

        let mut best_move_san = get_san(
            &prev_best_move_uci,
            &prev_pos,
        );

        // Delta check for horizon effect
        let class_prev_eval = white_to_moving_pov(
            prev_eval,
            is_white_moving,
        );
        let class_played_eval =
            white_to_moving_pov(
                normalized_cp,
                is_white_moving,
            );
        let eval_drop =
            class_prev_eval - class_played_eval;

        if san == best_move_san && eval_drop > 60
        {
            // 3.5s investigation allocation
            let deep_cmd = format!(
                "go depth 24 movetime {}",
                3500
            );

            // Re-evaluate the previous position deeply
            let (
                deep_prev_eval,
                deep_prev_best,
                _,
                deep_prev_multi,
            ) = engine.analyze_position(
                &prev_pos_cmd,
                &deep_cmd,
            );

            // Re-evaluate the current position deeply
            let (
                deep_played_eval,
                deep_opp_best,
                deep_pv,
                deep_multi,
            ) = engine.analyze_position(
                &current_pos_cmd,
                &deep_cmd,
            );

            // Re-normalize previous state to Absolute White POV.
            // Also update prev_best_mate from the deeper re-analysis so the
            // classifier sees the accurate mate distance.
            prev_best_move_uci = deep_prev_best;
            prev_best_mate =
                extract_mate(&deep_prev_eval)
                    .map(|m| {
                        engine_to_white_pov(
                            m,
                            is_white_moving,
                        )
                    });
            prev_eval = engine_to_white_pov(
                extract_cp(&deep_prev_eval),
                is_white_moving,
            );
            prev_multi_pv_evals = deep_prev_multi
                .into_iter()
                .map(|v| {
                    engine_to_white_pov(
                        v,
                        is_white_moving,
                    )
                })
                .collect();

            // Re-calculate the expected move in case the deep search changed its mind
            best_move_san = get_san(
                &prev_best_move_uci,
                &prev_pos,
            );

            // Re-normalize current state to Absolute White POV
            played_eval = deep_played_eval;
            opponent_best_move = deep_opp_best;
            pv = deep_pv;
            multi_pv_evals = deep_multi
                .into_iter()
                .map(|v| {
                    engine_to_white_pov(
                        v,
                        is_white_to_move_after_play,
                    )
                })
                .collect();

            played_cp = extract_cp(&played_eval);
            normalized_cp = engine_to_white_pov(
                played_cp,
                is_white_to_move_after_play,
            );
        }

        // Extract the played move's mate score in Absolute White POV.
        // `prev_best_mate` already holds the best-move mate from the previous
        // position's analysis (set at the end of the previous iteration, or
        // updated above during the horizon re-analysis).
        let class_played_mate =
            extract_mate(&played_eval).map(|m| {
                engine_to_white_pov(
                    m,
                    is_white_to_move_after_play,
                )
            });

        let (classification, current_win_loss) =
            evaluate_move_context(
                &san,
                &fen,
                &prev_san,
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

        // Increment move counters and tally classifications per side
        if ply_count % 2 != 0 {
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

        // Construct AnalyzedMove payload
        let analyzed_move = AnalyzedMove {
            ply: ply_count,
            san: san.clone(),
            fen: fen.clone(),
            uci: played_uci,
            played_eval: normalized_cp,
            prev_best_eval: prev_eval,
            best_move_san: best_move_san.clone(),
            classification,
            principal_variation: pv,
            mate_in: class_played_mate,
            best_mate_in: prev_best_mate,
        };

        analyzed_moves_collection
            .push(analyzed_move);

        // Emit progress
        app.emit(
            "analysis-progress",
            AnalysisProgress {
                current_ply: ply_count,
                total_plies,
            },
        )
        .map_err(|e| e.to_string())?;

        // `prev_best_mate` must come from the best-move analysis of the current
        // position (i.e. what the opponent will face). The engine returns scores
        // relative to the side to move, so after the played move it is the
        // opponent's turn; we negate to stay in Absolute White POV before storing.
        prev_best_mate =
            extract_mate(&played_eval).map(|m| {
                engine_to_white_pov(
                    m,
                    is_white_to_move_after_play,
                )
            });

        prev_eval = normalized_cp;
        prev_fen = fen;
        prev_san = san.clone();
        prev_win_loss = current_win_loss;
        prev_best_move_uci = opponent_best_move;
        prev_multi_pv_evals = multi_pv_evals;
        ply_count += 1;
    }

    engine.quit();

    // Construct and emit the final analysis summary
    let summary = AnalysisSummary {
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
    };

    app.emit("analysis-complete", &summary)
        .map_err(|e| e.to_string())?;

    Ok(())
}
