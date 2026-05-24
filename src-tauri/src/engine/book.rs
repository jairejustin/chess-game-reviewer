use polyglot_book_rs::PolyglotBook;
use shakmaty::san::San;
use shakmaty::Chess;
use std::path::Path;

/// A wrapper around a Polyglot binary database to evaluate opening theory.
pub struct OpeningBook {
    book: Option<PolyglotBook>,
}

impl OpeningBook {
    /// Initializes the opening database
    pub fn new(book_path: &str) -> Self {
        let book = if Path::new(book_path).exists() {
            PolyglotBook::load(book_path).ok()
        } else {
            None
        };
        
        OpeningBook { book }
    }

    /// Checks if the specific move played exists in the opening database for the given board state.
    pub fn is_book_move(&self, pos: &Chess, board_fen: &str, played_san: &str) -> bool {
        let book = match &self.book {
            Some(b) => b,
            None => return false,
        };

        // Query the Polyglot database directly with the provided string
        let theory_moves = book.get_all_moves_from_fen(board_fen);

        if theory_moves.is_empty() {
            return false; // The current position is entirely out of book.
        }

        // Parse the player's SAN string into a geometric move
        let parsed_san_result = San::from_ascii(played_san.as_bytes());
        let player_move = match parsed_san_result {
            Ok(san) => match san.to_move(pos) {
                Ok(m) => m,
                Err(_) => return false,
            },
            Err(_) => return false,
        };

        // Convert shakmaty's move to a standard UCI string
        let player_uci = player_move.to_string();

        // Check if the player's move matches any of the theory moves
        theory_moves.iter().any(|entry| entry.move_string == player_uci)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::Chess;

    #[test]
    fn test_handles_missing_book_gracefully() {
        // Proves the engine won't panic if the Polyglot .bin file is missing
        let book = OpeningBook::new("non_existent_path.bin");
        let pos = Chess::default();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        
        // Should safely return false instead of panicking
        assert!(!book.is_book_move(&pos, fen, "e4"));
    }
}