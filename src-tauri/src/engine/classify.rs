use crate::math::calculate_win_percent;
use crate::models::game::MoveBadge;

pub fn classify(
    prev_eval: i32,
    played_eval: i32,
    best_move_eval: i32,
    multi_pv_evals: &[i32],
    material_delta: i32,
    is_obvious_recapture: bool,
    prev_win_loss: f64,
    is_forced_move: bool,
) -> (MoveBadge, f64) {
    let delta = best_move_eval - played_eval;
    let win_loss =
        calculate_win_percent(prev_eval)
            - calculate_win_percent(played_eval);

    if is_forced_move {
        return (MoveBadge::Forced, win_loss);
    }

    // 1. Base Classification
    let mut classification = if prev_eval.abs()
        > 1000
        && played_eval.abs() > 1000
    {
        if delta == 0 {
            MoveBadge::Best
        } else {
            MoveBadge::Excellent
        }
    } else if delta == 0 {
        MoveBadge::Best // Match exact engine line, ignoring depth fluctuations
    } else {
        match win_loss {
            w if w >= 20.0 => MoveBadge::Blunder,
            w if w >= 10.0 => MoveBadge::Mistake,
            w if w >= 5.0 => {
                MoveBadge::Inaccuracy
            }
            w if w >= 2.0 => MoveBadge::Good,
            w if w > 0.0 => MoveBadge::Excellent,
            _ => MoveBadge::Best,
        }
    };

    // Upgrades & Overrides

    // The Miss: Opponent blundered (>= 5%), and we gave 70-140% of it right back
    if prev_win_loss >= 5.0
        && win_loss >= (prev_win_loss * 0.7)
        && win_loss <= (prev_win_loss * 1.4)
    {
        classification = MoveBadge::Miss;
    }

    // Great Move: 8.5% threshold + Recapture Filter
    if (classification == MoveBadge::Best
        || classification == MoveBadge::Excellent)
        && prev_eval.abs() <= 1000
        && is_great_move(
            played_eval,
            best_move_eval,
            multi_pv_evals,
            is_obvious_recapture,
        )
    {
        classification = MoveBadge::Great;
    }

    // Brilliant Move: Material sacrifice of at least 2 points
    if win_loss < 5.0
        && delta <= 40
        && material_delta <= -2
    {
        classification = MoveBadge::Brilliant;
    }

    (classification, win_loss)
}

pub fn is_great_move(
    played_eval: i32,
    best_eval: i32,
    multi_pv_evals: &[i32],
    is_obvious_recapture: bool,
) -> bool {
    if is_obvious_recapture
        || multi_pv_evals.len() < 2
    {
        return false;
    }

    let played_best =
        (best_eval - played_eval).abs() <= 15;
    let win_loss_to_second_best =
        calculate_win_percent(best_eval)
            - calculate_win_percent(
                multi_pv_evals[1],
            );

    played_best && win_loss_to_second_best >= 8.5
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
        let (badge, _) = classify(
            1200,
            1100,
            1200,
            &[1200, 1100],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Excellent);
    }

    #[test]
    fn best_when_position_decided_and_played_engine_line(
    ) {
        // Delta is 0, they matched the exact top line in a winning position.
        let (badge, _) = classify(
            1200,
            1200,
            1200,
            &[1200, 1100],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Best);
    }

    // --- Brilliant Move Heuristic ---

    #[test]
    fn brilliant_when_sacrifice_confirmed_by_engine(
    ) {
        // -3 material is <= -2
        let (badge, _) = classify(
            300,
            280,
            290,
            &[290, 20, -50],
            -3,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Brilliant);
    }

    #[test]
    fn not_brilliant_without_material_sacrifice()
    {
        let (badge, _) = classify(
            50,
            45,
            80,
            &[80, 60, 40],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Excellent);
    }

    // --- Great Move Heuristic ---

    #[test]
    fn great_when_only_move_maintaining_equality()
    {
        let (badge, _) = classify(
            30,
            28,
            30,
            &[30, -150, -200],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Great);
    }

    #[test]
    fn not_great_if_obvious_recapture() {
        // Even with a massive drop, if is_obvious_recapture is true, it shouldn't be Great
        assert!(!is_great_move(
            30,
            30,
            &[30, -150],
            true
        ));
    }

    #[test]
    fn is_great_move_false_with_insufficient_pv_lines(
    ) {
        assert!(!is_great_move(
            30,
            30,
            &[30],
            false
        ));
    }

    #[test]
    fn is_great_move_false_when_played_not_near_best(
    ) {
        assert!(!is_great_move(
            10,
            30,
            &[30, -150],
            false
        ));
    }

    #[test]
    fn is_great_move_false_when_no_steep_dropoff()
    {
        assert!(!is_great_move(
            28,
            30,
            &[30, 25, 20],
            false
        ));
    }

    // --- The Miss Classification ---

    #[test]
    fn test_miss_classification() {
        // prev_win_loss = 10.0 (opponent blundered 10%)
        // eval dropping from 100 to 0 is approx a 9% win loss for us.
        // 9% is within 70% to 140% of the 10% the opponent handed us.
        let (badge, _) = classify(
            100,
            0,
            100,
            &[100, 80],
            0,
            false,
            10.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Miss);
    }

    // --- Win Percentage Drop Thresholds (Expected Points Model) ---

    #[test]
    fn blunder_by_win_percent_drop() {
        // 400 (~82.6%) to 0 (50%) -> ~32.6% drop (>= 20%)
        let (badge, _) = classify(
            400,
            0,
            400,
            &[400, 300],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Blunder);
    }

    #[test]
    fn mistake_by_win_percent_drop() {
        // 200 (~68.1%) to 0 (50%) -> ~18.1% drop (>= 10%)
        let (badge, _) = classify(
            200,
            0,
            200,
            &[200, 100],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Mistake);
    }

    #[test]
    fn inaccuracy_by_win_percent_drop() {
        // 150 (~63.7%) to 70 (~56.4%) -> ~7.3% drop (>= 5%)
        let (badge, _) = classify(
            150,
            70,
            150,
            &[150, 70],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Inaccuracy);
    }

    #[test]
    fn good_by_win_percent_drop() {
        // 100 (~59.1%) to 50 (~54.6%) -> ~4.5% drop (>= 2%)
        let (badge, _) = classify(
            100,
            50,
            100,
            &[100, 50],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Good);
    }

    #[test]
    fn excellent_by_win_percent_drop() {
        // 50 (~54.6%) to 40 (~53.7%) -> ~0.9% drop (> 0%)
        let (badge, _) = classify(
            50,
            40,
            50,
            &[50, 40],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Excellent);
    }

    #[test]
    fn best_when_played_equals_best_move() {
        // Delta == 0 should always be Best, ignoring minor engine depth variations
        let (badge, _) = classify(
            30,
            28,
            28,
            &[28, 10, -20],
            0,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Best);
    }

    #[test]
    fn forced_move() {
        // Delta == 0 should always be Best, ignoring minor engine depth variations
        let (badge, _) = classify(
            30,
            28,
            28,
            &[28, 10, -20],
            0,
            false,
            0.0,
            true,
        );
        assert_eq!(badge, MoveBadge::Forced);
    }
}
