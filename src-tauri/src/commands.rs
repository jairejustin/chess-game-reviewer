// use crate::pgn::PgnVisitor;
use crate::engine::process::{test_engine_communication};
// use std::io::Cursor;
// use pgn_reader::Reader;

#[tauri::command]
pub fn analyze_game(_pgn: String) {
    test_engine_communication();
    // let mut visitor = PgnVisitor::new();
    // let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));
    
    // if let Ok(Some(positions)) = reader.read_game(&mut visitor) {
    //     for (san, fen) in &positions {
    //         println!("{} -> {}", san, fen);
    //     }
    // }
}