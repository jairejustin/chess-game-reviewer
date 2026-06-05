use crate::uci::uci_engine::Evaluation;

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
/// Mate scores are clamped to +-10000.
pub fn extract_cp(eval: &Evaluation) -> i32 {
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
pub fn extract_mate(
    eval: &Evaluation,
) -> Option<i32> {
    match eval {
        Evaluation::Mate(m) => Some(*m),
        Evaluation::Cp(_) => None,
    }
}
