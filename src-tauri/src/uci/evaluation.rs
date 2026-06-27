use crate::uci::uci_engine::Evaluation;
use shakmaty::san::San;
use shakmaty::uci::UciMove;
use shakmaty::Chess;

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
                10_000
            } else {
                -10_000
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

/// Extracts a mate score and converts it to Absolute White POV in one step.
pub fn extract_mate_white_pov(
    eval: &Evaluation,
    is_white_to_move: bool,
) -> Option<i32> {
    extract_mate(eval).map(|m| {
        engine_to_white_pov(m, is_white_to_move)
    })
}

/// Normalises a raw multi-PV eval vector to Absolute White POV in one call.
pub fn normalize_multi_pv(
    evals: Vec<i32>,
    is_white_to_move: bool,
) -> Vec<i32> {
    evals
        .into_iter()
        .map(|v| {
            engine_to_white_pov(
                v,
                is_white_to_move,
            )
        })
        .collect()
}

/// Strips check (`+`) and mate (`#`) suffixes for bare move comparison.
///
/// Engines omit these suffixes from UCI output, so `get_san` / `San::from_move`
/// may produce `"Bb4+"` while a PGN source produces `"Bb4"` (or vice-versa).
/// Always compare through this function to avoid mismatches.
pub fn bare_san(san: &str) -> &str {
    san.trim_end_matches(['+', '#'])
}

/// Converts a UCI move string to SAN in the context of a given board position.
///
/// Returns the UCI string unchanged if the position is absent or the move
/// cannot be parsed / applied (illegal moves).
pub fn uci_to_san(
    uci_str: &str,
    pos: &Option<Chess>,
) -> String {
    if let Some(pos) = pos {
        if let Ok(uci) = UciMove::from_ascii(
            uci_str.as_bytes(),
        ) {
            if let Ok(m) = uci.to_move(pos) {
                return San::from_move(pos, m)
                    .to_string();
            }
        }
    }
    uci_str.to_string()
}

/// Converts a backend `Evaluation` into a frontend display string.
/// Negative mate scores render as `"-M3"` so the UI can distinguish losing mates.
pub fn format_eval(eval: Evaluation) -> String {
    match eval {
        Evaluation::Cp(cp) => {
            let score = cp as f32 / 100.0;
            if score >= 0.0 {
                format!("+{:.2}", score)
            } else {
                format!("{:.2}", score)
            }
        }
        Evaluation::Mate(m) => {
            if m >= 0 {
                format!("M{}", m)
            } else {
                format!("-M{}", m.abs())
            }
        }
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
            10_000
        );
        assert_eq!(
            extract_cp(&Evaluation::Mate(1)),
            10_000
        );
    }

    #[test]
    fn extract_cp_clamps_negative_mate_to_neg_10000(
    ) {
        assert_eq!(
            extract_cp(&Evaluation::Mate(-2)),
            -10_000
        );
        assert_eq!(
            extract_cp(&Evaluation::Mate(-5)),
            -10_000
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

    #[test]
    fn extract_mate_white_pov_returns_none_for_cp(
    ) {
        assert_eq!(
            extract_mate_white_pov(
                &Evaluation::Cp(300),
                true
            ),
            None
        );
    }

    #[test]
    fn extract_mate_white_pov_negates_when_black_to_move(
    ) {
        assert_eq!(
            extract_mate_white_pov(
                &Evaluation::Mate(2),
                false
            ),
            Some(-2)
        );
    }

    #[test]
    fn extract_mate_white_pov_preserves_when_white_to_move(
    ) {
        assert_eq!(
            extract_mate_white_pov(
                &Evaluation::Mate(3),
                true
            ),
            Some(3)
        );
    }

    #[test]
    fn normalize_multi_pv_negates_all_when_black_to_move(
    ) {
        assert_eq!(
            normalize_multi_pv(
                vec![100, -50],
                false
            ),
            vec![-100, 50]
        );
    }

    #[test]
    fn normalize_multi_pv_preserves_all_when_white_to_move(
    ) {
        assert_eq!(
            normalize_multi_pv(
                vec![100, -50],
                true
            ),
            vec![100, -50]
        );
    }

    #[test]
    fn bare_san_strips_check_suffix() {
        assert_eq!(bare_san("Bb4+"), "Bb4");
    }

    #[test]
    fn bare_san_strips_mate_suffix() {
        assert_eq!(bare_san("Qxf7#"), "Qxf7");
    }

    #[test]
    fn bare_san_leaves_plain_move_unchanged() {
        assert_eq!(bare_san("e4"), "e4");
        assert_eq!(bare_san("Nf3"), "Nf3");
    }

    #[test]
    fn format_eval_positive_cp_has_plus_prefix() {
        assert_eq!(
            format_eval(Evaluation::Cp(145)),
            "+1.45"
        );
    }

    #[test]
    fn format_eval_negative_cp_has_no_prefix() {
        assert_eq!(
            format_eval(Evaluation::Cp(-200)),
            "-2.00"
        );
    }

    #[test]
    fn format_eval_zero_cp_has_plus_prefix() {
        assert_eq!(
            format_eval(Evaluation::Cp(0)),
            "+0.00"
        );
    }

    #[test]
    fn format_eval_cp_rounds_to_two_decimal_places(
    ) {
        assert_eq!(
            format_eval(Evaluation::Cp(133)),
            "+1.33"
        );
        assert_eq!(
            format_eval(Evaluation::Cp(-7)),
            "-0.07"
        );
    }

    #[test]
    fn format_eval_positive_mate_renders_as_m_prefix(
    ) {
        assert_eq!(
            format_eval(Evaluation::Mate(3)),
            "M3"
        );
    }

    #[test]
    fn format_eval_mate_in_one_renders_correctly()
    {
        assert_eq!(
            format_eval(Evaluation::Mate(1)),
            "M1"
        );
    }

    #[test]
    fn format_eval_negative_mate_renders_with_dash_prefix(
    ) {
        assert_eq!(
            format_eval(Evaluation::Mate(-2)),
            "-M2"
        );
    }

    #[test]
    fn format_eval_negative_mate_in_one_renders_correctly(
    ) {
        assert_eq!(
            format_eval(Evaluation::Mate(-1)),
            "-M1"
        );
    }
}
