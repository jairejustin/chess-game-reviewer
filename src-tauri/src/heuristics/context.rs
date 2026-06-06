use crate::heuristics::classify::{
    classify, ClassifyArgs,
};
use crate::heuristics::see::{
    get_target_square, is_material_sacrifice,
};
use crate::models::game::MoveBadge;
use crate::uci::evaluation::white_to_moving_pov;

use shakmaty::san::San;
use shakmaty::{
    fen::Fen, CastlingMode, Chess, Position,
};

/// Sums non-pawn material points for both sides on the given board,
/// using standard piece values (N/B=3, R=5, Q=9, K excluded).
/// Used to detect endgame positions for the Brilliant move guard.
fn count_non_pawn_material(
    board: &shakmaty::Board,
) -> i32 {
    use shakmaty::Role;
    board
        .occupied()
        .into_iter()
        .filter_map(|sq| board.piece_at(sq))
        .map(|piece| match piece.role {
            Role::Knight | Role::Bishop => 3,
            Role::Rook => 5,
            Role::Queen => 9,
            _ => 0, // Pawns and Kings excluded
        })
        .sum()
}

/// Assembles all heuristic flags from the pre- and post-move board state,
/// normalizes evaluations into moving-player POV, and delegates to `classify`.
///
/// This is the per-ply bridge between the raw engine output in the pipeline
/// and the pure classification logic in `heuristics::classify`.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_move_context(
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

    // Endgame detection: sum all non-pawn material (both sides) on the pre-move
    // board. The threshold of 26 points corresponds to roughly two minor pieces
    // and one rook per side remaining
    const ENDGAME_MATERIAL_THRESHOLD: i32 = 26;
    let is_endgame = prev_pos
        .as_ref()
        .map(|pos| {
            count_non_pawn_material(pos.board())
                <= ENDGAME_MATERIAL_THRESHOLD
        })
        .unwrap_or(false);

    let prev_target = get_target_square(prev_san);
    let current_target = get_target_square(san);

    // During a queen to queen capture, taking their Queen back is mathematically the
    // "Best" move (prevents a -9.0 drop) and in most cases it is technically a "Great"
    // move because not recapturing usually just loses.
    // However, this requires zero calculation from the player. We explicitly flag
    // immediate recaptures on the same square so the classifier degrades them
    // from "Great" down to "Best".
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
        is_endgame,
    };

    classify(classify_args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::fen::Fen;
    use shakmaty::{CastlingMode, Chess};

    /// Parses a FEN string into a `Shakmaty::Chess` position.
    fn pos(fen_str: &str) -> Chess {
        Fen::from_ascii(fen_str.as_bytes())
            .expect("Invalid FEN")
            .into_position(CastlingMode::Standard)
            .expect("Invalid position")
    }

    /// `evaluate_move_context` but it runs classification directly from board state + logical hardcoded eval values,
    /// skipping most of the fields. Supply it with the centipawn scores as if the
    /// engine had returned them
    fn evaluate_position_context(
        prev_fen: &str,
        san: &str,
        post_fen: &str,
        prev_eval: i32, // moving player POV
        played_eval: i32, // moving player POV
        prev_best_eval: i32, // moving player POV
        multi_pv_evals: &[i32], // moving player POV
        ply: u32, // odd = white moving
        played_mate: Option<i32>,
        best_mate: Option<i32>,
    ) -> MoveBadge {
        use crate::heuristics::classify::{
            classify, ClassifyArgs,
        };
        use crate::heuristics::see::is_material_sacrifice;
        use shakmaty::san::San;

        let is_delivering_mate =
            played_mate.map_or(false, |m| m > 0);

        let is_white_moving = ply % 2 != 0;

        let prev_pos = pos(prev_fen);
        let post_pos = pos(post_fen);

        let played_move =
            San::from_ascii(san.as_bytes())
                .expect("Invalid SAN")
                .to_move(&prev_pos)
                .expect("Illegal move");

        let color = if is_white_moving {
            shakmaty::Color::White
        } else {
            shakmaty::Color::Black
        };

        let is_losing_material =
            is_material_sacrifice(
                &prev_pos,
                &post_pos,
                &played_move,
                color,
            );

        let is_endgame = count_non_pawn_material(
            prev_pos.board(),
        ) <= 26;

        let classify_args = ClassifyArgs {
            prev_eval,
            played_eval,
            prev_best_eval,
            multi_pv_evals,
            is_losing_material,
            is_endgame,
            is_delivering_mate,
            played_mate,
            best_mate,
            ..Default::default()
        };

        classify(classify_args).0
    }

    #[test]
    fn queen_sac_for_mate_in_3_is_brilliant() {
        let badge = evaluate_position_context(
            "rnb1kb1r/pp3ppp/2p5/4q3/4n3/3Q4/PPPB1PPP/2KR1BNR w kq - 0 1",
            "Qd8+",
            "rnbQkb1r/pp3ppp/2p5/4q3/4n3/8/PPPB1PPP/2KR1BNR b kq - 0 1",
            100,
            10000,
            10000,
            &[10000, 100],
            9,
            Some(3),
            Some(3),
        );
        assert_eq!(badge, MoveBadge::Brilliant);
    }

    #[test]
    fn test_count_non_pawn_material_values() {
        // Starting position: 62 total
        let starting_pos = pos("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            count_non_pawn_material(
                starting_pos.board()
            ),
            62
        );

        // Endgame threshold position (<= 26).
        // 1 rook (5) + 1 knight (3) per side = 16 total.
        let endgame_pos = pos("8/4k3/8/2n1r3/3R1N2/8/3K4/8 w - - 0 1");
        assert_eq!(
            count_non_pawn_material(
                endgame_pos.board()
            ),
            16
        );

        // King and pawns only should total 0
        // (they are excluded from the endgame material tally)
        let pawn_endgame = pos(
            "8/4k3/8/3p4/4P3/8/3K4/8 w - - 0 1",
        );
        assert_eq!(
            count_non_pawn_material(
                pawn_endgame.board()
            ),
            0
        );
    }
}
