use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameSource {
    Steam,
    Gog,
    Epic,
    Amazon,
    Desktop,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum ImageSource {
    File(String),
    Url(String),
}

impl ImageSource {
    pub fn local_path(&self) -> Option<PathBuf> {
        match self {
            ImageSource::File(path) => Some(PathBuf::from(path)),
            ImageSource::Url(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchTarget {
    SteamApp(String),
    Heroic {
        runner: &'static str,
        app_name: String,
    },
    Program(PathBuf),
    Command {
        arguments: Vec<String>,
        working_directory: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessMarker {
    EnvVar { name: &'static str, value: String },
    Mentions(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AchievementSource {
    Steam { app_id: String },
    Gog { app_name: String },
    EmulatorStats { path: PathBuf },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AchievementSummary {
    pub unlocked: usize,
    pub total: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub title: String,
    pub source: GameSource,
    pub cover: Option<ImageSource>,
    pub hero: Option<ImageSource>,
    pub accent: Option<String>,
    pub hidden: bool,
    pub achievements: Option<AchievementSummary>,
    #[serde(skip)]
    pub launch: LaunchTarget,
    #[serde(skip)]
    pub process_marker: ProcessMarker,
    #[serde(skip)]
    pub icon_source: Option<PathBuf>,
    #[serde(skip)]
    pub achievement_source: Option<AchievementSource>,
}

pub const KONSOL_GAME_ID_VARIABLE: &str = "KONSOL_GAME_ID";

impl Game {
    pub fn launched_by_konsol(
        id: String,
        title: String,
        source: GameSource,
        launch: LaunchTarget,
    ) -> Self {
        Self {
            process_marker: ProcessMarker::EnvVar {
                name: KONSOL_GAME_ID_VARIABLE,
                value: id.clone(),
            },
            id,
            title,
            source,
            cover: None,
            hero: None,
            accent: None,
            hidden: false,
            achievements: None,
            launch,
            icon_source: None,
            achievement_source: None,
        }
    }

    pub fn needs_store_artwork(&self) -> bool {
        matches!(self.source, GameSource::Desktop | GameSource::Manual)
    }
}
