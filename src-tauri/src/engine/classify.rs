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
    50.0 + 50.0 * (2.0 / (1.0 + (-0.00368 * cp as f64).exp()) - 1.0)
}