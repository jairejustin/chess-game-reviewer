use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Debug,
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Engine,
    Book,
}

#[derive(
    Serialize, Deserialize, Clone, Debug,
)]
pub struct Installed {
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub version: String,
    /// Relative to the data root, or absolute for external/dev files
    /// (PathBuf::join with an absolute path replaces the base).
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Registry {
    pub schema: u32,
    pub installed: Vec<Installed>,
    pub active_engine: Option<String>,
    pub active_book: Option<String>,
}
