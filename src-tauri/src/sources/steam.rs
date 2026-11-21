use crate::game::{
    AchievementSource, Game, GameDetails, GameSource, ImageSource, LaunchTarget, ProcessMarker,
};
use crate::metadata::OwnedApp;
use crate::vdf::{self, VdfValue};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const FULLY_INSTALLED_FLAG: u32 = 4;
const STORE_ASSETS_URL: &str =
    "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps";
const ICON_SIZES: [&str; 6] = ["256x256", "128x128", "96x96", "64x64", "48x48", "32x32"];
const TOOL_NAME_PREFIXES: [&str; 3] = [
    "Proton",
    "Steam Linux Runtime",
    "Steamworks Common Redistributables",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Playtime {
    minutes: u64,
    last_played: i64,
}

pub fn scan(home: &Path) -> Vec<Game> {
    let steam_root = home.join(".local/share/Steam");
    let playtimes = read_playtimes(&steam_root);
    library_paths(&steam_root)
        .iter()
        .flat_map(|library| manifests_in(&library.join("steamapps")))
        .filter_map(|manifest| game_from_manifest(&manifest, &steam_root, home, &playtimes))
        .collect()
}

pub fn watched_paths(home: &Path) -> Vec<PathBuf> {
    library_paths(&home.join(".local/share/Steam"))
        .into_iter()
        .map(|library| library.join("steamapps"))
        .collect()
}

pub fn logged_in_steam_id(home: &Path) -> Option<String> {
    let content = fs::read_to_string(home.join(".local/share/Steam/config/loginusers.vdf")).ok()?;
    let root = vdf::parse(&content);
    let users = root.get("users")?.entries();
    let is_steam_id = |id: &&String| id.len() == 17 && id.chars().all(|c| c.is_ascii_digit());
    users
        .iter()
        .find(|(id, user)| is_steam_id(&id) && user.get_text("MostRecent") == Some("1"))
        .or_else(|| users.iter().find(|(id, _)| is_steam_id(&id)))
        .map(|(id, _)| id.clone())
}

pub fn owned_not_installed(owned: &[OwnedApp], installed_ids: &HashSet<String>) -> Vec<Game> {
    owned
        .iter()
        .filter(|app| !installed_ids.contains(&format!("steam:{}", app.app_id)))
        .filter(|app| !is_steam_tool(&app.name))
        .map(|app| {
            let app_id = app.app_id.to_string();
            let asset =
                |file: &str| ImageSource::Url(format!("{STORE_ASSETS_URL}/{app_id}/{file}"));
            Game {
                id: format!("steam:{app_id}"),
                title: app.name.clone(),
                source: GameSource::Steam,
                cover: Some(asset("library_600x900.jpg")),
                hero: Some(asset("library_hero.jpg")),
                capsule: None,
                accent: None,
                hidden: false,
                installed: false,
                achievements: None,
                details: GameDetails {
                    playtime_minutes: Some(app.playtime_minutes),
                    last_played: (app.last_played > 0).then_some(app.last_played),
                    ..GameDetails::default()
                },
                launch: LaunchTarget::SteamApp(app_id.clone()),
                process_marker: ProcessMarker::EnvVar {
                    name: "SteamAppId",
                    value: app_id,
                },
                install_command: None,
                icon_source: None,
                achievement_source: None,
            }
        })
        .collect()
}

fn read_playtimes(steam_root: &Path) -> HashMap<String, Playtime> {
    let mut playtimes: HashMap<String, Playtime> = HashMap::new();
    let Ok(users) = fs::read_dir(steam_root.join("userdata")) else {
        return playtimes;
    };
    for user in users.filter_map(Result::ok) {
        let Ok(content) = fs::read_to_string(user.path().join("config/localconfig.vdf")) else {
            continue;
        };
        for (app_id, playtime) in playtimes_in(&vdf::parse(&content)) {
            let known = playtimes.entry(app_id).or_default();
            known.minutes = known.minutes.max(playtime.minutes);
            known.last_played = known.last_played.max(playtime.last_played);
        }
    }
    playtimes
}

fn playtimes_in(local_config: &VdfValue) -> Vec<(String, Playtime)> {
    let apps = ["UserLocalConfigStore", "Software", "Valve", "Steam", "apps"]
        .iter()
        .try_fold(local_config, |value, key| value.get(key));
    apps.map(VdfValue::entries)
        .unwrap_or(&[])
        .iter()
        .map(|(app_id, app)| {
            let number = |key: &str| app.get_text(key).and_then(|text| text.parse().ok());
            let playtime = Playtime {
                minutes: number("Playtime").map_or(0, |minutes: i64| minutes.max(0).unsigned_abs()),
                last_played: number("LastPlayed").unwrap_or(0),
            };
            (app_id.clone(), playtime)
        })
        .collect()
}

fn library_paths(steam_root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![steam_root.to_path_buf()];
    let Ok(content) = fs::read_to_string(steam_root.join("steamapps/libraryfolders.vdf")) else {
        return paths;
    };
    let root = vdf::parse(&content);
    let folders = root
        .get("libraryfolders")
        .map(VdfValue::entries)
        .unwrap_or(&[]);
    for (_, folder) in folders {
        if let Some(path) = folder.get_text("path").map(PathBuf::from)
            && !paths.iter().any(|known| same_directory(known, &path))
        {
            paths.push(path);
        }
    }
    paths
}

fn same_directory(first: &Path, second: &Path) -> bool {
    match (fs::canonicalize(first), fs::canonicalize(second)) {
        (Ok(first), Ok(second)) => first == second,
        _ => first == second,
    }
}

fn manifests_in(steamapps: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(steamapps) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"))
        })
        .collect()
}

