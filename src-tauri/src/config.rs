use crate::metadata::ApiKeys;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KonsolConfig {
    pub manual_games: Vec<ManualGame>,
    pub hidden_game_ids: Vec<String>,
    pub custom_titles: BTreeMap<String, String>,
    pub steam_grid_db_api_key: Option<String>,
    pub steam_web_api_key: Option<String>,
    pub preferences: Preferences,
    pub game_folders: Option<Vec<PathBuf>>,
    pub collections: Vec<Collection>,
    pub playtime_offsets: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub name: String,
    pub game_ids: Vec<String>,
}

impl KonsolConfig {
    pub fn api_keys(&self) -> ApiKeys {
        ApiKeys {
            steam_grid_db: self.steam_grid_db_api_key.clone(),
            steam_web: self.steam_web_api_key.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualGame {
    pub id: String,
    pub title: String,
    pub executable: PathBuf,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub cover: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub launch_at_login: bool,
    pub minimize_during_session: bool,
    pub keep_running_in_background: bool,
    pub accent_follows_game: bool,
    pub reduce_motion: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            minimize_during_session: false,
            keep_running_in_background: true,
            accent_follows_game: true,
            reduce_motion: false,
        }
    }
}

pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<KonsolConfig, String> {
        match fs::read_to_string(&self.path) {
            Ok(content) => serde_json::from_str(&content)
                .map_err(|error| format!("Config illisible ({}) : {error}", self.path.display())),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(KonsolConfig::default()),
            Err(error) => Err(format!("Impossible de lire la config : {error}")),
        }
    }

    pub fn save(&self, config: &KonsolConfig) -> Result<(), String> {
        if let Some(directory) = self.path.parent() {
            fs::create_dir_all(directory)
                .map_err(|error| format!("Impossible de créer le dossier de config : {error}"))?;
        }
        let content = serde_json::to_string_pretty(config)
            .map_err(|error| format!("Impossible d'écrire la config : {error}"))?;
        let temporary_path = self.path.with_extension("json.tmp");
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temporary_path)
            .and_then(|mut file| file.write_all(content.as_bytes()))
            .and_then(|()| fs::rename(&temporary_path, &self.path))
            .map_err(|error| format!("Impossible d'écrire la config : {error}"))
    }
}
