use crate::models::resource::Kind;
use serde::{Deserialize, Serialize};

/// The remote catalog of installable resources.
#[derive(
    Debug, Clone, Serialize, Deserialize,
)]
pub struct Manifest {
    pub schema: u32,
    pub resources: Vec<ResourceEntry>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize,
)]
pub struct ResourceEntry {
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: String,
    /// Ids that must be installed first.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Ordered best-first; the first match for the machine wins.
    pub artifacts: Vec<Artifact>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveKind {
    /// The download is the final file (for opening books).
    None,
    Zip,
    Tar,
    TarGz,
}

#[derive(
    Debug, Clone, Serialize, Deserialize,
)]
pub struct Artifact {
    /// e.g. "linux-x86_64"
    pub platform: String,
    /// CPU features required, e.g. ["bmi2"]. Empty means runs anywhere.
    #[serde(default)]
    pub cpu: Vec<String>,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub size: u64,
    pub archive: ArchiveKind,
    /// Main file, relative to the install dir. For archives it is the path
    /// inside the archive; for `ArchiveKind::None` it is the saved file name.
    pub entry: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "schema": 1,
        "resources": [{
            "id": "stockfish-17.1",
            "kind": "engine",
            "name": "Stockfish 17.1",
            "version": "17.1",
            "artifacts": [
                { "platform": "linux-x86_64", "cpu": ["bmi2"],
                  "archive": "tar", "url": "https://example.com/a.tar",
                  "sha256": "00", "entry": "stockfish/sf-bmi2" },
                { "platform": "linux-x86_64",
                  "archive": "tar_gz", "url": "https://example.com/b.tar.gz",
                  "sha256": "00", "entry": "stockfish/sf" }
            ]
        }]
    }"#;

    #[test]
    fn parses_a_manifest() {
        let m: Manifest =
            serde_json::from_str(SAMPLE).unwrap();
        let e = &m.resources[0];
        assert_eq!(e.id, "stockfish-17.1");
        assert_eq!(e.kind, Kind::Engine);
        assert_eq!(e.artifacts.len(), 2);
        assert_eq!(
            e.artifacts[0].archive,
            ArchiveKind::Tar
        );
        assert_eq!(
            e.artifacts[0].cpu,
            vec!["bmi2"]
        );
    }

    #[test]
    fn optional_fields_default() {
        let m: Manifest =
            serde_json::from_str(SAMPLE).unwrap();
        let e = &m.resources[0];
        assert_eq!(e.description, "");
        assert!(e.requires.is_empty());
        assert!(e.artifacts[1].cpu.is_empty());
        assert_eq!(e.artifacts[1].size, 0);
    }

    #[test]
    fn unknown_archive_kind_is_rejected() {
        let bad = SAMPLE.replace("tar_gz", "rar");
        assert!(
            serde_json::from_str::<Manifest>(
                &bad
            )
            .is_err()
        );
    }
}
