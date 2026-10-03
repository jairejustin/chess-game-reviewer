use crate::models::resource::{Installed, Kind};
use crate::resources::store::ResourceStore;

impl ResourceStore {
    /// Registers the manual files from src-tauri/core so the app keeps
    /// working while the real installer doesn't exist yet.
    /// Compiled only in debug builds.
    pub fn seed_dev_resources(&self) {
        let core = match std::env::current_dir() {
            Ok(d) => d.join("core"),
            Err(_) => return,
        };
        let engine = core.join(
            "engine/stockfish-ubuntu-x86-64-bmi2",
        );
        let book = core.join("database/book.bin");
        if engine.exists() {
            let _ = self.register(
                Installed {
                    id: "dev-stockfish".into(),
                    kind: Kind::Engine,
                    name: "Stockfish (dev)"
                        .into(),
                    version: "dev".into(),
                    path: engine,
                },
                true,
            );
        }
        if book.exists() {
            let _ = self.register(
                Installed {
                    id: "dev-book".into(),
                    kind: Kind::Book,
                    name: "book.bin (dev)".into(),
                    version: "dev".into(),
                    path: book,
                },
                true,
            );
        }
    }
}
