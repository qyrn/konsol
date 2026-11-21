mod emulator;
mod gog;
mod steam;

use crate::game::{AchievementSource, AchievementSummary};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub unlocked: bool,
    pub unlocked_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaEntry {
    pub api_name: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
}

pub struct AchievementList {
    pub achievements: Vec<Achievement>,
    pub total_known: bool,
}

impl AchievementList {
    pub fn summary(&self) -> AchievementSummary {
        AchievementSummary {
            unlocked: self
                .achievements
                .iter()
                .filter(|item| item.unlocked)
                .count(),
            total: self.total_known.then_some(self.achievements.len()),
        }
    }
}

pub fn read(
    source: &AchievementSource,
    home: &Path,
    emulator_schema: Option<&[SchemaEntry]>,
) -> Option<AchievementList> {
    let mut list = match source {
        AchievementSource::Steam { app_id } => steam::read(home, app_id)?,
        AchievementSource::Gog { app_name } => gog::read(home, app_name)?,
        AchievementSource::EmulatorStats { path } => emulator::read(path, emulator_schema)?,
    };
    list.achievements.sort_by(|first, second| {
        second
            .unlocked
            .cmp(&first.unlocked)
            .then(second.unlocked_at.cmp(&first.unlocked_at))
    });
    Some(list)
}

pub fn watched_files(source: &AchievementSource, home: &Path) -> Vec<PathBuf> {
    match source {
        AchievementSource::Steam { app_id } => steam::user_stats_files(home, app_id),
        AchievementSource::Gog { .. } => vec![gog::cache_file(home)],
        AchievementSource::EmulatorStats { path } => vec![path.clone()],
    }
}
