use super::{Achievement, AchievementList, SchemaEntry};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct UnlockState {
    unlocked: bool,
    time: Option<i64>,
}

pub fn read(path: &Path, schema: Option<&[SchemaEntry]>) -> Option<AchievementList> {
    let states = parse_stats(&fs::read_to_string(path).ok()?);
    let achievements: Vec<Achievement> = match schema.filter(|entries| !entries.is_empty()) {
        Some(entries) => entries
            .iter()
            .map(|entry| {
                let state = states.get(&entry.api_name);
                achievement(
                    &entry.api_name,
                    entry.name.clone(),
                    entry.description.clone(),
                    entry.icon_url.clone(),
                    state,
                )
            })
            .collect(),
        None => states
            .iter()
            .map(|(key, state)| {
                achievement(key, readable_name(key), String::new(), None, Some(state))
            })
            .collect(),
    };
    let total_known = schema.is_some_and(|entries| !entries.is_empty());
    (!achievements.is_empty()).then_some(AchievementList {
        achievements,
        total_known,
    })
}

fn achievement(
    id: &str,
    name: String,
    description: String,
    icon_url: Option<String>,
    state: Option<&UnlockState>,
) -> Achievement {
    let unlocked = state.is_some_and(|state| state.unlocked);
    Achievement {
        id: id.to_string(),
        name,
        description,
        icon_url,
        unlocked,
        unlocked_at: state.and_then(|state| state.time).filter(|_| unlocked),
    }
}

fn parse_stats(content: &str) -> HashMap<String, UnlockState> {
    let mut in_achievements = false;
    let mut states = HashMap::new();
    for line in content.lines().map(str::trim) {
        if line.starts_with('[') {
            in_achievements = line == "[ACHIEVEMENTS]";
            continue;
        }
        if !in_achievements {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().trim_matches('"').to_string();
        let fields = value.trim().trim_start_matches('{').trim_end_matches('}');
        let field = |name: &str| {
            fields
                .split(',')
                .filter_map(|pair| pair.split_once('='))
                .find(|(field_name, _)| field_name.trim() == name)
                .map(|(_, field_value)| field_value.trim().trim_matches('"').to_string())
        };
        states.insert(
            key,
            UnlockState {
                unlocked: field("unlocked").as_deref() == Some("true"),
                time: field("time").and_then(|time| time.parse().ok()),
            },
        );
    }
    states
}

fn readable_name(key: &str) -> String {
    let words: Vec<&str> = key
        .split(['_', '-'])
        .filter(|word| !word.is_empty())
        .collect();
    let sentence = words.join(" ");
    let mut characters = sentence.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().chain(characters).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# comment\n[STATS]\n\"walking_street_stat\" = 6558\n\n[ACHIEVEMENTS]\n\"open_warehouse_quest\" = {unlocked = true, time = 1795086828, unlocked_at_iso = \"2026-11-19T11:13:48Z\"}\n\"locked_one\" = {unlocked = false, time = 0}\n";

    #[test]
    fn reads_unlocks_from_the_emulator_ini() {
        let states = parse_stats(SAMPLE);
        assert_eq!(states.len(), 2);
        assert!(states["open_warehouse_quest"].unlocked);
        assert_eq!(states["open_warehouse_quest"].time, Some(1_795_086_828));
        assert!(!states["locked_one"].unlocked);
    }

    #[test]
    fn uses_the_schema_for_names_and_totals_when_available() {
        let path = std::env::temp_dir().join(format!("konsol-stats-{}.ini", std::process::id()));
        fs::write(&path, SAMPLE).unwrap();
        let schema = [
            SchemaEntry {
                api_name: "open_warehouse_quest".into(),
                name: "Ouverture".into(),
                description: "Ouvre l'entrepôt".into(),
                icon_url: None,
            },
            SchemaEntry {
                api_name: "never_seen".into(),
                name: "Secret".into(),
                description: String::new(),
                icon_url: None,
            },
        ];
        let with_schema = read(&path, Some(&schema)).unwrap();
        assert!(with_schema.total_known);
        assert_eq!(with_schema.summary().unlocked, 1);
        assert_eq!(with_schema.summary().total, Some(2));
        let without_schema = read(&path, None).unwrap();
        assert!(!without_schema.total_known);
        assert!(
            without_schema
                .achievements
                .iter()
                .any(|item| item.name == "Open warehouse quest")
        );
        fs::remove_file(path).unwrap();
    }
}
