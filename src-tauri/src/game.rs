use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameSource {
    Steam,
    Gog,
    Epic,
    Amazon,
    Script,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum Cover {
    File(String),
    Url(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchTarget {
    SteamApp(String),
    Heroic {
        runner: &'static str,
        app_name: String,
    },
    Program(PathBuf),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub title: String,
    pub source: GameSource,
    pub cover: Option<Cover>,
    pub hidden: bool,
    #[serde(skip)]
    pub launch: LaunchTarget,
}
