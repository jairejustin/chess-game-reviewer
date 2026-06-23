use crate::heuristics::accuracy::calculate_win_percent;
use crate::models::game::MoveBadge;

/// A dependency injection struct holding all contextual data and normalized centipawn scores
/// needed to mathematically classify a move.
// If we invent new heuristics (like `is_endgame`), we can add them here without breaking older tests.
pub struct ClassifyArgs<'a> {
    pub is_book: bool,
    pub prev_eval: i32,
    pub played_eval: i32,
    pub prev_best_eval: i32,
    pub multi_pv_evals: &'a [i32],
    pub is_losing_material: bool,
    pub is_obvious_recapture: bool,
    pub prev_win_loss: f64,
    pub is_forced_move: bool,
    pub is_best_engine_move: bool,
    pub is_delivering_mate: bool,
    pub is_getting_mated: bool,
    /// Mate score from the best engine move, in the moving player's POV.
    /// Positive = the moving player is delivering mate in N moves.
    /// Negative = the moving player is getting mated in N moves.
    /// None = the best move does not lead to a forced mate.
    pub best_mate: Option<i32>,
    /// Mate score from the played move, in the moving player's POV.
    /// Positive = the moving player is delivering mate in N moves.
    /// Negative = the moving player is getting mated in N moves.
    /// None = the played move does not lead to a forced mate.
    pub played_mate: Option<i32>,
    /// True when total non-pawn material on the board (both sides combined)
    /// falls below the endgame threshold (~26 points using standard piece values).
    /// Computed from the pre-move board state in `evaluate_move_context`.
    pub is_endgame: bool,
    pub is_trivial_check_evasion: bool,
}

impl<'a> Default for ClassifyArgs<'a> {
    fn default() -> Self {
        Self {
            is_book: false,
            prev_eval: 0,
            played_eval: 0,
            prev_best_eval: 0,
            multi_pv_evals: &[],
            is_losing_material: false,
            is_obvious_recapture: false,
            prev_win_loss: 0.0,
            is_forced_move: false,
            is_best_engine_move: false,
            is_delivering_mate: false,
            is_getting_mated: false,
            best_mate: None,
            played_mate: None,
            is_endgame: false,
            is_trivial_check_evasion: false,
        }
    }
}

/// Classifies a move when both the best line and the played line are in a
/// forced-mate sequence, using the raw mate-in-N distances.
///
/// Both values are in the moving player's POV:
/// - Positive  = moving player is delivering mate (attacker).
/// - Negative  = moving player is getting mated (defender).
///
/// Returns `None` when the situation is not a pure mate-vs-mate comparison
fn classify_mate_vs_mate(
    best_mate: i32,
    played_mate: i32,
    is_best_engine_move: bool,
) -> Option<MoveBadge> {
    match (best_mate > 0, played_mate > 0) {
        // Attacker (positive mate values means moving player is mating)
        (true, true) => {
            if is_best_engine_move
                || played_mate <= best_mate
            {
                // Played the fastest (or equally fast) forced mate => Best
                Some(MoveBadge::Best)
            } else {
                // Played a checkmating move, but slower than optimal
                let delay =
                    played_mate - best_mate;
                Some(if delay <= 2 {
                    // Within 2 moves of the fastest line, close enough to call Excellent
                    MoveBadge::Excellent
                } else if delay <= 6 {
                    // Noticeably slower, but still a clean win
                    MoveBadge::Good
                } else {
                    // Seriously prolonged the win
                    MoveBadge::Inaccuracy
                })
            }
        }

        // Defender (negative mate values means opponent is mating)
        (false, false) => {
            // Both sides: engine says getting mated in |best_mate|,
            // played move gets mated in |played_mate|.
            // "More negative" = mated sooner, "less negative" = more resilient.
            if is_best_engine_move
                || played_mate <= best_mate
            {
                // Found the most resilient defense (or equally good) => Best
                Some(MoveBadge::Best)
            } else {
                // Allowed faster mate than necessary
                let hastened =
                    played_mate - best_mate;
                Some(if hastened <= 2 {
                    MoveBadge::Excellent
                } else if hastened <= 5 {
                    MoveBadge::Inaccuracy
                } else {
                    MoveBadge::Mistake
                })
            }
        }

        // Attacker slipped and defender escaped mate / Defender found a mate
        // These are dramatic positional swings; fall through to the
        // win-probability classifier, which handles them naturally.
        _ => None,
    }
}

