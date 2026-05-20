use crate::engine::classify::classify;
use crate::engine::uci_engine::{
    Evaluation, UciEngine,
};
use crate::models::game::AnalyzedMove;
use crate::pgn::PgnVisitor;
use pgn_reader::Reader;
use std::io::Cursor;

#[tauri::command]
pub fn analyze_game(
    pgn: String,
) -> Result<Vec<AnalyzedMove>, String> {
    let mut visitor = PgnVisitor::new();
    let mut reader =
        Reader::new(Cursor::new(pgn.as_bytes()));

    let positions =
        match reader.read_game(&mut visitor) {
            Ok(Some(p)) => p,
            _ => {
                return Err("Failed to parse PGN"
                    .to_string())
            }
        };

    let binary_path = "/home/j/Documents/Personal Projects/chess-analyze/src-tauri/binaries/theoria-x86_64-unknown-linux-gnu";
    let mut engine = UciEngine::new(binary_path);
    let depth = 15;

    let initial = engine.analyze_position(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 
        depth
    );

    let mut prev_eval = match initial.0 {
        Evaluation::Cp(cp) => cp,
        Evaluation::Mate(m) => {
            if m > 0 {
                10000
            } else {
                -10000
            }
        }
    };

    let mut analysis_results = Vec::new();
    let mut ply_count = 1;

    for (san, fen) in positions {
        let (
            played_eval,
            best_move,
            pv,
            multi_pv_evals,
        ) = engine.analyze_position(&fen, depth);

        let played_cp = match played_eval {
            Evaluation::Cp(cp) => cp,
            Evaluation::Mate(m) => {
                if m > 0 {
                    10000
                } else {
                    -10000
                }
            }
        };

        let best_eval = prev_eval;

        let normalized_cp = if ply_count % 2 != 0
        {
            -played_cp
        } else {
            played_cp
        };

        // Pass multi_pv_evals here
        let classification = classify(
            prev_eval,
            normalized_cp,
            best_eval,
            &multi_pv_evals,
        );

        let analyzed_move = AnalyzedMove {
            ply: ply_count,
            san,
            fen,
            played_eval: played_cp,
            best_move_eval: best_eval,
            best_move_san: best_move,
            classification,
            principal_variation: pv,
        };

        analysis_results.push(analyzed_move);

        prev_eval = normalized_cp;
        ply_count += 1;
    }

    engine.quit();

    Ok(analysis_results)
}
