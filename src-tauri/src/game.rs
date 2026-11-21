use serde::{Deserialize, Serialize};
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
    Rockstar,
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
    AnyOf(Vec<ProcessMarker>),
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetails {
    pub playtime_minutes: Option<u64>,
    pub last_played: Option<i64>,
    pub size_bytes: Option<u64>,
    pub developer: Option<String>,
    pub year: Option<u16>,
    pub genre: Option<String>,
    pub tags: Vec<TagWeight>,
    pub location: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagWeight {
    #[serde(default)]
    pub id: u32,
    pub name: String,
    pub weight: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub title: String,
    pub source: GameSource,
    pub cover: Option<ImageSource>,
    pub hero: Option<ImageSource>,
    pub capsule: Option<ImageSource>,
    pub accent: Option<String>,
    pub hidden: bool,
    pub installed: bool,
    pub achievements: Option<AchievementSummary>,
    pub details: GameDetails,
    #[serde(skip)]
    pub launch: LaunchTarget,
    #[serde(skip)]
    pub install_command: Option<Vec<String>>,
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
            capsule: None,
            accent: None,
            hidden: false,
            installed: true,
            achievements: None,
            details: GameDetails::default(),
            launch,
            install_command: None,
            icon_source: None,
            achievement_source: None,
        }
    }

    pub fn also_running_when(mut self, marker: ProcessMarker) -> Self {
        self.process_marker = ProcessMarker::AnyOf(vec![self.process_marker, marker]);
        self
    }

    pub fn wine_prefix(&self) -> Option<PathBuf> {
        let LaunchTarget::Command { arguments, .. } = &self.launch else {
            return None;
        };
        arguments
            .iter()
            .find_map(|argument| argument.strip_prefix("WINEPREFIX="))
            .map(PathBuf::from)
    }

    pub fn playtime_tracked_by_konsol(&self) -> bool {
        matches!(
            self.source,
            GameSource::Desktop | GameSource::Manual | GameSource::Rockstar
        )
    }

    pub fn needs_store_artwork(&self) -> bool {
        matches!(
            self.source,
            GameSource::Desktop | GameSource::Manual | GameSource::Rockstar
        )
    }
}
