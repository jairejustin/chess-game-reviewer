use serde::Serialize;

/// Represents the qualitative grade awarded to a chess move
#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MoveBadge {
    Book,
    Brilliant,
    Great,
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
    Miss,
    Forced,
}

/// A payload for a single half-move (a ply), containing its
/// evaluation, classification, and SAN (Standard Algebraic Notation).
#[derive(Serialize)]
pub struct AnalyzedMove {
    pub ply: u32,
    pub san: String,
    pub fen: String,
    pub played_eval: i32,
    pub prev_best_eval: i32,
    pub best_move_san: String,
    pub classification: MoveBadge,
    pub principal_variation: Vec<String>,
}

/// An aggregated tally tracking the total number of each
/// classification badge awarded during the game.
#[derive(Serialize)]
pub struct MoveCounts {
    pub brilliant: u32,
    pub great: u32,
    pub best: u32,
    pub excellent: u32,
    pub good: u32,
    pub inaccuracy: u32,
    pub mistake: u32,
    pub blunder: u32,
    pub miss: u32,
    pub book: u32,
    pub forced: u32,
}

/// The parsed PGN header information including the players,
/// date, event, and final match result.
#[derive(Serialize, Clone)]
pub struct GameMetadata {
    pub white: String,
    pub black: String,
    pub date: String,
    pub result: String,
    pub event: Option<String>,
}

/// The final end-of-game payload containing the total move counts,
/// match metadata, and calculated CAPS accuracy scores for both players
/// (Computer Aggregated Precision Score).
#[derive(Serialize)]
pub struct AnalysisSummary {
    pub white_accuracy: f64,
    pub black_accuracy: f64,
    pub move_counts: MoveCounts,
    pub metadata: GameMetadata,
}
