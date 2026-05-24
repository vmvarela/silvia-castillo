use std::sync::Mutex;
use crate::engine::ruleset::Ruleset;

pub struct AppState {
    pub ruleset: Mutex<Ruleset>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            ruleset: Mutex::new(Ruleset::new()),
        }
    }
}
