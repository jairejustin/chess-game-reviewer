use crate::math::calculate_win_percent;
use crate::models::game::MoveBadge;

/// Struct containing all context needed to classify a chess move.
/// If we invent new heuristics (like `is_endgame`), we can add them here without breaking older tests.
pub struct ClassifyArgs<'a> {
    pub is_book: bool,
    pub prev_eval: i32,
    pub played_eval: i32,
    pub best_move_eval: i32,
    pub multi_pv_evals: &'a [i32],
    pub is_sacrifice: bool,
    pub is_obvious_recapture: bool,
    pub prev_win_loss: f64,
    pub is_forced_move: bool,
}

impl<'a> Default for ClassifyArgs<'a> {
    fn default() -> Self {
        Self {
            is_book: false,
            prev_eval: 0,
            played_eval: 0,
            best_move_eval: 0,
            multi_pv_evals: &[],
            is_sacrifice: false,
            is_obvious_recapture: false,
            prev_win_loss: 0.0,
            is_forced_move: false,
        }
    }
}

pub fn classify(
    args: ClassifyArgs,
) -> (MoveBadge, f64) {
    // Book moves are established human theory. An engine might hate a
    // deep Sicilian Najdorf line at depth 15, but it is objectively correct.
    // Therefore, book moves bypass all mathematical evaluation.
    if args.is_book {
        return (MoveBadge::Book, 0.0);
    }

    let delta =
        args.best_move_eval - args.played_eval;
    let win_loss =
        calculate_win_percent(args.prev_eval)
            - calculate_win_percent(
                args.played_eval,
            );

    if args.is_forced_move {
        return (MoveBadge::Forced, win_loss);
    }

    // Base Classification
    let mut classification = if args
        .prev_eval
        .abs()
        > 1000
        && args.played_eval.abs() > 1000
    {
        if delta == 0 {
            MoveBadge::Best
        } else {
            MoveBadge::Excellent
        }
    } else if delta == 0 {
        MoveBadge::Best
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

    // THE MISS: If the opponent's previous move resulted in a large drop for them (>= 5%),
    // but our current move gives 70% to 140% of that advantage right back, we didn't just play
    // poorly. We "Missed" the punishment for their mistake.
    if args.prev_win_loss >= 5.0
        && win_loss >= (args.prev_win_loss * 0.7)
        && win_loss <= (args.prev_win_loss * 1.4)
    {
        classification = MoveBadge::Miss;
    }

    // GREAT MOVE: If the played move matches the best move, AND the second-best move
    // creates an 8.5% win probability drop, this was an "Only Move". The player navigated
    // a tightrope where any other choice would have lost the advantage.
    if (classification == MoveBadge::Best
        || classification == MoveBadge::Excellent)
        && args.prev_eval.abs() <= 1000
        && is_great_move(
            args.played_eval,
            args.best_move_eval,
            args.multi_pv_evals,
            args.is_obvious_recapture,
        )
    {
        classification = MoveBadge::Great;
    }

    // BRILLIANT MOVE: we use Static Exchange Evaluation (SEE) to confirm a piece is *actually* hanging.
    // If SEE says it's hanging, but the engine eval didn't drop, it's a sound sacrifice.
    if win_loss < 5.0
        && delta <= 40
        && args.is_sacrifice
        && args.played_eval > -200
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

    #[test]
    fn excellent_when_position_decided_but_not_best_move(
    ) {
        let args = ClassifyArgs {
            prev_eval: 1200,
            played_eval: 1100,
            best_move_eval: 1200,
            multi_pv_evals: &[1200, 1100],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Excellent
        );
    }

    #[test]
    fn best_when_position_decided_and_played_engine_line(
    ) {
        let args = ClassifyArgs {
            prev_eval: 1200,
            played_eval: 1200,
            best_move_eval: 1200,
            multi_pv_evals: &[1200, 1100],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Best
        );
    }

    #[test]
    fn brilliant_when_sacrifice_confirmed_by_engine(
    ) {
        let args = ClassifyArgs {
            prev_eval: 300,
            played_eval: 280,
            best_move_eval: 290,
            multi_pv_evals: &[290, 20, -50],
            is_sacrifice: true,
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Brilliant
        );
    }

    #[test]
    fn not_brilliant_without_material_sacrifice()
    {
        let args = ClassifyArgs {
            prev_eval: 50,
            played_eval: 45,
            best_move_eval: 80,
            multi_pv_evals: &[80, 60, 40],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Excellent
        );
    }

    #[test]
    fn not_brilliant_if_position_is_lost() {
        let args = ClassifyArgs {
            prev_eval: -280,
            played_eval: -300,
            best_move_eval: -300,
            multi_pv_evals: &[-300, -500],
            is_sacrifice: true,
            ..Default::default()
        };
        assert_ne!(
            classify(args).0,
            MoveBadge::Brilliant
        );
    }

    #[test]
    fn great_when_only_move_maintaining_equality()
    {
        let args = ClassifyArgs {
            prev_eval: 30,
            played_eval: 28,
            best_move_eval: 30,
            multi_pv_evals: &[30, -150, -200],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Great
        );
    }

    #[test]
    fn test_miss_classification() {
        let args = ClassifyArgs {
            prev_eval: 100,
            played_eval: 0,
            best_move_eval: 100,
            multi_pv_evals: &[100, 80],
            prev_win_loss: 10.0,
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Miss
        );
    }

    #[test]
    fn blunder_by_win_percent_drop() {
        let args = ClassifyArgs {
            prev_eval: 400,
            played_eval: 0,
            best_move_eval: 400,
            multi_pv_evals: &[400, 300],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Blunder
        );
    }

    #[test]
    fn mistake_by_win_percent_drop() {
        let args = ClassifyArgs {
            prev_eval: 200,
            played_eval: 0,
            best_move_eval: 200,
            multi_pv_evals: &[200, 100],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Mistake
        );
    }

    #[test]
    fn inaccuracy_by_win_percent_drop() {
        let args = ClassifyArgs {
            prev_eval: 150,
            played_eval: 70,
            best_move_eval: 150,
            multi_pv_evals: &[150, 70],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Inaccuracy
        );
    }

    #[test]
    fn good_by_win_percent_drop() {
        let args = ClassifyArgs {
            prev_eval: 100,
            played_eval: 50,
            best_move_eval: 100,
            multi_pv_evals: &[100, 50],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Good
        );
    }

    #[test]
    fn excellent_by_win_percent_drop() {
        let args = ClassifyArgs {
            prev_eval: 50,
            played_eval: 40,
            best_move_eval: 50,
            multi_pv_evals: &[50, 40],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Excellent
        );
    }

    #[test]
    fn best_when_played_equals_best_move() {
        let args = ClassifyArgs {
            prev_eval: 30,
            played_eval: 28,
            best_move_eval: 28,
            multi_pv_evals: &[28, 10, -20],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Best
        );
    }

    #[test]
    fn forced_move() {
        let args = ClassifyArgs {
            prev_eval: 30,
            played_eval: 28,
            best_move_eval: 28,
            multi_pv_evals: &[28, 10, -20],
            is_forced_move: true,
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Forced
        );
    }

    #[test]
    fn book_move_bypasses_math() {
        // Even if the move loses 500 centipawns, if it's theory, it's a Book move.
        let args = ClassifyArgs {
            prev_eval: 500,
            played_eval: -100,
            is_book: true,
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Book
        );
    }
}
