use crate::game::{Cover, Game, GameSource, LaunchTarget};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct StoreEntry {
    title: String,
    cover_url: Option<String>,
}

struct InstalledGame {
    app_name: String,
    fallback_title: Option<String>,
}

struct Store {
    source: GameSource,
    runner: &'static str,
    library_file: &'static str,
    installed: fn(&Path) -> Vec<InstalledGame>,
}

const STORES: [Store; 3] = [
    Store {
        source: GameSource::Gog,
        runner: "gog",
        library_file: "store_cache/gog_library.json",
        installed: installed_gog,
    },
    Store {
        source: GameSource::Epic,
        runner: "legendary",
        library_file: "store_cache/legendary_library.json",
        installed: installed_epic,
    },
    Store {
        source: GameSource::Amazon,
        runner: "nile",
        library_file: "store_cache/nile_library.json",
        installed: installed_amazon,
    },
];

pub fn scan(home: &Path) -> Vec<Game> {
    let heroic_dir = home.join(".config/heroic");
    STORES
        .iter()
        .flat_map(|store| scan_store(&heroic_dir, store))
        .collect()
}

fn scan_store(heroic_dir: &Path, store: &Store) -> Vec<Game> {
    let library = read_library(&heroic_dir.join(store.library_file));
    (store.installed)(heroic_dir)
        .into_iter()
        .filter(|installed| is_safe_app_name(&installed.app_name))
        .filter_map(|installed| {
            let entry = library.get(&installed.app_name);
            let title = entry
                .map(|entry| entry.title.clone())
                .or(installed.fallback_title)?;
            Some(Game {
                id: format!("{}:{}", store.runner, installed.app_name),
                title,
                source: store.source,
                cover: entry
                    .and_then(|entry| entry.cover_url.clone())
                    .map(Cover::Url),
                hidden: false,
                launch: LaunchTarget::Heroic {
                    runner: store.runner,
                    app_name: installed.app_name,
                },
            })
        })
        .collect()
}

fn is_safe_app_name(app_name: &str) -> bool {
    !app_name.is_empty()
        && app_name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn read_json(path: &Path) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn text_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn read_library(path: &Path) -> HashMap<String, StoreEntry> {
    let Some(library) = read_json(path) else {
        return HashMap::new();
    };
    library
        .get("games")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|game| {
            let app_name = text_field(game, "app_name")?;
            let title = text_field(game, "title")?;
            let cover_url = text_field(game, "art_cover")
                .or_else(|| text_field(game, "art_square"))
                .filter(|url| url.starts_with("https://"));
            Some((app_name, StoreEntry { title, cover_url }))
        })
        .collect()
}

fn folder_name(path: Option<String>) -> Option<String> {
    path.as_deref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

fn installed_gog(heroic_dir: &Path) -> Vec<InstalledGame> {
    let Some(installed) = read_json(&heroic_dir.join("gog_store/installed.json")) else {
        return Vec::new();
    };
    installed
        .get("installed")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|game| !game.get("is_dlc").and_then(Value::as_bool).unwrap_or(false))
        .filter_map(|game| {
            Some(InstalledGame {
                app_name: text_field(game, "appName")?,
                fallback_title: folder_name(text_field(game, "install_path")),
            })
        })
        .collect()
}

fn installed_epic(heroic_dir: &Path) -> Vec<InstalledGame> {
    let Some(installed) = read_json(&heroic_dir.join("legendaryConfig/legendary/installed.json"))
    else {
        return Vec::new();
    };
    installed
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(_, game)| !game.get("is_dlc").and_then(Value::as_bool).unwrap_or(false))
        .map(|(app_name, game)| InstalledGame {
            app_name: app_name.clone(),
            fallback_title: text_field(game, "title"),
        })
        .collect()
}

fn installed_amazon(heroic_dir: &Path) -> Vec<InstalledGame> {
    let Some(installed) = read_json(&heroic_dir.join("nile_config/nile/installed.json")) else {
        return Vec::new();
    };
    installed
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|game| {
            Some(InstalledGame {
                app_name: text_field(game, "id")?,
                fallback_title: folder_name(text_field(game, "path")),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_app_names_that_could_alter_the_launch_url() {
        assert!(is_safe_app_name("1423049311"));
        assert!(is_safe_app_name("Fortnite_x-1"));
        assert!(!is_safe_app_name("x&runner=evil"));
        assert!(!is_safe_app_name(""));
    }
}
