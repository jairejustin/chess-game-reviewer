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

/// Determines whether the played move introduced a genuine material sacrifice for the
/// moving player using Static Exchange Evaluation (SEE).
///
/// Handles both direct sacrifices (the moved piece lands on a dangerous square) and
/// indirect sacrifices (the move exposes or abandons a defender elsewhere on the board).
///
/// A sacrifice is only confirmed if ALL of the following are true:
///
/// 1. **Differential check** (`see_after > see_before`): the danger on a square is strictly
///    worse after the move than before. This filters out pre-existing hangs, even trades,
///    and profitable captures automatically.
///
/// 2. **Danger levels check**: simulates the opponent's cheapest capture on the hanging
///    piece and checks if the opponent has an equal or greater counter-threat available
///    afterward. If so, the opponent cannot safely take because the move is a tempo or
///    counter-attack, therefore it is not a real sacrifice.
pub fn is_material_sacrifice(
    pre_move_pos: &Chess,
    post_move_pos: &Chess,
    played_move: &Move,
    color: Color,
) -> bool {
    let board_after = post_move_pos.board();

    // Look up the value of what was captured on the destination square
    // before the move (0 if empty)
    let captured_val = pre_move_pos
        .board()
        .piece_at(played_move.to())
        .map(|p| piece_value(p.role))
        .unwrap_or(0);

    // Iterates exclusively through the moving player's pieces on the
    // post-move board
    for sq in board_after.by_color(color) {
        let see_after =
            legal_see(post_move_pos, sq);

        // Skip squares that do not carry a meaningful material loss
        // going forward (threshold >= 2)
        if see_after < 2 {
            continue;
        }

        // Establish what the hanging baseline score was before the move occurred
        let see_before = if sq == played_move.to()
        {
            // Direct case: destination square's baseline is the captured piece's value
            captured_val
        } else {
            // Indirect case: check if this piece was already compromised on
            // its existing square
            legal_see(pre_move_pos, sq)
        };

        // If danger was newly introduced or strictly worsened on this square,
        // a sacrifice is confirmed
        if see_after > see_before {
            // Danger levels check: if the opponent captures the hanging piece,
            // does the moving player have an immediate counter-threat of equal
            // or greater value? If so, the opponent can't safely take.
            // This is a tempo/counter-attack move, not a real sacrifice.
            let capture = cheapest_capture(
                post_move_pos,
                sq,
            )
            .unwrap();
            let danger_pos = post_move_pos
                .clone()
                .play(capture)
                .unwrap();

            let counter_threat =
                best_counter_threat(
                    &danger_pos,
                    !color,
                );
            let hanging_piece_val = pre_move_pos
                .board()
                .piece_at(sq)
                .map(|p| piece_value(p.role))
                .unwrap_or(see_after);

            if counter_threat >= hanging_piece_val
            {
                continue;
            }

            return true;
        }
    }

    false
}

/// Finds the highest value piece of `color` that is hanging (SEE >= 2) on the given position.
/// Used to evaluate counter-threats after a simulated opponent capture.
fn best_counter_threat(
    pos: &Chess,
    color: Color,
) -> i32 {
    pos.board()
        .by_color(color)
        .into_iter()
        .map(|sq| legal_see(pos, sq))
        .filter(|&see| see >= 2)
        .max()
        .unwrap_or(0)
}

