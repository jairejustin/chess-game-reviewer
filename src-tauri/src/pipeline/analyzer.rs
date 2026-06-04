use crate::data::book::OpeningBook;
use crate::heuristics::accuracy::calculate_accuracy;
use crate::heuristics::classify::{
    classify, ClassifyArgs,
};
use crate::heuristics::see::{
    get_target_square, is_material_sacrifice,
};

use crate::data::pgn::PgnVisitor;
#[allow(unused_imports)]
use crate::models::game::{
    AnalysisProgress, AnalysisSummary,
    AnalyzedMove, GameMetadata, MoveBadge,
    MoveCounts,
};
use crate::uci::uci_engine::{
    Evaluation, UciEngine,
};
use pgn_reader::Reader;
use shakmaty::san::San;
use shakmaty::uci::UciMove;
use std::io::Cursor;
use tauri::{AppHandle, Emitter};

use shakmaty::{
    fen::Fen, CastlingMode, Chess, Position,
};

/// Converts a raw engine score (relative to the side whose turn it is to move)
/// into an absolute evaluation from White's perspective (+ is winning for White).
pub fn engine_to_white_pov(
    score: i32,
    is_white_to_move: bool,
) -> i32 {
    if is_white_to_move {
        score
    } else {
        -score
    }
}

/// Converts an absolute White evaluation into the perspective of the player making the move
/// (+ is winning for the moving player).
pub fn white_to_moving_pov(
    score: i32,
    is_white_moving: bool,
) -> i32 {
    if is_white_moving {
        score
    } else {
        -score
    }
}

/// Extracts a clamped centipawn value from an `Evaluation`.
fn extract_cp(eval: &Evaluation) -> i32 {
    match eval {
        Evaluation::Cp(cp) => *cp,
        Evaluation::Mate(m) => {
            if *m > 0 {
                10000
            } else {
                -10000
            }
        }
    }
}

