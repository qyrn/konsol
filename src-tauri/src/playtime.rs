use crate::journal_sessions::JournalSession;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedPlaytime {
    pub seconds: u64,
    pub last_played: i64,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct StoredPlaytime {
    games: HashMap<String, TrackedPlaytime>,
    imported_sessions: HashSet<String>,
}

pub struct PlaytimeStore {
    path: PathBuf,
    stored: StoredPlaytime,
}

impl PlaytimeStore {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("playtime.json");
        let content = fs::read_to_string(&path).unwrap_or_default();
        let stored = serde_json::from_str::<StoredPlaytime>(&content)
            .ok()
            .filter(|stored| !stored.games.is_empty() || !stored.imported_sessions.is_empty())
            .or_else(|| {
                serde_json::from_str::<HashMap<String, TrackedPlaytime>>(&content)
                    .ok()
                    .map(|games| StoredPlaytime {
                        games,
                        imported_sessions: HashSet::new(),
                    })
            })
            .unwrap_or_default();
        Self { path, stored }
    }

    pub fn get(&self, game_id: &str) -> Option<TrackedPlaytime> {
        self.stored.games.get(game_id).copied()
    }

    pub fn add(&mut self, game_id: &str, seconds: u64, ended_at: i64) {
        let entry = self.stored.games.entry(game_id.to_string()).or_default();
        entry.seconds += seconds;
        entry.last_played = entry.last_played.max(ended_at);
    }

    pub fn import(
        &mut self,
        sessions: &[JournalSession],
        known_desktop_ids: &HashSet<String>,
    ) -> bool {
        let mut imported = false;
        for session in sessions {
            let game_id = format!("desktop:{}", session.desktop_stem);
            if known_desktop_ids.contains(&game_id)
                && self.stored.imported_sessions.insert(session.id.clone())
            {
                self.add(&game_id, session.seconds, session.ended_at);
                imported = true;
            }
        }
        imported
    }

    pub fn save(&self) {
        if let Some(directory) = self.path.parent()
            && fs::create_dir_all(directory).is_ok()
            && let Ok(content) = serde_json::to_string(&self.stored)
        {
            let _ = fs::write(&self.path, content);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_each_menu_session_once() {
        let directory =
            std::env::temp_dir().join(format!("konsol-playtime-{}", std::process::id()));
        let mut store = PlaytimeStore::load(&directory);
        let sessions = [JournalSession {
            id: "app-skate3recomp@a#10".into(),
            desktop_stem: "skate3recomp".into(),
            seconds: 4982,
            ended_at: 10,
        }];
        let known: HashSet<String> = ["desktop:skate3recomp".to_string()].into();
        assert!(store.import(&sessions, &known));
        assert!(!store.import(&sessions, &known));
        assert_eq!(
            store
                .get("desktop:skate3recomp")
                .map(|tracked| tracked.seconds),
            Some(4982)
        );
        store.save();
        assert_eq!(
            PlaytimeStore::load(&directory)
                .get("desktop:skate3recomp")
                .map(|tracked| tracked.last_played),
            Some(10)
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
