use crate::game::{
    AchievementSource, Game, GameDetails, GameSource, ImageSource, LaunchTarget, ProcessMarker,
};
use crate::time::iso_timestamp;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct StoreEntry {
    title: String,
    cover_url: Option<String>,
    hero_url: Option<String>,
    developer: Option<String>,
    genre: Option<String>,
}

struct InstalledGame {
    app_name: String,
    install_path: Option<String>,
    fallback_title: Option<String>,
    size_bytes: Option<u64>,
}

#[derive(Clone, Copy, Default)]
struct Playtime {
    minutes: u64,
    last_played: Option<i64>,
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
    let playtimes = read_playtimes(&heroic_dir);
    STORES
        .iter()
        .flat_map(|store| scan_store(&heroic_dir, store, &playtimes))
        .collect()
}

fn read_playtimes(heroic_dir: &Path) -> HashMap<String, Playtime> {
    read_json(&heroic_dir.join("store/timestamp.json"))
        .as_ref()
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(app_name, entry)| {
            let playtime = Playtime {
                minutes: entry
                    .get("totalPlayed")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                last_played: entry
                    .get("lastPlayed")
                    .and_then(Value::as_str)
                    .and_then(iso_timestamp),
            };
            (app_name.clone(), playtime)
        })
        .collect()
}

fn scan_store(
    heroic_dir: &Path,
    store: &Store,
    playtimes: &HashMap<String, Playtime>,
) -> Vec<Game> {
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
            let playtime = playtimes
                .get(&installed.app_name)
                .copied()
                .unwrap_or_default();
            let details = GameDetails {
                playtime_minutes: Some(playtime.minutes),
                last_played: playtime.last_played,
                size_bytes: installed.size_bytes,
                developer: entry.and_then(|entry| entry.developer.clone()),
                genre: entry.and_then(|entry| entry.genre.clone()),
                location: installed.install_path.clone(),
                ..GameDetails::default()
            };
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
                achievements: None,
                details,
                achievement_source: (store.source == GameSource::Gog).then(|| {
                    AchievementSource::Gog {
                        app_name: installed.app_name.clone(),
                    }
                }),
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
                developer: text_field(game, "developer"),
                genre: game
                    .get("extra")
                    .and_then(|extra| extra.get("genres"))
                    .and_then(|genres| genres.get(0))
                    .and_then(Value::as_str)
                    .map(str::to_string),
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
                size_bytes: game.get("install_size").and_then(size_in_bytes),
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
            size_bytes: game.get("install_size").and_then(size_in_bytes),
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
                size_bytes: None,
            })
        })
        .collect()
}

fn size_in_bytes(value: &Value) -> Option<u64> {
    if let Some(bytes) = value.as_u64() {
        return Some(bytes);
    }
    let (amount, unit) = value.as_str()?.trim().split_once(' ')?;
    let amount: f64 = amount.parse().ok()?;
    let multiplier = match unit {
        "B" => 1.0,
        "KiB" | "KB" => 1024.0,
        "MiB" | "MB" => 1024.0 * 1024.0,
        "GiB" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "TiB" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    let bytes = amount * multiplier;
    (bytes.is_finite() && bytes >= 0.0).then_some(bytes as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_install_sizes_in_both_heroic_formats() {
        assert_eq!(
            size_in_bytes(&serde_json::json!("82.22 GiB")),
            Some(88_283_052_769)
        );
        assert_eq!(size_in_bytes(&serde_json::json!(1234)), Some(1234));
        assert_eq!(size_in_bytes(&serde_json::json!("lots")), None);
    }

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
