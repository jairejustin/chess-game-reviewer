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