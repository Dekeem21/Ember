use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::db::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    running: Arc<Mutex<HashSet<String>>>,
}

impl AppState {
    pub fn new(db: Database) -> AppState {
        AppState {
            db: Arc::new(Mutex::new(db)),
            running: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn is_running(&self, game_id: &str) -> bool {
        self.running.lock().expect("running lock").contains(game_id)
    }

    pub fn mark_running(&self, game_id: &str) {
        self.running
            .lock()
            .expect("running lock")
            .insert(game_id.to_string());
    }

    pub fn mark_stopped(&self, game_id: &str) {
        self.running.lock().expect("running lock").remove(game_id);
    }

    pub fn running_games(&self) -> Vec<String> {
        self.running
            .lock()
            .expect("running lock")
            .iter()
            .cloned()
            .collect()
    }
}
