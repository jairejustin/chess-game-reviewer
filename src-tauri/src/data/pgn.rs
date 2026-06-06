use crate::models::game::GameMetadata;
use pgn_reader::{SanPlus, Visitor};
use shakmaty::fen::Fen;
use shakmaty::{
    CastlingMode, Chess, EnPassantMode, Position,
};
use std::ops::ControlFlow;

pub struct PgnVisitor {
    board: Chess,
    positions: Vec<(String, String, String)>,
    metadata: GameMetadata,
}

impl PgnVisitor {
    pub fn new() -> Self {
        PgnVisitor {
            board: Chess::default(),
            positions: Vec::new(),
            metadata: GameMetadata {
                white: "Unknown".to_string(),
                black: "Unknown".to_string(),
                date: "????.??.??".to_string(),
                result: "*".to_string(),
                event: None,
            },
        }
    }
}

impl Visitor for PgnVisitor {
    type Tags = ();
    type Movetext = ();

    type Output = (
        GameMetadata,
        Vec<(String, String, String)>,
    );

    fn begin_tags(
        &mut self,
    ) -> ControlFlow<Self::Output, Self::Tags>
    {
        self.board = Chess::default();
        self.positions = Vec::new();
        self.metadata = GameMetadata {
            white: "Unknown".to_string(),
            black: "Unknown".to_string(),
            date: "????.??.??".to_string(),
            result: "*".to_string(),
            event: None,
        };
        ControlFlow::Continue(())
    }

    fn tag(
        &mut self,
        _tags: &mut Self::Tags,
        name: &[u8],
        value: pgn_reader::RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
        if let Ok(k) = std::str::from_utf8(name) {
            if let Ok(v) = value.decode_utf8() {
                let val = v.into_owned();
                match k {
                    "White" => {
                        self.metadata.white = val
                    }
                    "Black" => {
                        self.metadata.black = val
                    }
                    "Date" => {
                        self.metadata.date = val
                    }
                    "Result" => {
                        self.metadata.result = val
                    }
                    "Event" => {
                        self.metadata.event =
                            Some(val)
                    }
                    _ => {}
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn begin_movetext(
        &mut self,
        _tags: Self::Tags,
    ) -> ControlFlow<Self::Output, Self::Movetext>
    {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        san_plus: SanPlus,
    ) -> ControlFlow<Self::Output> {
        if let Ok(m) =
            san_plus.san.to_move(&self.board)
        {
            let san_string = san_plus.to_string();

            let uci_string = shakmaty::uci::UciMove::from_move(m, CastlingMode::Standard).to_string();

            self.board = self
                .board
                .clone()
                .play(m)
                .expect(""); // san_plus.san.to_move(&self.board) already validated this

            let fen = Fen::from_position(
                &self.board.clone(),
                EnPassantMode::Legal,
            )
            .to_string();

            self.positions.push((
                san_string, fen, uci_string,
            ));
        }
        ControlFlow::Continue(())
    }

    fn end_game(
        &mut self,
        _movetext: Self::Movetext,
    ) -> Self::Output {
        (
            self.metadata.clone(),
            self.positions.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgn_reader::Reader;
    use std::io::Cursor;

    #[test]
    fn ignores_illegal_moves_in_san() {
        let pgn = "1. Qxf7#"; // ChatGPT ahh move
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(
            Cursor::new(pgn.as_bytes()),
        );

        let positions = reader
            .read_game(&mut visitor)
            .unwrap()
            .unwrap()
            .1;

        // Because the move fails `to_move`, it skips the push, leaving 0 positions.
        assert_eq!(positions.len(), 0);
    }

    #[test]
    fn ignores_invalid_utf8_tags() {
        use pgn_reader::Reader;
        use std::io::Cursor;

        let mut visitor = PgnVisitor::new();

        let mut pgn_bytes = b"[White \"".to_vec();
        pgn_bytes.push(0xFF);
        pgn_bytes.extend_from_slice(b"\"]\n");

        pgn_bytes.push(b'[');
        pgn_bytes.push(0xFF);
        pgn_bytes.extend_from_slice(
            b" \"Magnus Carlsen\"]\n\n1. e4",
        );

        let mut reader =
            Reader::new(Cursor::new(pgn_bytes));
        let _ = reader.read_game(&mut visitor);

        assert_eq!(
            visitor.metadata.white,
            "Unknown"
        );
    }
    #[test]
    fn parses_four_move_game_into_correct_position_count(
    ) {
        let pgn = "1. e4 e5 2. Nf3 Nc6";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(
            Cursor::new(pgn.as_bytes()),
        );
        let positions = reader
            .read_game(&mut visitor)
            .unwrap()
            .unwrap()
            .1;
        assert_eq!(positions.len(), 4);
    }

    #[test]
    fn first_move_produces_correct_fen() {
        let pgn = "1. e4";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(
            Cursor::new(pgn.as_bytes()),
        );
        let positions = reader
            .read_game(&mut visitor)
            .unwrap()
            .unwrap()
            .1;
        assert_eq!(
            positions[0].1,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        );
    }

    #[test]
    fn san_strings_match_played_moves() {
        let pgn = "1. e4 e5";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(
            Cursor::new(pgn.as_bytes()),
        );
        let positions = reader
            .read_game(&mut visitor)
            .unwrap()
            .unwrap()
            .1;
        assert_eq!(positions[0].0, "e4");
        assert_eq!(positions[1].0, "e5");
    }

    #[test]
    fn parses_metadata_correctly() {
        let pgn = "[White \"Magnus Carlsen\"]\n[Black \"Hikaru Nakamura\"]\n[Result \"1/2-1/2\"]\n\n1. e4 e5";
        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(
            Cursor::new(pgn.as_bytes()),
        );
        let metadata = reader
            .read_game(&mut visitor)
            .unwrap()
            .unwrap()
            .0;
        assert_eq!(
            metadata.white,
            "Magnus Carlsen"
        );
        assert_eq!(
            metadata.black,
            "Hikaru Nakamura"
        );
        assert_eq!(metadata.result, "1/2-1/2");
    }

    #[test]
    fn parses_full_metadata_and_ignores_unknown_tags(
    ) {
        // Added Date, Event, and an unrecognized "Site" tag
        let pgn = "[White \"Magnus Carlsen\"]\n[Black \"Hikaru Nakamura\"]\n[Result \"1/2-1/2\"]\n[Date \"2023.12.01\"]\n[Event \"Champions Chess Tour\"]\n[Site \"Toronto, CAN\"]\n\n1. e4 e5";

        let mut visitor = PgnVisitor::new();
        let mut reader = Reader::new(
            Cursor::new(pgn.as_bytes()),
        );
        let metadata = reader
            .read_game(&mut visitor)
            .unwrap()
            .unwrap()
            .0;

        assert_eq!(
            metadata.white,
            "Magnus Carlsen"
        );
        assert_eq!(
            metadata.black,
            "Hikaru Nakamura"
        );
        assert_eq!(metadata.result, "1/2-1/2");
        assert_eq!(metadata.date, "2023.12.01");
        assert_eq!(
            metadata.event,
            Some(
                "Champions Chess Tour"
                    .to_string()
            )
        );
    }
}