fn game_from_manifest(
    manifest: &Path,
    steam_root: &Path,
    home: &Path,
    playtimes: &HashMap<String, Playtime>,
) -> Option<Game> {
    let content = fs::read_to_string(manifest).ok()?;
    let root = vdf::parse(&content);
    let app_state = root.get("AppState")?;
    let app_id = app_state.get_text("appid")?;
    let name = app_state.get_text("name")?;
    let state_flags: u32 = app_state.get_text("StateFlags")?.parse().ok()?;
    if !app_id.chars().all(|character| character.is_ascii_digit())
        || state_flags & FULLY_INSTALLED_FLAG == 0
        || is_steam_tool(name)
    {
        return None;
    }
    let cache = steam_root.join("appcache/librarycache");
    let cover = find_art(&cache, app_id, "library_600x900.jpg");
    let icon_source =
        find_icon(home, app_id).or_else(|| cover.as_ref().and_then(ImageSource::local_path));
    let playtime = playtimes.get(app_id).copied().unwrap_or_default();
    let last_played = app_state
        .get_text("LastPlayed")
        .and_then(|text| text.parse::<i64>().ok())
        .unwrap_or(0)
        .max(playtime.last_played);
    let details = GameDetails {
        playtime_minutes: Some(playtime.minutes),
        last_played: (last_played > 0).then_some(last_played),
        size_bytes: app_state
            .get_text("SizeOnDisk")
            .and_then(|text| text.parse().ok())
            .filter(|size| *size > 0),
        location: app_state.get_text("installdir").and_then(|directory| {
            manifest
                .parent()
                .map(|steamapps| steamapps.join("common").join(directory))
                .map(|path| path.to_string_lossy().into_owned())
        }),
        ..GameDetails::default()
    };
    Some(Game {
        id: format!("steam:{app_id}"),
        title: name.to_string(),
        source: GameSource::Steam,
        hero: find_art(&cache, app_id, "library_hero.jpg"),
        capsule: None,
        cover,
        accent: None,
        hidden: false,
        installed: true,
        achievements: None,
        details,
        launch: LaunchTarget::SteamApp(app_id.to_string()),
        process_marker: ProcessMarker::EnvVar {
            name: "SteamAppId",
            value: app_id.to_string(),
        },
        install_command: None,
        icon_source,
        achievement_source: Some(AchievementSource::Steam {
            app_id: app_id.to_string(),
        }),
    })
}

const SIDE_BUILD_MARKERS: [&str; 9] = [
    "test server",
    "staging branch",
    "public test",
    "playtest",
    "dedicated server",
    "experimental server",
    " pts",
    " sdk",
    "benchmark",
];

fn is_steam_tool(name: &str) -> bool {
    let lowercase = name.to_lowercase();
    TOOL_NAME_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
        || SIDE_BUILD_MARKERS
            .iter()
            .any(|marker| lowercase.contains(marker))
}

fn find_art(cache: &Path, app_id: &str, file_name: &str) -> Option<ImageSource> {
    [
        cache.join(app_id).join(file_name),
        cache.join(format!("{app_id}_{file_name}")),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .map(|path| ImageSource::File(path.to_string_lossy().into_owned()))
}

fn find_icon(home: &Path, app_id: &str) -> Option<PathBuf> {
    let icons = home.join(".local/share/icons/hicolor");
    ICON_SIZES
        .iter()
        .map(|size| {
            icons
                .join(size)
                .join("apps")
                .join(format!("steam_icon_{app_id}.png"))
        })
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_playtime_and_last_session_from_local_config() {
        let config = vdf::parse(
            r#""UserLocalConfigStore" { "Software" { "Valve" { "Steam" { "apps" {
                "250900" { "LastPlayed" "1795257247" "Playtime" "12180" }
                "570" { "cloud" { } }
            } } } } }"#,
        );
        let playtimes: HashMap<_, _> = playtimes_in(&config).into_iter().collect();
        assert_eq!(
            playtimes["250900"],
            Playtime {
                minutes: 12180,
                last_played: 1795257247
            }
        );
        assert_eq!(playtimes["570"], Playtime::default());
    }

    #[test]
    fn filters_out_steam_tools() {
        assert!(is_steam_tool("Proton Experimental"));
        assert!(is_steam_tool("Steam Linux Runtime 3.0 (sniper)"));
        assert!(is_steam_tool("Steamworks Common Redistributables"));
        assert!(!is_steam_tool("The Binding of Isaac: Rebirth"));
        assert!(is_steam_tool(
            "Tom Clancy's Rainbow Six Siege - Test Server"
        ));
        assert!(is_steam_tool("Rust - Staging Branch"));
        assert!(is_steam_tool("Contraband Police Playtest"));
        assert!(!is_steam_tool("Football Manager 26"));
    }
}