/// Processes evaluation shifts, heuristic flags, and win-probability loss
/// to assign `MoveBadge` classification.
pub fn classify(
    args: ClassifyArgs,
) -> (MoveBadge, f64) {
    // Book moves are established human theory. An engine might hate a
    // deep Sicilian Najdorf line at depth 15, but it is objectively correct.
    // Therefore, book moves bypass all mathematical evaluation.
    if args.is_book {
        return (MoveBadge::Book, 0.0);
    }

    // The absolute difference in centipawns between best engine move and played move.
    let delta =
        args.prev_best_eval - args.played_eval;

    // The drop in win probability caused by the move.
    // Is used as the primary metric for assigning base classifications.
    let mut win_loss =
        calculate_win_percent(args.prev_eval)
            - calculate_win_percent(
                args.played_eval,
            );

    if args.is_forced_move {
        return (MoveBadge::Forced, win_loss);
    }

    // Mate Distance Classification
    // When both the best line and the played line are in a forced-mate sequence,
    // the -+10000 centipawn clamping used elsewhere is too coarse: it cannot
    // distinguish "mate in 1" from "mate in 20". We use the raw mate-in-N
    // distances instead and skip the centipawn-based base classification below.
    let mut classification =
        if let (Some(bm), Some(pm)) =
            (args.best_mate, args.played_mate)
        {
            match classify_mate_vs_mate(
                bm,
                pm,
                args.is_best_engine_move,
            ) {
                Some(badge) => badge,
                // fall through to centipawn logic
                None => {
                    centipawn_base_classification(
                        args.prev_eval,
                        args.played_eval,
                        delta,
                        win_loss,
                        args.is_best_engine_move,
                    )
                }
            }
        } else {
            // Normal (non-mate vs mate) base classification
            // The derived `is_delivering_mate` / `is_getting_mated` flags handle the edge
            // cases where only one side is in a mate sequence (like finding a
            // checkmate from an equal position). Those positions still produce
            // valid +-10000 clamped centipawn scores that the centipawn classifier handles
            // correctly; the flags then refine the result below.
            centipawn_base_classification(
                args.prev_eval,
                args.played_eval,
                delta,
                win_loss,
                args.is_best_engine_move,
            )
        };

    // Refinements that apply after both classification paths

    // Attacker played a slower mate than the engine's best: the mate-vs-mate
    // classifier already handles this precisely. The `is_delivering_mate` flag
    // only fires when played_mate is None (ex. no forced mate in the played
    // line), which means the player found a mating move but not the fastest
    // forced sequence
    // downgrade Best -> Excellent.
    if args.is_delivering_mate
        && args.played_mate.is_none()
        && !args.is_best_engine_move
        && classification == MoveBadge::Best
    {
        classification = MoveBadge::Excellent;
    }

    // Defender is getting mated and played_mate is None: they were in a lost
    // position and played a non-optimal move.  The existing centipawn logic already
    // reflects the win-probability, but the `is_getting_mated` flag lets us
    // catch the specific case where the best move also leads to getting mated
    // (both evals approximately equals −10000) yet the player still played sub-optimally.
    if args.is_getting_mated
        && args.played_mate.is_none()
        && !args.is_best_engine_move
        && classification == MoveBadge::Best
    {
        classification = MoveBadge::Inaccuracy;
    }

    // MISS: If the opponent's previous move resulted in a large drop for them (>= 5%),
    // but our current move gives 70% to 140% of that advantage right back.
    // It's a bad/inaccurate move that fails to punish a bad move from opponent.
    // Or just misses a faster way to win the game
    if args.prev_win_loss >= 5.0
        && win_loss >= (args.prev_win_loss * 0.7)
        && win_loss <= (args.prev_win_loss * 1.4)
    {
        classification = MoveBadge::Miss;
    }

    // MISTAKE Override:
    // If the player had a highly winning position (e.g., > +3.0 / 300cp) and made a massive
    // drop in evaluation, but the resulting position is still equal or better (>= 0cp),
    // it shouldn't be a Blunder. They didn't lose the game, they just threw away the win.
    if classification == MoveBadge::Blunder
        && args.prev_eval >= 300
        && args.played_eval >= 0
    {
        classification = MoveBadge::Mistake;
    }

    // GREAT MOVE: If the played move matches the best move, AND the second-best move
    // creates an 8.5% win probability drop, this was an "Only Move". The player navigated
    // a tightrope where any other choice would have lost the advantage or straight up loses.
    if (classification == MoveBadge::Best
        || classification == MoveBadge::Excellent)
        && args.prev_eval >= -100
        && is_great_move(
            args.played_eval,
            args.prev_best_eval,
            args.multi_pv_evals,
            args.is_obvious_recapture,
        )
    {
        classification = MoveBadge::Great;
    }

    //if an only King move out of check was the only logical choice instead of chucking pieces
    if classification == MoveBadge::Great
        && args.is_trivial_check_evasion
    {
        classification = MoveBadge::Best;
    }

    // BRILLIANT MOVE: we use Static Exchange Evaluation (SEE) to confirm a piece is *actually* hanging.
    // If SEE says it's hanging, but the engine eval didn't drop, it's a sound sacrifice.
    //
    // In decided positions (|prev_eval| >= 500cp) or endgames, a sacrifice is only awarded
    // Brilliant if it also passes the `is_brilliant_in_decided_position` check. It is a hybrid
    // guard that requires either a true Great-move-level only-move pressure, or a large
    // second-PV divergence. This prevents farming brilliancies in won/lost positions where
    // the opponent simply hangs material without meaningful resistance (e.g. K+B vs K+P).
    if win_loss < 5.0
        && delta <= 40
        && args.is_losing_material
        && args.played_eval > 0
    {
        let position_is_decided =
            args.prev_eval.abs() >= 500;

        let eligible = if position_is_decided
            || args.is_endgame
        {
            is_brilliant_in_decided_position(
                args.played_eval,
                args.prev_best_eval,
                args.multi_pv_evals,
                args.is_obvious_recapture,
                delta,
            )
        } else {
            true
        };

        if eligible {
            classification = MoveBadge::Brilliant;
        }
    }

    if classification == MoveBadge::Best
        || classification == MoveBadge::Great
        || classification == MoveBadge::Brilliant
    {
        win_loss = win_loss.min(0.0);
    }

    (classification, win_loss)
}

