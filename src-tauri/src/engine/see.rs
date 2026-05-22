use shakmaty::{
    Chess, Color, Move, Position, Role, Square,
};

/// Extracts the 2-character destination square from a capture SAN.
/// Example: "Bxf6+" -> Some("f6"), "e4" -> None
pub fn capture_square(san: &str) -> Option<&str> {
    if let Some(x_idx) = san.find('x') {
        let square_start = x_idx + 1;
        let square_end = square_start + 2;
        if square_end <= san.len() {
            return Some(
                &san[square_start..square_end],
            );
        }
    }
    None
}

pub fn piece_value(role: Role) -> i32 {
    match role {
        Role::Pawn => 1,
        Role::Knight | Role::Bishop => 3,
        Role::Rook => 5,
        Role::Queen => 9,
        Role::King => 10000,
    }
}

/// Recursively calculates the Static Exchange Evaluation (SEE) on a target square.
/// Returns the net material gain for the side whose turn it is to move.
pub fn legal_see(
    pos: &Chess,
    target: Square,
) -> i32 {
    let captures: Vec<Move> = pos
        .legal_moves()
        .into_iter()
        .filter(|m| {
            m.to() == target && m.is_capture()
        })
        .collect();

    if captures.is_empty() {
        return 0; // No attackers left, sequence ends
    }

    // Find the capture with the lowest value attacker (LVA)
    let mut best_capture = None;
    let mut lowest_attacker_val = i32::MAX;

    for m in captures {
        let val = piece_value(m.role());
        if val < lowest_attacker_val {
            lowest_attacker_val = val;
            best_capture = Some(m);
        }
    }

    let best_move = best_capture.unwrap();

    let captured_role = pos
        .board()
        .piece_at(target)
        .unwrap()
        .role;
    let current_gain = piece_value(captured_role);

    let next_pos =
        pos.clone().play(best_move).unwrap();

    let opponent_gain =
        legal_see(&next_pos, target);

    current_gain - opponent_gain.max(0)
}

/// Checks if any piece belonging to `color` is hanging (loss of >= 2 material).
pub fn is_sacrifice(
    current_pos: &Chess,
    color: Color,
) -> bool {
    let board = current_pos.board();

    for sq in board.by_color(color) {
        if legal_see(current_pos, sq) >= 2 {
            return true;
        }
    }
    false
}
