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
#[serde(rename_all = "camelCase")]
pub struct AnalyzedMove {
    pub ply: u32,
    pub san: String,
    pub fen: String,
    pub uci: String,
    pub played_eval: i32,
    pub mate_in: Option<i32>,
    pub best_mate_in: Option<i32>,
    pub prev_best_eval: i32,
    pub best_move_san: String,
    pub classification: MoveBadge,
    pub principal_variation: Vec<String>,
}

/// An aggregated tally tracking the total number of each
/// classification badge awarded during the game.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
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

impl MoveCounts {
    /// Increments the counter for the given badge.
    pub fn tally(&mut self, badge: &MoveBadge) {
        match badge {
            MoveBadge::Brilliant => {
                self.brilliant += 1
            }
            MoveBadge::Great => self.great += 1,
            MoveBadge::Best => self.best += 1,
            MoveBadge::Excellent => {
                self.excellent += 1
            }
            MoveBadge::Good => self.good += 1,
            MoveBadge::Inaccuracy => {
                self.inaccuracy += 1
            }
            MoveBadge::Mistake => {
                self.mistake += 1
            }
            MoveBadge::Blunder => {
                self.blunder += 1
            }
            MoveBadge::Miss => self.miss += 1,
            MoveBadge::Book => self.book += 1,
            MoveBadge::Forced => self.forced += 1,
        }
    }
}

/// The parsed PGN header information including the players,
/// date, event, and final match result.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    pub white_accuracy: f64,
    pub black_accuracy: f64,
    pub move_counts_white: MoveCounts,
    pub move_counts_black: MoveCounts,
    pub metadata: GameMetadata,
    pub moves: Vec<AnalyzedMove>,
}

/// Analysis progress event emitted once per ply.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisProgress {
    pub current_ply: u32,
    pub total_plies: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_move_counts_are_all_zero() {
        let counts = MoveCounts::default();
        assert_eq!(counts.brilliant, 0);
        assert_eq!(counts.great, 0);
        assert_eq!(counts.best, 0);
        assert_eq!(counts.excellent, 0);
        assert_eq!(counts.good, 0);
        assert_eq!(counts.inaccuracy, 0);
        assert_eq!(counts.mistake, 0);
        assert_eq!(counts.blunder, 0);
        assert_eq!(counts.miss, 0);
        assert_eq!(counts.book, 0);
        assert_eq!(counts.forced, 0);
    }

    #[test]
    fn tally_increments_correct_field_for_each_badge(
    ) {
        let badges = [
            (MoveBadge::Brilliant, "brilliant"),
            (MoveBadge::Great, "great"),
            (MoveBadge::Best, "best"),
            (MoveBadge::Excellent, "excellent"),
            (MoveBadge::Good, "good"),
            (MoveBadge::Inaccuracy, "inaccuracy"),
            (MoveBadge::Mistake, "mistake"),
            (MoveBadge::Blunder, "blunder"),
            (MoveBadge::Miss, "miss"),
            (MoveBadge::Book, "book"),
            (MoveBadge::Forced, "forced"),
        ];

        for (badge, label) in badges {
            let mut counts =
                MoveCounts::default();
            counts.tally(&badge);

            // Only the tallied field should be 1;
            // every other field must remain 0.
            let total = counts.brilliant
                + counts.great
                + counts.best
                + counts.excellent
                + counts.good
                + counts.inaccuracy
                + counts.mistake
                + counts.blunder
                + counts.miss
                + counts.book
                + counts.forced;

            assert_eq!(
                total, 1,
                "{} tally incremented more than one field",
                label
            );
        }
    }

    #[test]
    fn tally_accumulates_across_multiple_calls() {
        let mut counts = MoveCounts::default();

        counts.tally(&MoveBadge::Blunder);
        counts.tally(&MoveBadge::Blunder);
        counts.tally(&MoveBadge::Mistake);
        counts.tally(&MoveBadge::Best);

        assert_eq!(counts.blunder, 2);
        assert_eq!(counts.mistake, 1);
        assert_eq!(counts.best, 1);

        // Everything else untouched
        assert_eq!(counts.brilliant, 0);
        assert_eq!(counts.great, 0);
    }
}
