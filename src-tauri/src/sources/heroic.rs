use crate::game::{Game, GameSource, ImageSource, LaunchTarget, ProcessMarker};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct StoreEntry {
    title: String,
    cover_url: Option<String>,
    hero_url: Option<String>,
}

struct InstalledGame {
    app_name: String,
    install_path: Option<String>,
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
                .or(installed.fallback_title.clone())?;
            let cover = entry
                .and_then(|entry| entry.cover_url.as_deref())
                .map(|url| cached_image(heroic_dir, url));
            let hero = entry
                .and_then(|entry| entry.hero_url.as_deref())
                .map(|url| cached_image(heroic_dir, url));
            let process_marker = wine_prefix(heroic_dir, &installed.app_name)
                .or(installed.install_path)
                .map(ProcessMarker::Mentions)?;
            Some(Game {
                id: format!("{}:{}", store.runner, installed.app_name),
                title,
                source: store.source,
                icon_source: cover.as_ref().and_then(ImageSource::local_path),
                cover,
                hero,
                accent: None,
                hidden: false,
                launch: LaunchTarget::Heroic {
                    runner: store.runner,
                    app_name: installed.app_name,
                },
                process_marker,
            })
        })
        .collect()
}

fn cached_image(heroic_dir: &Path, url: &str) -> ImageSource {
    let file_name: String = Sha256::digest(url.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let cached = heroic_dir.join("images-cache").join(file_name);
    if cached.is_file() {
        ImageSource::File(cached.to_string_lossy().into_owned())
    } else {
        ImageSource::Url(url.to_string())
    }
}

fn wine_prefix(heroic_dir: &Path, app_name: &str) -> Option<String> {
    let config = read_json(
        &heroic_dir
            .join("GamesConfig")
            .join(format!("{app_name}.json")),
    )?;
    text_field(config.get(app_name)?, "winePrefix")
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

fn https_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| text_field(value, key))
        .find(|url| url.starts_with("https://"))
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
            let entry = StoreEntry {
                title: text_field(game, "title")?,
                cover_url: https_field(game, &["art_square", "art_cover"]),
                hero_url: https_field(game, &["art_background", "art_cover"]),
            };
            Some((app_name, entry))
        })
        .collect()
}

fn folder_name(path: Option<&str>) -> Option<String> {
    path.map(Path::new)
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
            let install_path = text_field(game, "install_path");
            Some(InstalledGame {
                app_name: text_field(game, "appName")?,
                fallback_title: folder_name(install_path.as_deref()),
                install_path,
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
            install_path: text_field(game, "install_path"),
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
            let install_path = text_field(game, "path");
            Some(InstalledGame {
                app_name: text_field(game, "id")?,
                fallback_title: folder_name(install_path.as_deref()),
                install_path,
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

    #[test]
    fn finds_heroic_cached_images_by_url_hash() {
        let directory = std::env::temp_dir().join(format!("konsol-heroic-{}", std::process::id()));
        let cache = directory.join("images-cache");
        fs::create_dir_all(&cache).unwrap();
        let hash = "8b37ef624d6f8a4092c43daed92660cde499434f8c0a2078f342bc34f4e752f2";
        let url = format!("https://images.gog.com/{hash}.jpg?namespace=gamesdb");
        let cached_name = "18556e6dcf60097852399c5336f527b638a76439e5213609c6e7ed1a8cc224ef";
        fs::write(cache.join(cached_name), b"x").unwrap();
        assert_eq!(
            cached_image(&directory, &url),
            ImageSource::File(cache.join(cached_name).to_string_lossy().into_owned())
        );
        assert_eq!(
            cached_image(&directory, "https://example.test/missing.jpg"),
            ImageSource::Url("https://example.test/missing.jpg".into())
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn prefers_square_art_for_covers_and_background_art_for_heroes() {
        let game = serde_json::json!({
            "art_cover": "https://example.test/cover.jpg",
            "art_square": "https://example.test/square.jpg",
            "art_background": "http://example.test/insecure.jpg"
        });
        assert_eq!(
            https_field(&game, &["art_square", "art_cover"]).as_deref(),
            Some("https://example.test/square.jpg")
        );
        assert_eq!(
            https_field(&game, &["art_background", "art_cover"]).as_deref(),
            Some("https://example.test/cover.jpg")
        );
    }
}
