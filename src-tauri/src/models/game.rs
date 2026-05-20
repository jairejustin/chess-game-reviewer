use serde::Serialize;

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
}

#[derive(Serialize)]
pub struct AnalyzedMove {
    pub ply: u32,
    pub san: String,
    pub fen: String,
    pub played_eval: i32,
    pub best_move_eval: i32,
    pub best_move_san: String,
    pub classification: MoveBadge,
    pub principal_variation: Vec<String>,
}

#[derive(Serialize)]
pub struct GameMetadata {
    pub white: String,
    pub black: String,
    pub date: String,
    pub result: String,
    pub event: Option<String>,
}

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
}

#[derive(Serialize)]
pub struct AnalysisSummary {
    pub white_accuracy: f64,
    pub black_accuracy: f64,
    pub move_counts: MoveCounts,
    pub metadata: GameMetadata,
}
