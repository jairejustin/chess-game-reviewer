use serde::Serialize;

/// Emitted on the `"live-engine-info"` event for every throttled UCI info line
/// the live engine produces during infinite analysis.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePayload {
    pub fen: String,
    pub depth: usize,
    pub multipv: usize,
    pub evaluation: String,
    pub pv: Vec<String>,
}
