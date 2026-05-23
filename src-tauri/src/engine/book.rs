use shakmaty::{Chess, Position};
use std::path::Path;

pub struct OpeningBook {
    is_loaded: bool,
}

impl OpeningBook {
    pub fn new(book_path: &str) -> Self {
        // Just check if the file exists for now to prevent crashes
        let is_loaded =
            Path::new(book_path).exists();
        OpeningBook { is_loaded }
    }

    pub fn is_book_move(
        &self,
        pos: &Chess,
        _played_san: &str,
    ) -> bool {
        if !self.is_loaded {
            return false;
        }

        // Stub: Treat the first 10 plies as book moves for testing
        // to bypass the engine math correctly.
        pos.fullmoves().get() <= 5
    }
}
