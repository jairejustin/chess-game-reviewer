use crate::models::game::MoveBadge;

pub fn classify(
    prev_eval: i32,
    played_eval: i32,
    best_move_eval: i32,
    multi_pv_evals: &[i32],
    material_delta: i32,
) -> MoveBadge {
    if prev_eval.abs() > 1000
        && played_eval.abs() > 1000
    {
        return MoveBadge::Best;
    }

    let win_loss =
        calculate_win_percent(prev_eval)
            - calculate_win_percent(played_eval);
    let delta = best_move_eval - played_eval;

    // Brilliant: position dropped in raw material (sacrificed), but engine confirms move is near-best
    if win_loss < 5.0
        && delta <= 40
        && material_delta < 0
    {
        return MoveBadge::Brilliant;
    }

    let mut classification: MoveBadge =
        match win_loss {
            w if w >= 20.0 => MoveBadge::Blunder,
            w if w >= 10.0 => MoveBadge::Mistake,
            w if w >= 5.0 => {
                MoveBadge::Inaccuracy
            }
            _ => match delta {
                d if d <= 15 => MoveBadge::Best,
                d if d <= 40 => {
                    MoveBadge::Excellent
                }
                _ => MoveBadge::Inaccuracy,
            },
        };

    // Great: move is near-best and there is a steep drop-off to the next best line
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

    // Practical blunder checks
    classification = if delta >= 200
        && prev_eval > -100
        && classification == MoveBadge::Inaccuracy
    {
        MoveBadge::Blunder
    } else {
        classification
    };

    classification
}

fn is_great_move(
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

#[test]
fn brilliant_when_sacrifice_confirmed_by_engine()
{
    assert_eq!(
        classify(300, 50, 80, &[80, 20, -50]),
        MoveBadge::Brilliant
    );
}

#[test]
fn not_brilliant_without_material_sacrifice() {
    assert_eq!(
        classify(50, 45, 80, &[80, 60, 40]),
        MoveBadge::Excellent
    );
}

#[test]
fn great_when_only_move_maintaining_equality() {
    assert!(is_great_move(
        28,
        30,
        &[30, -150, -200]
    ));
}

#[test]
fn not_great_when_multiple_good_moves_exist() {
    assert!(!is_great_move(
        28,
        30,
        &[30, 25, 20]
    ));
}

#[test]
fn best_when_played_equals_best_move() {
    assert_eq!(
        classify(30, 28, 28, &[28, 10, -20]),
        MoveBadge::Best
    );
}
