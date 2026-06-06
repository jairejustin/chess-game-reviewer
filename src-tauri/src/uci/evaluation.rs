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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_to_white_pov_preserves_score_when_white_to_move(
    ) {
        assert_eq!(
            engine_to_white_pov(300, true),
            300
        );
        assert_eq!(
            engine_to_white_pov(-150, true),
            -150
        );
        assert_eq!(
            engine_to_white_pov(0, true),
            0
        );
    }

    #[test]
    fn engine_to_white_pov_negates_score_when_black_to_move(
    ) {
        assert_eq!(
            engine_to_white_pov(300, false),
            -300
        );
        assert_eq!(
            engine_to_white_pov(-150, false),
            150
        );
        assert_eq!(
            engine_to_white_pov(0, false),
            0
        );
    }

    #[test]
    fn white_to_moving_pov_preserves_score_for_white(
    ) {
        assert_eq!(
            white_to_moving_pov(400, true),
            400
        );
        assert_eq!(
            white_to_moving_pov(-200, true),
            -200
        );
    }

    #[test]
    fn white_to_moving_pov_negates_score_for_black(
    ) {
        assert_eq!(
            white_to_moving_pov(400, false),
            -400
        );
        assert_eq!(
            white_to_moving_pov(-200, false),
            200
        );
    }

    #[test]
    fn pov_conversions_are_inverse_of_each_other()
    {
        let original = 250;
        let white_pov =
            engine_to_white_pov(original, false);
        let back =
            white_to_moving_pov(white_pov, false);
        assert_eq!(back, original);
    }

    #[test]
    fn extract_cp_returns_value_for_cp_eval() {
        assert_eq!(
            extract_cp(&Evaluation::Cp(120)),
            120
        );
        assert_eq!(
            extract_cp(&Evaluation::Cp(-75)),
            -75
        );
        assert_eq!(
            extract_cp(&Evaluation::Cp(0)),
            0
        );
    }

    #[test]
    fn extract_cp_clamps_positive_mate_to_10000()
    {
        assert_eq!(
            extract_cp(&Evaluation::Mate(3)),
            10000
        );
        assert_eq!(
            extract_cp(&Evaluation::Mate(1)),
            10000
        );
    }

    #[test]
    fn extract_cp_clamps_negative_mate_to_neg_10000(
    ) {
        assert_eq!(
            extract_cp(&Evaluation::Mate(-2)),
            -10000
        );
        assert_eq!(
            extract_cp(&Evaluation::Mate(-5)),
            -10000
        );
    }

    #[test]
    fn extract_mate_returns_some_for_mate_eval() {
        assert_eq!(
            extract_mate(&Evaluation::Mate(3)),
            Some(3)
        );
        assert_eq!(
            extract_mate(&Evaluation::Mate(-2)),
            Some(-2)
        );
    }

    #[test]
    fn extract_mate_returns_none_for_cp_eval() {
        assert_eq!(
            extract_mate(&Evaluation::Cp(200)),
            None
        );
        assert_eq!(
            extract_mate(&Evaluation::Cp(0)),
            None
        );
    }
}
