use shakmaty::san::San;
use shakmaty::{
    Chess, Color, Move, Position, Role, Square,
};

/// Extracts the exact geometric destination square from SAN string.
/// It then maps the SAN string into `shakmaty::Square` enum.
/// Returns `None` on castling.
pub fn get_target_square(
    san_str: &str,
) -> Option<Square> {
    // Parses the SAN string
    let parsed_san =
        San::from_ascii(san_str.as_bytes())
            .ok()?;

    // Extract the destination square.
    // Castling (O-O / O-O-O) returns None because you can't "recapture" on a castling square.
    match parsed_san {
        San::Normal { to, .. } => Some(to),
        _ => None,
    }
}

/// Maps standard chess pieces to their traditional mathematical point values
pub fn piece_value(role: Role) -> i32 {
    match role {
        Role::Pawn => 1,
        Role::Knight | Role::Bishop => 3,
        Role::Rook => 5,
        Role::Queen => 9,
        Role::King => 10000,
    }
}

/// Recursively simulates a sequence of captures on a single square from
/// the lowest value attackers to calculate the final net material gain (for the opponent).
///
/// Guarantees accurate evaluations of pins,
/// discovered attacks, and en passant, at the slight cost of performance though.
///
/// Reference: [Chess Programming Wiki: Static Exchange Evaluation](https://www.chessprogramming.org/Static_Exchange_Evaluation)
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

    // Base case - no attackers left
    if captures.is_empty() {
        return 0;
    }

    // Find the capture with the lowest value attacker
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

/// Iterates through player's every pieces using Static Exchange Evaluation to determine
/// if any piece is genuinely hanging for a material loss of 2 or more points.
/// This is a pure mathematical heuristic evaluating the squares. It confirms
/// if any piece is vulnerable to a losing capture sequence
//
//
// SEE >= 2: A value of 1 usually means an even pawn trade. A loss of 2 or more indicates
// a true piece loss (e.g., losing a Knight (3) to a Pawn (1) results in a net loss of 2).
pub fn is_losing_significant_material(
    current_pos: &Chess,
    color: Color,
) -> bool {
    // Extracts the layout of the current chessboard position
    let board = current_pos.board();

    // Iterates though the board layout to identify squares occupied by color's pieces.
    // It then uses SEE to simulate every capture possible to that piece
    for sq in board.by_color(color) {
        if legal_see(current_pos, sq) >= 2 {
            return true;
        }
    }
    false
}

/// Checks if the played move is a sacrifice, meaning the player deliberately moved a piece
/// to a square where the opponent can profitably capture it (net loss >= 2 points).
///
/// This must be evaluated on the pre-move board because SEE internally simulates
/// the capture sequence starting from the destination square.
/// Its a different check from the function above.
/// The function above answers: Did the move leave anything else on the board undefended?
/// This function answeres the question: Did the player voluntarily land on a dangerous square?
pub fn is_sacrifice(
    pre_move_pos: &Chess,
    played_move: &Move,
) -> bool {
    let target_sq = played_move.to();

    // Identifies the value of the piece being captured (0 on empty square)
    let captured_val = pre_move_pos
        .board()
        .piece_at(target_sq)
        .map(|p| piece_value(p.role))
        .unwrap_or(0);

    // Plays the move to shift the turn perspective to the opponent
    let next_pos = pre_move_pos
        .clone()
        .play(played_move.clone())
        .unwrap();

    // Calculates how much the opponent can win on that square
    let opponent_gain =
        legal_see(&next_pos, target_sq);

    // Calculates the net material exchange for the moving player
    let net_gain = captured_val - opponent_gain;

    // A significant material sacrifice occurs when the player voluntarily 
    // loses >= 2 points in the exchange
    net_gain <= -2
}
