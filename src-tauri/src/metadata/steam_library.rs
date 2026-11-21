use super::http;
use crate::time::now_unix;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use ureq::Agent;

const REFRESH_AFTER_SECONDS: i64 = 6 * 3600;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedApp {
    pub app_id: u32,
    pub name: String,
    pub playtime_minutes: u64,
    pub last_played: i64,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedLibrary {
    fetched_at: i64,
    apps: Vec<OwnedApp>,
}

pub struct SteamLibraryCache {
    path: PathBuf,
}

impl SteamLibraryCache {
    pub fn new(cache_dir: &Path) -> Self {
        Self {
            path: cache_dir.join("steam-library.json"),
        }
    }

    fn read(&self) -> CachedLibrary {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn apps(&self) -> Vec<OwnedApp> {
        self.read().apps
    }

    pub fn is_stale(&self) -> bool {
        now_unix() - self.read().fetched_at > REFRESH_AFTER_SECONDS
    }

    pub fn refresh(&self, agent: &Agent, key: &str, steam_id: &str) -> bool {
        let Some(apps) = fetch_owned(agent, key, steam_id) else {
            return false;
        };
        let previous = self.read().apps;
        let changed = previous != apps;
        let cached = CachedLibrary {
            fetched_at: now_unix(),
            apps,
        };
        if let Some(directory) = self.path.parent()
            && fs::create_dir_all(directory).is_ok()
            && let Ok(content) = serde_json::to_string(&cached)
        {
            let _ = fs::write(&self.path, content);
        }
        changed
    }
}

fn fetch_owned(agent: &Agent, key: &str, steam_id: &str) -> Option<Vec<OwnedApp>> {
    let response = http::get_json(
        agent,
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/",
        &[
            ("key", key),
            ("steamid", steam_id),
            ("include_appinfo", "1"),
            ("include_played_free_games", "1"),
        ],
        None,
    )?;
    let games = response.get("response")?.get("games")?.as_array()?;
    Some(
        games
            .iter()
            .filter_map(|game| {
                Some(OwnedApp {
                    app_id: u32::try_from(game.get("appid")?.as_u64()?).ok()?,
                    name: game.get("name")?.as_str()?.to_string(),
                    playtime_minutes: game
                        .get("playtime_forever")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    last_played: game
                        .get("rtime_last_played")
                        .and_then(Value::as_i64)
                        .unwrap_or(0),
                })
            })
            .collect(),
    )
}
