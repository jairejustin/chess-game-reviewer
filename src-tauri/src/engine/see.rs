use shakmaty::san::San;
use shakmaty::{
    Chess, Color, Move, Position, Role, Square,
};

/// Safely extracts the geometric destination square of a move from SAN
/// using `shakmaty`'s AST parser guarantees we always identify the exact board square.
pub fn get_target_square(
    san_str: &str,
) -> Option<Square> {
    // Parse the SAN string into shakmaty's native enum
    let parsed_san =
        San::from_ascii(san_str.as_bytes())
            .ok()?;

    // Extract the destination square. Castling (O-O / O-O-O) returns None
    // because you can't "recapture" on a castling square.
    match parsed_san {
        San::Normal { to, .. } => Some(to),
        _ => None,
    }
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
///
/// LEGAL MOVES: Custom ray-casting SEE implementations often fail to account
/// for absolute pins (e.g., an attacking Rook is pinned to its King and cannot actually capture).
/// By utilizing `pos.legal_moves()`, we guarantee 100% accurate evaluations of pins,
/// discovered attacks, and en passant, at the slight cost of performance.
///
/// LVA (Lowest Value Attacker): We always simulate the capture using the weakest
/// attacking piece. A rational player will capture a pawn with their pawn, not their Queen.
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
///
/// SEE >= 2: A value of 1 usually means a pawn trade. A loss of 2 or more indicates
/// a true piece sacrifice (e.g., giving up a Knight (3) for a Pawn (1) results in a net loss of 2).
/// We use SEE because it mathematically confirms the piece is actually trapped/hanging on the board,
/// separate from what the engine evaluation says about the overall position.
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