/// Centipawn base classification used when neither the best nor the played
/// move is part of a forced mate sequence (or when one side has escaped a mate).
fn centipawn_base_classification(
    prev_eval: i32,
    played_eval: i32,
    delta: i32,
    win_loss: f64,
    is_best_engine_move: bool,
) -> MoveBadge {
    if prev_eval.abs() > 1000
        && played_eval.abs() > 1000
    {
        if delta <= 0 || is_best_engine_move {
            MoveBadge::Best
        } else {
            MoveBadge::Excellent
        }
    } else if is_best_engine_move || delta <= 0 {
        MoveBadge::Best
    } else {
        match win_loss {
            w if w >= 20.0 => MoveBadge::Blunder,
            w if w >= 10.0 => MoveBadge::Mistake,
            w if w >= 5.0 => {
                MoveBadge::Inaccuracy
            }
            w if w >= 2.0 => MoveBadge::Good,
            _ => MoveBadge::Excellent,
        }
    }
}

/// A heuristic helper function that checks if the played move was
/// the only viable option to maintain advantage or equality.
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

/// Gate for Brilliant moves in decided positions (`|prev_eval| >= 500`) or endgames.
///
/// In these contexts the position is already largely resolved, so a hanging piece
/// is often just a free capture with no real calculation required, not a genuine
/// brilliancy. We require the sacrifice to also demonstrate real decisiveness
/// via one of two paths:
///
/// ONLY MOVE:
/// The `is_great_move` check fires: every alternative would cause a meaningful
/// win-probability collapse. The sigmoid is intentionally flat near +-100%, so
/// this path wont fire in truly dead positions, which is the correct behaviour.
///
/// :
/// The second PV is at least 100cp worse than the played move AND the sacrifice
/// itself barely costs anything (`delta <= 20`). This catches genuine tactical
/// shots (ex. a rook sac to force stalemate or promotion) in endgames
/// where the sigmoid flatness prevents Path A from triggering.
///
/// If neither path passes, the position is decided enough that the sacrifice
/// adds no meaningful complexity therefore capped it at `Excellent` or `Best`.
fn is_brilliant_in_decided_position(
    played_eval: i32,
    best_eval: i32,
    multi_pv_evals: &[i32],
    is_obvious_recapture: bool,
    delta: i32,
) -> bool {
    // Path A: only-move pressure (same bar as Great)
    if is_great_move(
        played_eval,
        best_eval,
        multi_pv_evals,
        is_obvious_recapture,
    ) {
        return true;
    }

    // Path B: large second-PV divergence with a tight delta
    if let Some(&second_pv) =
        multi_pv_evals.get(1)
    {
        let second_pv_gap =
            played_eval - second_pv;
        if second_pv_gap >= 100 && delta <= 20 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Existing centipawn-based tests ───────────────────────────────────────────────

    #[test]
    fn excellent_when_position_decided_but_not_best_move(
    ) {
        let args = ClassifyArgs {
            prev_eval: 1200,
            played_eval: 1100,
            prev_best_eval: 1200,
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
            prev_best_eval: 1200,
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
            prev_best_eval: 290,
            multi_pv_evals: &[290, 20, -50],
            is_losing_material: true,
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
            prev_best_eval: 80,
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
            prev_best_eval: -300,
            multi_pv_evals: &[-300, -500],
            is_losing_material: true,
            ..Default::default()
        };
        assert_ne!(
            classify(args).0,
            MoveBadge::Brilliant
        );
    }

    #[test]
    fn test_miss_classification() {
        let args = ClassifyArgs {
            prev_eval: 100,
            played_eval: 0,
            prev_best_eval: 100,
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
            played_eval: -50,
            prev_best_eval: 400,
            multi_pv_evals: &[400, 300],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Blunder
        );
    }

    #[test]
    fn mistake_by_equalizing_position_from_winning(
    ) {
        let args = ClassifyArgs {
            prev_eval: 400,
            played_eval: 0,
            prev_best_eval: 400,
            multi_pv_evals: &[400, 300],
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Mistake
        );
    }

    #[test]
    fn mistake_by_win_percent_drop() {
        let args = ClassifyArgs {
            prev_eval: 200,
            played_eval: 0,
            prev_best_eval: 200,
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
            prev_best_eval: 150,
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
            prev_best_eval: 100,
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
            prev_eval: 55,
            played_eval: 40,
            prev_best_eval: 55,
            multi_pv_evals: &[55, 40],
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
            prev_best_eval: 28,
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
            prev_best_eval: 28,
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

    #[test]
    fn black_blunder_detected_with_corrected_pov()
    {
        let args = ClassifyArgs {
            prev_eval: 0,
            played_eval: -400,
            prev_best_eval: 0,
            multi_pv_evals: &[0, -400],
            ..Default::default()
        };

        assert_eq!(
            classify(args).0,
            MoveBadge::Blunder
        );
    }

    #[test]
    fn great_move_with_corrected_multi_pv_pov() {
        let args = ClassifyArgs {
            prev_eval: 30,
            played_eval: 28,
            prev_best_eval: 30,
            multi_pv_evals: &[30, -150],
            ..Default::default()
        };

        assert_eq!(
            classify(args).0,
            MoveBadge::Great
        );
    }

    #[test]
    fn test_attacker_plays_slower_mate_sequence()
    {
        let args = ClassifyArgs {
            prev_eval: 10000,
            played_eval: 10000,
            prev_best_eval: 10000,
            multi_pv_evals: &[10000, 10000],
            is_delivering_mate: true,
            is_best_engine_move: false,
            ..Default::default()
        };

        assert_eq!(
            classify(args).0,
            MoveBadge::Excellent
        );
    }

    #[test]
    fn test_defender_allows_faster_mate_sequence()
    {
        let args = ClassifyArgs {
            prev_eval: -10000,
            played_eval: -10000,
            prev_best_eval: -10000,
            multi_pv_evals: &[-10000, -10000],
            is_getting_mated: true,
            is_best_engine_move: false,
            ..Default::default()
        };

        assert_eq!(
            classify(args).0,
            MoveBadge::Inaccuracy
        );
    }

    #[test]
    fn test_defender_finds_most_resilient_line() {
        let args = ClassifyArgs {
            prev_eval: -10000,
            played_eval: -10000,
            prev_best_eval: -10000,
            multi_pv_evals: &[-10000, -10000],
            is_getting_mated: true,
            is_best_engine_move: true,
            ..Default::default()
        };

        assert_eq!(
            classify(args).0,
            MoveBadge::Best
        );
    }

    #[test]
    fn attacker_plays_fastest_mate_is_best() {
        let args = ClassifyArgs {
            prev_eval: 10000,
            played_eval: 10000,
            prev_best_eval: 10000,
            multi_pv_evals: &[10000, 10000],
            best_mate: Some(3),
            played_mate: Some(3),
            is_best_engine_move: false,
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Best
        );
    }

    #[test]
    fn attacker_plays_mate_one_move_slower_is_excellent(
    ) {
        let args = ClassifyArgs {
            prev_eval: 10000,
            played_eval: 10000,
            prev_best_eval: 10000,
            multi_pv_evals: &[10000, 10000],
            best_mate: Some(2),
            played_mate: Some(3),
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Excellent
        );
    }

    #[test]
    fn attacker_plays_mate_four_moves_slower_is_good(
    ) {
        let args = ClassifyArgs {
            prev_eval: 10000,
            played_eval: 10000,
            prev_best_eval: 10000,
            multi_pv_evals: &[10000, 10000],
            best_mate: Some(1),
            played_mate: Some(5),
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Good
        );
    }

    #[test]
    fn attacker_plays_mate_ten_moves_slower_is_inaccuracy(
    ) {
        let args = ClassifyArgs {
            prev_eval: 10000,
            played_eval: 10000,
            prev_best_eval: 10000,
            multi_pv_evals: &[10000, 10000],
            best_mate: Some(3),
            played_mate: Some(13),
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Inaccuracy
        );
    }

    #[test]
    fn attacker_with_is_best_engine_move_is_best()
    {
        // Even if mate distances differ, is_best_engine_move overrides
        let args = ClassifyArgs {
            prev_eval: 10000,
            played_eval: 10000,
            prev_best_eval: 10000,
            multi_pv_evals: &[10000, 10000],
            best_mate: Some(1),
            played_mate: Some(1),
            is_best_engine_move: true,
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Best
        );
    }

    #[test]
    fn defender_finds_most_resilient_line_with_mate_distances(
    ) {
        let args = ClassifyArgs {
            prev_eval: -10000,
            played_eval: -10000,
            prev_best_eval: -10000,
            multi_pv_evals: &[-10000, -10000],
            best_mate: Some(-8),
            played_mate: Some(-8),
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Best
        );
    }

    #[test]
    fn defender_slightly_worse_is_excellent() {
        let args = ClassifyArgs {
            prev_eval: -10000,
            played_eval: -10000,
            prev_best_eval: -10000,
            multi_pv_evals: &[-10000, -10000],
            best_mate: Some(-8),
            played_mate: Some(-7),
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Excellent
        );
    }

    #[test]
    fn defender_significantly_worse_is_inaccuracy(
    ) {
        // Best: mated in 10. Played: mated in 6. Hastened by 4 -> Inaccuracy
        let args = ClassifyArgs {
            prev_eval: -10000,
            played_eval: -10000,
            prev_best_eval: -10000,
            multi_pv_evals: &[-10000, -10000],
            best_mate: Some(-10),
            played_mate: Some(-6),
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Inaccuracy
        );
    }

    #[test]
    fn defender_badly_worse_is_mistake() {
        // Best: mated in 12. Played: mated in 4. Hastened by 8 -> Mistake
        let args = ClassifyArgs {
            prev_eval: -10000,
            played_eval: -10000,
            prev_best_eval: -10000,
            multi_pv_evals: &[-10000, -10000],
            best_mate: Some(-12),
            played_mate: Some(-4),
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Mistake
        );
    }

    #[test]
    fn mixed_mate_falls_through_to_centipawn_classifier(
    ) {
        // Attacker's best: mate in 3 (positive). Player: no mate found, eval drops.
        // Mixed scenario -> falls through to win-probability classifier -> Blunder.
        let args = ClassifyArgs {
            prev_eval: 400,
            played_eval: -50,
            prev_best_eval: 400,
            multi_pv_evals: &[400, 300],
            best_mate: Some(3), // engine found mate
            played_mate: None, // player missed it entirely
            ..Default::default()
        };
        assert_eq!(
            classify(args).0,
            MoveBadge::Blunder
        );
    }
}
