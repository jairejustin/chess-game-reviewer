use crate::models::resource::{
    Installed, Kind, Registry,
};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

pub struct ResourceStore {
    root: PathBuf,
    registry: Mutex<Registry>,
}

impl ResourceStore {
    pub fn open(
        app: &AppHandle,
    ) -> Result<Self, String> {
        let root = match std::env::var_os(
            "CHESS_GAME_REVIEWER_DATA_DIR",
        ) {
            Some(p) => PathBuf::from(p),
            None => app
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?,
        };
        fs::create_dir_all(&root)
            .map_err(|e| e.to_string())?;
        let reg_path = root.join("registry.json");
        let registry = match fs::read_to_string(
            &reg_path,
        ) {
            Ok(s) => serde_json::from_str(&s)
                .unwrap_or_else(|_| {
                    let _ = fs::rename(
                        &reg_path,
                        root.join(
                            "registry.json.bak",
                        ),
                    );
                    Registry::default()
                }),
            Err(_) => Registry::default(),
        };
        Ok(Self {
            root,
            registry: Mutex::new(registry),
        })
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    fn save(
        &self,
        reg: &Registry,
    ) -> Result<(), String> {
        // atomic: write temp, then rename
        let tmp =
            self.root.join("registry.json.tmp");
        let json =
            serde_json::to_string_pretty(reg)
                .map_err(|e| e.to_string())?;
        fs::write(&tmp, json)
            .map_err(|e| e.to_string())?;
        fs::rename(
            &tmp,
            self.root.join("registry.json"),
        )
        .map_err(|e| e.to_string())
    }

    pub fn register(
        &self,
        item: Installed,
        activate_if_none: bool,
    ) -> Result<(), String> {
        let mut reg =
            self.registry.lock().unwrap();
        reg.installed.retain(|r| r.id != item.id);
        if activate_if_none {
            match item.kind {
                Kind::Engine
                    if reg
                        .active_engine
                        .is_none() =>
                {
                    reg.active_engine =
                        Some(item.id.clone())
                }
                Kind::Book
                    if reg
                        .active_book
                        .is_none() =>
                {
                    reg.active_book =
                        Some(item.id.clone())
                }
                _ => {}
            }
        }
        reg.installed.push(item);
        self.save(&reg)
    }

    fn resolve(
        &self,
        id: Option<&String>,
    ) -> Option<PathBuf> {
        let reg = self.registry.lock().unwrap();
        let r = reg
            .installed
            .iter()
            .find(|r| Some(&r.id) == id)?;
        Some(self.root.join(&r.path))
    }

    pub fn active_engine_path(
        &self,
    ) -> Option<PathBuf> {
        let id = self
            .registry
            .lock()
            .unwrap()
            .active_engine
            .clone();
        self.resolve(id.as_ref())
    }

    pub fn active_book_path(
        &self,
    ) -> Option<PathBuf> {
        let id = self
            .registry
            .lock()
            .unwrap()
            .active_book
            .clone();
        self.resolve(id.as_ref())
    }
}
