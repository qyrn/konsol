use super::{Achievement, AchievementList};
use crate::binary_vdf;
use crate::vdf::VdfValue;
use std::fs;
use std::path::{Path, PathBuf};

const ICON_BASE_URL: &str =
    "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps";

fn stats_directory(home: &Path) -> PathBuf {
    home.join(".local/share/Steam/appcache/stats")
}

pub fn user_stats_files(home: &Path, app_id: &str) -> Vec<PathBuf> {
    let suffix = format!("_{app_id}.bin");
    let Ok(entries) = fs::read_dir(stats_directory(home)) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("UserGameStats_") && name.ends_with(&suffix))
        })
        .collect()
}

fn read_vdf(path: &Path) -> Option<VdfValue> {
    binary_vdf::parse(&fs::read(path).ok()?)
}

pub fn read(home: &Path, app_id: &str) -> Option<AchievementList> {
    let schema =
        read_vdf(&stats_directory(home).join(format!("UserGameStatsSchema_{app_id}.bin")))?;
    let stats = schema.get(app_id)?.get("stats")?;
    let user_stats = user_stats_files(home, app_id)
        .into_iter()
        .max_by_key(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .ok()
        })
        .and_then(|path| read_vdf(&path));
    let cache = user_stats.as_ref().and_then(|stats| stats.get("cache"));
    let mut achievements = Vec::new();
    for (stat_id, stat) in stats.entries() {
        let Some(bits) = stat.get("bits") else {
            continue;
        };
        let user_stat = cache.and_then(|cache| cache.get(stat_id));
        let unlocked_bits = user_stat
            .and_then(|stat| stat.get_text("data"))
            .and_then(|data| data.parse::<i32>().ok())
            .map_or(0, |data| data as u32);
        for (bit, definition) in bits.entries() {
            let Ok(bit_index) = bit.parse::<u32>() else {
                continue;
            };
            let unlocked = bit_index < 32 && unlocked_bits & (1 << bit_index) != 0;
            let display = definition.get("display");
            let localized = |key: &str| {
                display
                    .and_then(|display| display.get(key))
                    .and_then(localized_text)
            };
            achievements.push(Achievement {
                id: definition
                    .get_text("name")
                    .map_or_else(|| format!("{stat_id}_{bit}"), str::to_string),
                name: localized("name").unwrap_or_default(),
                description: localized("desc").unwrap_or_default(),
                icon_url: display
                    .and_then(|display| {
                        display.get_text(if unlocked { "icon" } else { "icon_gray" })
                    })
                    .map(|icon| format!("{ICON_BASE_URL}/{app_id}/{icon}")),
                unlocked,
                unlocked_at: user_stat
                    .and_then(|stat| stat.get("AchievementTimes"))
                    .and_then(|times| times.get_text(bit))
                    .and_then(|time| time.parse().ok())
                    .filter(|_| unlocked),
            });
        }
    }
    (!achievements.is_empty()).then_some(AchievementList {
        achievements,
        total_known: true,
    })
}

fn localized_text(value: &VdfValue) -> Option<String> {
    match value {
        VdfValue::Text(text) => Some(text.clone()),
        VdfValue::Section(_) => value
            .get_text("french")
            .or_else(|| value.get_text("english"))
            .map(str::to_string),
    }
}
