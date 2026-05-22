use crate::math::calculate_win_percent;
use crate::models::game::MoveBadge;

pub fn classify(
    prev_eval: i32,
    played_eval: i32,
    best_move_eval: i32,
    multi_pv_evals: &[i32],
    is_sacrifice: bool,
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

    // Base Classification
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

    // Brilliant Move: SEE confirmed a hanging piece, and the engine eval is still good
    if win_loss < 5.0
        && delta <= 40
        && is_sacrifice
        && played_eval > -200
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
        let (badge, _) = classify(
            1200,
            1100,
            1200,
            &[1200, 1100],
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Excellent);
    }

    #[test]
    fn best_when_position_decided_and_played_engine_line(
    ) {
        let (badge, _) = classify(
            1200,
            1200,
            1200,
            &[1200, 1100],
            false,
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
        // is_sacrifice = true
        let (badge, _) = classify(
            300,
            280,
            290,
            &[290, 20, -50],
            true,
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
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Excellent);
    }

    #[test]
    fn not_brilliant_if_position_is_lost() {
        // Eval is -300, so sacrificing is just desperation
        let (badge, _) = classify(
            -280,
            -300,
            -300,
            &[-300, -500],
            true,
            false,
            0.0,
            false,
        );
        assert_ne!(badge, MoveBadge::Brilliant);
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
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Great);
    }

    #[test]
    fn not_great_if_obvious_recapture() {
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
        let (badge, _) = classify(
            100,
            0,
            100,
            &[100, 80],
            false,
            false,
            10.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Miss);
    }

    // --- Win Percentage Drop Thresholds (Expected Points Model) ---

    #[test]
    fn blunder_by_win_percent_drop() {
        let (badge, _) = classify(
            400,
            0,
            400,
            &[400, 300],
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Blunder);
    }

    #[test]
    fn mistake_by_win_percent_drop() {
        let (badge, _) = classify(
            200,
            0,
            200,
            &[200, 100],
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Mistake);
    }

    #[test]
    fn inaccuracy_by_win_percent_drop() {
        let (badge, _) = classify(
            150,
            70,
            150,
            &[150, 70],
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Inaccuracy);
    }

    #[test]
    fn good_by_win_percent_drop() {
        let (badge, _) = classify(
            100,
            50,
            100,
            &[100, 50],
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Good);
    }

    #[test]
    fn excellent_by_win_percent_drop() {
        let (badge, _) = classify(
            50,
            40,
            50,
            &[50, 40],
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Excellent);
    }

    #[test]
    fn best_when_played_equals_best_move() {
        let (badge, _) = classify(
            30,
            28,
            28,
            &[28, 10, -20],
            false,
            false,
            0.0,
            false,
        );
        assert_eq!(badge, MoveBadge::Best);
    }

    #[test]
    fn forced_move() {
        let (badge, _) = classify(
            30,
            28,
            28,
            &[28, 10, -20],
            false,
            false,
            0.0,
            true,
        );
        assert_eq!(badge, MoveBadge::Forced);
    }
}
