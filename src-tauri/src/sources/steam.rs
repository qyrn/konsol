use crate::game::{AchievementSource, Game, GameSource, ImageSource, LaunchTarget, ProcessMarker};
use crate::vdf::{self, VdfValue};
use std::fs;
use std::path::{Path, PathBuf};

const FULLY_INSTALLED_FLAG: u32 = 4;
const ICON_SIZES: [&str; 6] = ["256x256", "128x128", "96x96", "64x64", "48x48", "32x32"];
const TOOL_NAME_PREFIXES: [&str; 3] = [
    "Proton",
    "Steam Linux Runtime",
    "Steamworks Common Redistributables",
];

pub fn scan(home: &Path) -> Vec<Game> {
    let steam_root = home.join(".local/share/Steam");
    library_paths(&steam_root)
        .iter()
        .flat_map(|library| manifests_in(&library.join("steamapps")))
        .filter_map(|manifest| game_from_manifest(&manifest, &steam_root, home))
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

fn game_from_manifest(manifest: &Path, steam_root: &Path, home: &Path) -> Option<Game> {
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
    Some(Game {
        id: format!("steam:{app_id}"),
        title: name.to_string(),
        source: GameSource::Steam,
        hero: find_art(&cache, app_id, "library_hero.jpg"),
        cover,
        accent: None,
        hidden: false,
        achievements: None,
        launch: LaunchTarget::SteamApp(app_id.to_string()),
        process_marker: ProcessMarker::EnvVar {
            name: "SteamAppId",
            value: app_id.to_string(),
        },
        icon_source,
        achievement_source: Some(AchievementSource::Steam {
            app_id: app_id.to_string(),
        }),
    })
}

fn is_steam_tool(name: &str) -> bool {
    TOOL_NAME_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
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
    fn filters_out_steam_tools() {
        assert!(is_steam_tool("Proton Experimental"));
        assert!(is_steam_tool("Steam Linux Runtime 3.0 (sniper)"));
        assert!(is_steam_tool("Steamworks Common Redistributables"));
        assert!(!is_steam_tool("The Binding of Isaac: Rebirth"));
    }
}
