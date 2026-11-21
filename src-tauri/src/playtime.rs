use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const MIN_SECONDS_FOR_LAST_PLAYED: u64 = 60;
const KEPT_INTERVALS: usize = 300;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedPlaytime {
    pub seconds: u64,
    pub last_played: i64,
}

pub struct PlaySession {
    pub id: String,
    pub game_id: String,
    pub seconds: u64,
    pub ended_at: i64,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct StoredPlaytime {
    games: HashMap<String, TrackedPlaytime>,
    imported_sessions: HashSet<String>,
    intervals: HashMap<String, Vec<(i64, i64)>>,
}

pub struct PlaytimeStore {
    path: PathBuf,
    stored: StoredPlaytime,
}

fn overlap(first: (i64, i64), second: (i64, i64)) -> i64 {
    (first.1.min(second.1) - first.0.max(second.0)).max(0)
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
                        ..StoredPlaytime::default()
                    })
            })
            .unwrap_or_default();
        Self { path, stored }
    }

    pub fn get(&self, game_id: &str) -> Option<TrackedPlaytime> {
        self.stored.games.get(game_id).copied()
    }

    fn remember_interval(&mut self, game_id: &str, started_at: i64, ended_at: i64) {
        let intervals = self
            .stored
            .intervals
            .entry(game_id.to_string())
            .or_default();
        match intervals.iter_mut().find(|(start, _)| *start == started_at) {
            Some(interval) => interval.1 = ended_at,
            None => intervals.push((started_at, ended_at)),
        }
        if intervals.len() > KEPT_INTERVALS {
            intervals.remove(0);
        }
    }

    pub fn add_tracked(&mut self, game_id: &str, seconds: u64, started_at: i64, now: i64) {
        let entry = self.stored.games.entry(game_id.to_string()).or_default();
        entry.seconds += seconds;
        if now - started_at >= MIN_SECONDS_FOR_LAST_PLAYED as i64 {
            entry.last_played = entry.last_played.max(now);
        }
        self.remember_interval(game_id, started_at, now);
    }

    fn already_counted(&self, game_id: &str, interval: (i64, i64)) -> bool {
        let length = (interval.1 - interval.0).max(1);
        self.stored.intervals.get(game_id).is_some_and(|known| {
            known
                .iter()
                .any(|existing| overlap(*existing, interval) * 2 >= length)
        })
    }

    pub fn import(&mut self, sessions: &[PlaySession]) -> bool {
        let mut imported = false;
        for session in sessions {
            if !self.stored.imported_sessions.insert(session.id.clone()) {
                continue;
            }
            imported = true;
            let interval = (session.ended_at - session.seconds as i64, session.ended_at);
            if self.already_counted(&session.game_id, interval) {
                continue;
            }
            let entry = self
                .stored
                .games
                .entry(session.game_id.clone())
                .or_default();
            entry.seconds += session.seconds;
            if session.seconds >= MIN_SECONDS_FOR_LAST_PLAYED {
                entry.last_played = entry.last_played.max(session.ended_at);
            }
            self.remember_interval(&session.game_id, interval.0, interval.1);
        }
        imported
    }

    pub fn intervals_since(&self, since: i64) -> Vec<(String, i64, i64)> {
        self.stored
            .intervals
            .iter()
            .flat_map(|(game_id, intervals)| {
                intervals
                    .iter()
                    .filter(move |(_, end)| *end >= since)
                    .map(move |(start, end)| (game_id.clone(), *start, *end))
            })
            .collect()
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

    fn session(id: &str, seconds: u64, ended_at: i64) -> PlaySession {
        PlaySession {
            id: id.into(),
            game_id: "desktop:skate3recomp".into(),
            seconds,
            ended_at,
        }
    }

    #[test]
    fn imports_each_session_once_and_skips_sessions_konsol_already_counted() {
        let directory =
            std::env::temp_dir().join(format!("konsol-playtime-{}", std::process::id()));
        let mut store = PlaytimeStore::load(&directory);
        store.add_tracked("desktop:skate3recomp", 600, 1_000, 1_600);
        assert!(store.import(&[session("journal#1600", 590, 1_605)]));
        assert_eq!(
            store
                .get("desktop:skate3recomp")
                .map(|tracked| tracked.seconds),
            Some(600)
        );

        assert!(store.import(&[session("journal#9000", 4982, 9_000)]));
        assert!(!store.import(&[session("journal#9000", 4982, 9_000)]));
        assert!(store.import(&[session("journal#9900", 30, 9_900)]));
        let tracked = store.get("desktop:skate3recomp").expect("tracked");
        assert_eq!(tracked.seconds, 600 + 4982 + 30);
        assert_eq!(tracked.last_played, 9_000);

        store.save();
        assert_eq!(
            PlaytimeStore::load(&directory)
                .get("desktop:skate3recomp")
                .map(|tracked| tracked.seconds),
            Some(5612)
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
