use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedPlaytime {
    pub seconds: u64,
    pub last_played: i64,
}

pub struct PlaytimeStore {
    path: PathBuf,
    entries: HashMap<String, TrackedPlaytime>,
}

impl PlaytimeStore {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("playtime.json");
        let entries = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();
        Self { path, entries }
    }

    pub fn get(&self, game_id: &str) -> Option<TrackedPlaytime> {
        self.entries.get(game_id).copied()
    }

    pub fn add(&mut self, game_id: &str, seconds: u64, now: i64) {
        let entry = self.entries.entry(game_id.to_string()).or_default();
        entry.seconds += seconds;
        entry.last_played = now;
    }

    pub fn save(&self) {
        if let Some(directory) = self.path.parent()
            && fs::create_dir_all(directory).is_ok()
            && let Ok(content) = serde_json::to_string(&self.entries)
        {
            let _ = fs::write(&self.path, content);
        }
    }
}
