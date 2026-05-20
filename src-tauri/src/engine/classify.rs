use crate::models::game::MoveBadge;

pub fn classify(
    prev_eval: i32,
    played_eval: i32,
    best_move_eval: i32,
    multi_pv_evals: &[i32],
    material_delta: i32,
) -> MoveBadge {
    let delta = best_move_eval - played_eval;

    // If the position is totally decided (+/- 10 pawns), suppress noise.
    // Only award "Best" if they exactly matched the top engine line (delta == 0).
    if prev_eval.abs() > 1000
        && played_eval.abs() > 1000
    {
        if delta == 0 {
            return MoveBadge::Best;
        } else {
            return MoveBadge::Excellent;
        }
    }

    let win_loss =
        calculate_win_percent(prev_eval)
            - calculate_win_percent(played_eval);

    // Brilliant Move
    if win_loss < 5.0
        && delta <= 40
        && material_delta < 0
    {
        return MoveBadge::Brilliant;
    }

    // Base Classification by Win Percentage
    let mut classification: MoveBadge =
        match win_loss {
            w if w >= 20.0 => MoveBadge::Blunder,
            w if w >= 10.0 => MoveBadge::Mistake,
            w if w >= 5.0 => {
                MoveBadge::Inaccuracy
            }

            // Base Classification by Evaluation Delta
            _ => match delta {
                d if d <= 15 => MoveBadge::Best,
                d if d <= 40 => {
                    MoveBadge::Excellent
                }
                d if d <= 80 => MoveBadge::Good,
                _ => MoveBadge::Inaccuracy,
            },
        };

    // Great Move Override
    if (classification == MoveBadge::Best
        || classification == MoveBadge::Excellent)
        && is_great_move(
            played_eval,
            best_move_eval,
            multi_pv_evals,
        )
    {
        classification = MoveBadge::Great;
    }

    classification
}

pub fn is_great_move(
    played_eval: i32,
    best_eval: i32,
    multi_pv_evals: &[i32],
) -> bool {
    if multi_pv_evals.len() < 2 {
        return false;
    }

    // Checks if the played move is virtually the best move, and
    // there's a steep evaluation drop-off to the second-best move
    (best_eval - played_eval).abs() <= 15
        && (multi_pv_evals[0] - multi_pv_evals[1])
            >= 100
}

fn calculate_win_percent(cp: i32) -> f64 {
    50.0 + 50.0
        * (2.0
            / (1.0
                + (-0.00368 * cp as f64).exp())
            - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Early Returns (Noise Suppressor) ---

    #[test]
    fn excellent_when_position_decided_but_not_best_move(
    ) {
        // Delta is 100, meaning they missed the absolute best mate line,
        // but the position is still totally winning (+1100).
        assert_eq!(
            classify(
                1200,
                1100,
                1200,
                &[1200, 1100],
                0
            ),
            MoveBadge::Excellent
        );
    }

    #[test]
    fn best_when_position_decided_and_played_engine_line(
    ) {
        // Delta is 0, they matched the exact top line in a winning position.
        assert_eq!(
            classify(
                1200,
                1200,
                1200,
                &[1200, 1100],
                0
            ),
            MoveBadge::Best
        );
    }

    // --- Brilliant Move Heuristic ---

    #[test]
    fn brilliant_when_sacrifice_confirmed_by_engine(
    ) {
        assert_eq!(
            classify(
                300,
                280,
                290,
                &[290, 20, -50],
                -3
            ),
            MoveBadge::Brilliant
        );
    }

    #[test]
    fn not_brilliant_without_material_sacrifice()
    {
        assert_eq!(
            classify(
                50,
                45,
                80,
                &[80, 60, 40],
                0
            ),
            MoveBadge::Excellent
        );
    }

    // --- Great Move Heuristic ---

    #[test]
    fn great_when_only_move_maintaining_equality()
    {
        assert_eq!(
            classify(
                30,
                28,
                30,
                &[30, -150, -200],
                0
            ),
            MoveBadge::Great
        );
    }

    #[test]
    fn is_great_move_false_with_insufficient_pv_lines(
    ) {
        assert!(!is_great_move(30, 30, &[30]));
    }

    #[test]
    fn is_great_move_false_when_played_not_near_best(
    ) {
        assert!(!is_great_move(
            10,
            30,
            &[30, -150]
        ));
    }

    #[test]
    fn is_great_move_false_when_no_steep_dropoff()
    {
        assert!(!is_great_move(
            28,
            30,
            &[30, 25, 20]
        ));
    }

    // --- Win Percentage Drop Thresholds ---

    #[test]
    fn blunder_by_win_percent_drop() {
        assert_eq!(
            classify(400, 0, 400, &[400, 300], 0),
            MoveBadge::Blunder
        );
    }

    #[test]
    fn mistake_by_win_percent_drop() {
        assert_eq!(
            classify(200, 0, 200, &[200, 100], 0),
            MoveBadge::Mistake
        );
    }

    #[test]
    fn inaccuracy_by_win_percent_drop() {
        assert_eq!(
            classify(100, 0, 100, &[100, 50], 0),
            MoveBadge::Inaccuracy
        );
    }

    // --- Delta Fallback Thresholds (Now testing 'Good') ---

    #[test]
    fn best_when_played_equals_best_move() {
        assert_eq!(
            classify(
                30,
                28,
                28,
                &[28, 10, -20],
                0
            ),
            MoveBadge::Best
        );
    }

    #[test]
    fn excellent_by_delta() {
        assert_eq!(
            classify(50, 20, 50, &[50, 40], 0),
            MoveBadge::Excellent
        );
    }
    
    #[test]
    fn good_by_delta() {
        // Delta is 60 (<= 80).
        // By testing at a higher eval (+900 to +840), the win probability drop is < 1%,
        // which bypasses the 5% Inaccuracy trigger and correctly tests the delta match block.
        assert_eq!(
            classify(
                900,
                840,
                900,
                &[900, 840],
                0
            ),
            MoveBadge::Good
        );
    }

    #[test]
    fn inaccuracy_by_delta() {
        // Win probability drop from +900 to +800 is almost 0%.
        // But delta is 100 (> 80).
        assert_eq!(
            classify(
                900,
                800,
                900,
                &[900, 800],
                0
            ),
            MoveBadge::Inaccuracy
        );
    }

    // --- Math Utilities ---

    #[test]
    fn calculate_win_percent_bounds() {
        let win_0 = calculate_win_percent(0);
        assert!((win_0 - 50.0).abs() < 0.1, "0 cp should be exactly 50% win probability");

        let win_high =
            calculate_win_percent(1000);
        assert!(win_high > 95.0, "High positive cp should yield near 100% win probability");

        let win_low =
            calculate_win_percent(-1000);
        assert!(win_low < 5.0, "High negative cp should yield near 0% win probability");
    }
}
