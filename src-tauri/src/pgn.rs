use pgn_reader::{Visitor, SanPlus};
use shakmaty::{Chess, Position, EnPassantMode};
use shakmaty::fen::Fen;
use std::ops::ControlFlow;

pub struct PgnVisitor {
    board: Chess,
    positions: Vec<(String, String)>,
}

impl PgnVisitor {
    pub fn new() -> Self {
        PgnVisitor {
            board: Chess::default(),
            positions: Vec::new(),
        }
    }
}

impl Visitor for PgnVisitor {
    type Tags = ();
    type Movetext = ();
    type Output = Vec<(String, String)>;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.board = Chess::default();
        self.positions = Vec::new();
        ControlFlow::Continue(())
    }

    fn begin_movetext(
        &mut self,
        _tags: Self::Tags,
    ) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        san_plus: SanPlus,
    ) -> ControlFlow<Self::Output> {
        if let Ok(m) = san_plus.san.to_move(&self.board) {
            let san_string = san_plus.to_string();
            self.board = self.board.clone().play(m).unwrap();
            let fen = Fen::from_position(
                &self.board.clone(),
                EnPassantMode::Legal,
            ).to_string();
            self.positions.push((san_string, fen));
        }
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, _movetext: Self::Movetext) -> Self::Output {
        self.positions.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgn_reader::Reader;
    use std::io::Cursor;

    #[test]
    fn parses_four_move_game_into_correct_position_count() {
        let pgn = "1. e4 e5 2. Nf3 Nc6";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));
        let positions = reader.read_game(&mut visitor).unwrap().unwrap();
        assert_eq!(positions.len(), 4);
    }

    #[test]
    fn first_move_produces_correct_fen() {
        let pgn = "1. e4";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));
        let positions = reader.read_game(&mut visitor).unwrap().unwrap();
        assert_eq!(
            positions[0].1,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        );
    }

    #[test]
    fn san_strings_match_played_moves() {
        let pgn = "1. e4 e5";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));
        let positions = reader.read_game(&mut visitor).unwrap().unwrap();
        assert_eq!(positions[0].0, "e4");
        assert_eq!(positions[1].0, "e5");
    }

    #[test]
    fn handles_empty_pgn_gracefully() {
        let pgn = "";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));
        let result = reader.read_game(&mut visitor).unwrap();
        assert!(result.is_none());
    }
}