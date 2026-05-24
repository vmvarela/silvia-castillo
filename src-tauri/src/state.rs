use std::sync::Mutex;

use crate::engine::ruleset::Ruleset;
use crate::engine::topology::Topology;
use crate::levels::Level;
use crate::progress::{FileStore, Progress};

pub struct AppState {
    pub ruleset: Mutex<Ruleset>,
    pub current_level: Mutex<Option<Level>>,
    pub current_level_idx: Mutex<Option<usize>>,
    pub topology: Mutex<Option<Topology>>,
    pub progress: Mutex<Progress>,
    pub store: FileStore,
}

impl AppState {
    pub fn new() -> Self {
        let store = FileStore::new().unwrap_or_else(|_| {
            // Fallback en tmp si no hay config dir
            FileStore::at(std::path::PathBuf::from("/tmp/silvia-progreso.json"))
        });
        let progress = store.load();
        AppState {
            ruleset: Mutex::new(Ruleset::new()),
            current_level: Mutex::new(None),
            current_level_idx: Mutex::new(None),
            topology: Mutex::new(None),
            progress: Mutex::new(progress),
            store,
        }
    }
}
