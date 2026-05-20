use crate::models::game::MoveBadge;

pub fn classify (prev_eval: i32, played_eval: i32, best_move_eval: i32) -> MoveBadge {
    if prev_eval.abs() > 1000 && played_eval.abs() > 1000 {
        return MoveBadge::Best;
    }

    let win_loss = calculate_win_percent(prev_eval) - calculate_win_percent(played_eval);
    let delta = best_move_eval - played_eval;

    // let mut classification: MoveBadge = check here if its on an opening database

    let classification: MoveBadge = match win_loss {
        w if w >= 30.0 => MoveBadge::Blunder,
        w if w >= 20.0 => MoveBadge::Mistake,
        w if w >= 10.0 => MoveBadge::Inaccuracy,
        _ => match delta {
            d if d <= 15 => MoveBadge::Best,
            d if d <= 40 => MoveBadge::Excellent,
            _ => MoveBadge::Inaccuracy,

        }
    };

    // here would probably be the great move check, I also need info on other lines,
    // to know a great move the first 3 lines must be of equavalent in eval or classification atleast
    // but the rest of the lines loses, or loses advantage atleast. like a critical move in a position.

    // ill probably put brilliancy check here, I somehow need a way to get material count,
    // to know if a move loses material but eval shows otherwise. To be a brilliant move,
    // it must be either a best move, an excellent move, a good move, or a great move beforehand.

    classification
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