/// Extracts the raw mate-in-N from an `Evaluation`, if it is one.
fn extract_mate(
    eval: &Evaluation,
) -> Option<i32> {
    match eval {
        Evaluation::Mate(m) => Some(*m),
        Evaluation::Cp(_) => None,
    }
}

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

    // Loads Starting position using hybrid limit
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
    let mut move_counts_white = MoveCounts {
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
    let mut move_counts_black = MoveCounts {
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

    // Total plies for progress tracking
    let total_plies = positions.len() as u32;
    let mut analyzed_moves_collection =
        Vec::with_capacity(positions.len());
    let mut uci_moves_history = Vec::new();
    let mut board = Chess::default();

    // Helper: Converts UCI strings to SAN
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

        // Plays move on board and build history command
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

        // Fast Evaluation with dynamic limits
        let (
            mut played_eval,
            mut opponent_best_move,
            mut pv,
            mut multi_pv_evals,
        ) = engine.analyze_position(
            &current_pos_cmd,
            &go_cmd,
        );

        // Normalize evaluations
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

        // If the player played the exact best move, but the evaluation inexplicably dropped
        // by more than x centipawn threshold, the engine likely suffered from horizon effect.
        if san == best_move_san && eval_drop > 60
        {
            // 3.5s investigation allocation
            let deep_time = 3500;
            let deep_cmd = format!(
                "go depth 24 movetime {}",
                deep_time
            );

            // Re-evaluate the prev position deeply
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

            // Re-normalize overwritten previous state to Absolute White POV.
            // Also update prev_best_mate from the deeper re-analysis of the
            // previous position, so the classifier sees the accurate mate distance.
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

            // Re-normalize overwritten current state to Absolute White POV
            played_eval = deep_played_eval;
            opponent_best_move = deep_opp_best;
            pv = deep_pv;
            multi_pv_evals = deep_multi
                .into_iter()
                .map(|v| engine_to_white_pov(v, is_white_to_move_after_play))
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
                // best-move mate from the previous position (White POV)
                class_played_mate, 
                // mate from the played move (White POV)
            );

        let positive_loss =
            current_win_loss.max(0.0);

        // Increments move counter and move tally
        if ply_count % 2 != 0 {
            white_win_loss += positive_loss;
            white_moves += 1;
            match classification {
                MoveBadge::Brilliant => {
                    move_counts_white.brilliant +=
                        1
                }
                MoveBadge::Great => {
                    move_counts_white.great += 1
                }
                MoveBadge::Best => {
                    move_counts_white.best += 1
                }
                MoveBadge::Excellent => {
                    move_counts_white.excellent +=
                        1
                }
                MoveBadge::Good => {
                    move_counts_white.good += 1
                }
                MoveBadge::Inaccuracy => {
                    move_counts_white
                        .inaccuracy += 1
                }
                MoveBadge::Mistake => {
                    move_counts_white.mistake += 1
                }
                MoveBadge::Blunder => {
                    move_counts_white.blunder += 1
                }
                MoveBadge::Miss => {
                    move_counts_white.miss += 1
                }
                MoveBadge::Book => {
                    move_counts_white.book += 1
                }
                MoveBadge::Forced => {
                    move_counts_white.forced += 1
                }
            }
        } else {
            black_win_loss += positive_loss;
            black_moves += 1;
            match classification {
                MoveBadge::Brilliant => {
                    move_counts_black.brilliant +=
                        1
                }
                MoveBadge::Great => {
                    move_counts_black.great += 1
                }
                MoveBadge::Best => {
                    move_counts_black.best += 1
                }
                MoveBadge::Excellent => {
                    move_counts_black.excellent +=
                        1
                }
                MoveBadge::Good => {
                    move_counts_black.good += 1
                }
                MoveBadge::Inaccuracy => {
                    move_counts_black
                        .inaccuracy += 1
                }
                MoveBadge::Mistake => {
                    move_counts_black.mistake += 1
                }
                MoveBadge::Blunder => {
                    move_counts_black.blunder += 1
                }
                MoveBadge::Miss => {
                    move_counts_black.miss += 1
                }
                MoveBadge::Book => {
                    move_counts_black.book += 1
                }
                MoveBadge::Forced => {
                    move_counts_black.forced += 1
                }
            }
        }

        // Construct AnalyzedMove data
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

        // Collect the analyzed move
        analyzed_moves_collection
            .push(analyzed_move);

        // Emit progress
        let progress = AnalysisProgress {
            current_ply: ply_count,
            total_plies,
        };
        app.emit("analysis-progress", progress)
            .map_err(|e| e.to_string())?;

        // `prev_best_mate` must come from the best-move analysis of the
        // current position (i.e. what the opponent will face), which is the
        // evaluation Stockfish just returned for `current_pos_cmd` and that is
        // `played_eval` negated into White POV from the opponent's side.
        // The engine returns scores relative to the side to move, so after
        // the played move it is the opponent's turn; we negate to stay in
        // Absolute White POV before storing.
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
        move_counts_black,
        move_counts_white,
        metadata,
        moves: analyzed_moves_collection,
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
    best_move_san: &str,
    prev_eval: i32,     // White POV
    normalized_cp: i32, // White POV
    prev_win_loss: f64,
    multi_pv_evals: &[i32], // White POV
    ply_count: u32,
    is_book_flag: bool,
    is_forced_move: bool,
    class_best_mate: Option<i32>, // White POV
    class_played_mate: Option<i32>, // White POV
) -> (MoveBadge, f64) {
    let is_white_moving = ply_count % 2 != 0;

    let class_prev_eval = white_to_moving_pov(
        prev_eval,
        is_white_moving,
    );
    let class_played_eval = white_to_moving_pov(
        normalized_cp,
        is_white_moving,
    );

    let normalized_multi_pv: Vec<i32> =
        multi_pv_evals
            .iter()
            .map(|&v| {
                white_to_moving_pov(
                    v,
                    is_white_moving,
                )
            })
            .collect();

    let class_best_eval = normalized_multi_pv
        .first()
        .copied()
        .unwrap_or(class_prev_eval);

    // Shift mate scores into Moving Player POV.
    // Positive = moving player is delivering mate in N.
    // Negative = moving player is getting mated in N.
    let best_mate_pov =
        class_best_mate.map(|m| {
            white_to_moving_pov(
                m,
                is_white_moving,
            )
        });
    let played_mate_pov =
        class_played_mate.map(|m| {
            white_to_moving_pov(
                m,
                is_white_moving,
            )
        });

    // Derive is_delivering_mate / is_getting_mated from the mate scores so the
    // classify fallback paths have accurate flags rather than always being false.
    let is_delivering_mate =
        played_mate_pov.map_or(false, |m| m > 0);
    let is_getting_mated =
        played_mate_pov.map_or(false, |m| m < 0);

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
    // Is used to check for hanging pieces.
    let current_pos_opt =
        Fen::from_ascii(fen.as_bytes())
            .ok()
            .and_then(|f| {
                f.into_position::<Chess>(
                    CastlingMode::Standard,
                )
                .ok()
            });

    let played_move =
        prev_pos.as_ref().and_then(|pos| {
            San::from_ascii(san.as_bytes())
                .ok()
                .and_then(|s| s.to_move(pos).ok())
        });

    // Checks if the move is a sacrifice
    let is_losing_material_flag = match (
        prev_pos.as_ref(),
        current_pos_opt.as_ref(),
        played_move.as_ref(),
    ) {
        (Some(prev), Some(current), Some(mv)) => {
            let color = if is_white_moving {
                shakmaty::Color::White
            } else {
                shakmaty::Color::Black
            };
            is_material_sacrifice(
                prev, current, mv, color,
            )
        }
        _ => false,
    };

    let prev_target = get_target_square(prev_san);
    let current_target = get_target_square(san);

    // During a queen to queen capture, taking their Queen back is mathematically the
    // "Best" move (prevents a -9.0 drop) and in most cases it is technicaly a "Great"
    // move because not recapturing usually just loses.
    // However, this requires zero calculation from the player. We explicitly flag
    // immediate recaptures on the same square so the classifier degrades them
    // from "Great" down to "Best"
    let is_obvious_recapture = prev_target
        .is_some()
        && current_target.is_some()
        && prev_target == current_target
        && san.contains('x');

    let is_best_engine_move =
        san == best_move_san;

    // Construct classification arguments
    let classify_args = ClassifyArgs {
        is_book: is_book_flag,
        prev_eval: class_prev_eval,
        played_eval: class_played_eval,
        prev_best_eval: class_best_eval,
        multi_pv_evals: &normalized_multi_pv,
        is_losing_material:
            is_losing_material_flag,
        is_obvious_recapture,
        prev_win_loss,
        is_forced_move,
        is_best_engine_move,
        is_delivering_mate,
        is_getting_mated,
        best_mate: best_mate_pov,
        played_mate: played_mate_pov,
    };

    classify(classify_args)
}
