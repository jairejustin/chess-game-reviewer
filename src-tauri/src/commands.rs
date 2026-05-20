use crate::engine::classify::classify;
use crate::engine::uci_engine::{
    Evaluation, UciEngine,
};
use crate::models::game::AnalyzedMove;
use crate::pgn::PgnVisitor;
use pgn_reader::Reader;
use std::io::Cursor;

/// Calculates standard material balance from a FEN string.
/// Positive means White is up material, Negative means Black is up material.
pub fn calculate_material_balance(
    fen: &str,
) -> i32 {
    let board_part = fen
        .split_whitespace()
        .next()
        .unwrap_or("");
    let mut score = 0;

    for c in board_part.chars() {
        match c {
            'P' => score += 1,
            'N' | 'B' => score += 3,
            'R' => score += 5,
            'Q' => score += 9,
            'p' => score -= 1,
            'n' | 'b' => score -= 3,
            'r' => score -= 5,
            'q' => score -= 9,
            _ => {}
        }
    }
    score
}

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

    let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let initial = engine
        .analyze_position(initial_fen, depth);

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

    let mut prev_fen = initial_fen.to_string();
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

        // Material sacrifice calculation
        let prev_material =
            calculate_material_balance(&prev_fen);
        let current_material =
            calculate_material_balance(&fen);
        let raw_material_delta =
            current_material - prev_material;

        // Normalize: Negative means the player who just moved lost material
        let material_delta = if ply_count % 2 != 0
        {
            raw_material_delta // White's turn
        } else {
            -raw_material_delta // Black's turn
        };

        let best_eval = prev_eval;

        let normalized_cp = if ply_count % 2 != 0
        {
            -played_cp
        } else {
            played_cp
        };

        // Pass everything into the classifier
        let classification = classify(
            prev_eval,
            normalized_cp,
            best_eval,
            &multi_pv_evals,
            material_delta,
        );

        let analyzed_move = AnalyzedMove {
            ply: ply_count,
            san,
            fen: fen.clone(),
            played_eval: played_cp,
            best_move_eval: best_eval,
            best_move_san: best_move,
            classification,
            principal_variation: pv,
        };

        analysis_results.push(analyzed_move);

        prev_eval = normalized_cp;
        prev_fen = fen;
        ply_count += 1;
    }

    engine.quit();

    Ok(analysis_results)
}