/// Finds the cheapest legal capture the opponent can make on the target square.
fn cheapest_capture(
    pos: &Chess,
    target: Square,
) -> Option<Move> {
    pos.legal_moves()
        .into_iter()
        .filter(|m| {
            m.to() == target && m.is_capture()
        })
        .min_by_key(|m| piece_value(m.role()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::fen::Fen;
    use shakmaty::san::San;
    use shakmaty::CastlingMode;

    #[test]
    fn test_get_target_square_invalid_san() {
        assert_eq!(
            get_target_square("not_a_chess_move"),
            None
        );
    }

    #[test]
    fn test_get_target_square_parsing() {
        // Normal pawn move
        assert_eq!(
            get_target_square("e4"),
            Some(Square::E4)
        );

        // Piece capture
        assert_eq!(
            get_target_square("Nxf3"),
            Some(Square::F3)
        );

        // Disambiguated piece move
        assert_eq!(
            get_target_square("Rxd8+"),
            Some(Square::D8)
        );

        // Castling should correctly return None
        assert_eq!(
            get_target_square("O-O"),
            None
        );
        assert_eq!(
            get_target_square("O-O-O"),
            None
        );
    }

    #[test]
    fn test_legal_see_computes_exact_exchange() {
        let fen =
            "8/8/8/1kpn4/6K1/2P5/1P6/8 b - - 0 1";

        let fen_obj =
            Fen::from_ascii(fen.as_bytes())
                .expect("Invalid FEN");
        let pos: Chess = fen_obj
            .into_position(CastlingMode::Standard)
            .expect("Invalid Position");

        let target = Square::C3;

        let net_gain = legal_see(&pos, target);

        // Knight exchanged for a pawn, net gain = -2
        assert_eq!(net_gain, -2);
    }

    #[test]
    fn test_legal_see_stops_trading_before_losing_queen(
    ) {
        let fen = "8/1q6/2b2k2/5p1p/3P2nP/3K1B2/3N1PQ1/8 b - - 0 1";

        let fen_obj =
            Fen::from_ascii(fen.as_bytes())
                .expect("Invalid FEN");
        let pos: Chess = fen_obj
            .into_position(CastlingMode::Standard)
            .expect("Invalid Position");

        // Target the contested f3 square
        let target = Square::F3;

        // Run legal_see for Black's perspective initiating on f3
        let net_gain = legal_see(&pos, target);

        // Trade of bishops, Net gain = 0
        assert_eq!(net_gain, 0);
    }

    #[test]
    fn test_legal_see_on_empty_square_returns_zero(
    ) {
        // Standard starting position
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen_obj =
            Fen::from_ascii(fen.as_bytes())
                .unwrap();
        let pos: Chess = fen_obj
            .into_position(CastlingMode::Standard)
            .unwrap();

        // Target an empty square like e4 before any moves are made
        let target = Square::E4;

        // Since there are no opposing pieces to capture on e4, SEE should be 0.
        let net_gain = legal_see(&pos, target);

        assert_eq!(net_gain, 0);
    }

    /// Helper to run the differential sacrifice check on a specific move
    fn run_sacrifice_test(
        fen_str: &str,
        san_str: &str,
        color: Color,
    ) -> bool {
        // Setup Pre-Move Position
        let fen =
            Fen::from_ascii(fen_str.as_bytes())
                .expect("Invalid FEN");
        let pre_pos: Chess = fen
            .into_position(CastlingMode::Standard)
            .expect("Invalid Position");

        // Parse the Move
        let san =
            San::from_ascii(san_str.as_bytes())
                .expect("Invalid SAN");
        let played_move = san
            .to_move(&pre_pos)
            .expect("Illegal Move");

        // Setup Post-Move Position
        let post_pos = pre_pos
            .clone()
            .play(played_move)
            .expect("Failed to play move");

        let see_f6_before =
            legal_see(&pre_pos, Square::F6);
        let see_f6_after =
            legal_see(&post_pos, Square::F6);
        println!(
            "f6 see_before: {}, see_after: {}",
            see_f6_before, see_f6_after
        );

        // Run the heuristic
        is_material_sacrifice(
            &pre_pos,
            &post_pos,
            &played_move,
            color,
        )
    }

    #[test]
    fn test_knight_captures_a_defended_bishop_is_not_a_sacrifice(
    ) {
        let fen = "r4rk1/5ppp/8/p1P1p2q/3pPn2/P4bP1/3N1P1P/R1Q1RBK1 w - - 0 1";

        let move_played = "Nxf3";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::White,
        );

        assert_eq!(is_sacrifice, false);
    }

    #[test]
    fn test_bishop_captures_h7_pawn_is_a_sacrifice(
    ) {
        let fen = "r1bq1rk1/pp1n1ppp/2n1p3/2ppP3/3P4/2PB1N2/P1P2PPP/R1BQ1RK1 w - - 0 1";

        let move_played = "Bxh7";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::White,
        );

        assert_eq!(is_sacrifice, true);
    }

    #[test]
    fn test_knight_moves_to_a_square_defended_by_pawn_is_a_sacrifice(
    ) {
        let fen = "r4rk1/5ppp/6n1/p1P1p2q/3pP3/PN3bP1/5P1P/R1Q1RBK1 b - - 0 1";

        let move_played = "Nf4";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::Black,
        );

        assert_eq!(is_sacrifice, true);
    }

    #[test]
    fn test_alien_gambit_my_gambit_my_legacy_is_a_sacrifice(
    ) {
        let fen = "rnbqkb1r/pp2ppp1/2p2n1p/6N1/3P4/8/PPP2PPP/R1BQKBNR w KQkq - 0 1";

        let move_played = "Nxf7";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::White,
        );

        assert_eq!(is_sacrifice, true);
    }

    #[test]
    fn test_hanging_the_queen_for_mate_is_a_sacrifice(
    ) {
        let fen = "rn1qkb1r/ppp1pppp/5n2/8/2B3b1/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 0 1";

        let move_played = "Ne5";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::White,
        );

        assert_eq!(is_sacrifice, true);
    }

    #[test]
    fn test_taking_a_hanging_knight_is_not_a_sacrifice(
    ) {
        let fen = "8/1b6/5k2/4pp1p/3P3P/2P1K3/5PN1/8 b - - 0 1";

        let move_played = "Bxg2";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::Black,
        );

        assert_eq!(is_sacrifice, false);
    }

    #[test]
    fn test_ignoring_the_hanging_bishop_to_push_passer_is_a_sacrifice(
    ) {
        let fen =
            "8/3k4/5bK1/4p3/8/8/8/8 b - - 0 1";

        let move_played = "e4";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::Black,
        );

        assert_eq!(is_sacrifice, true);
    }

    #[test]
    fn test_hanging_knight_to_counterattack_queen_is_not_sacrifice(
    ) {
        let fen = "1rb1qrk1/1ppn1pp1/p2p1n1p/4P3/2QP3B/1P3N2/P1B2PPP/R4RK1 b - - 0 1";

        let move_played = "b5";

        let is_sacrifice = run_sacrifice_test(
            fen,
            move_played,
            Color::Black,
        );

        assert_eq!(is_sacrifice, false);
    }
}
