use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KonsolConfig {
    pub manual_games: Vec<ManualGame>,
    pub hidden_game_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualGame {
    pub id: String,
    pub title: String,
    pub executable: PathBuf,
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
        fs::write(&temporary_path, content)
            .and_then(|()| fs::rename(&temporary_path, &self.path))
            .map_err(|error| format!("Impossible d'écrire la config : {error}"))
    }
}
