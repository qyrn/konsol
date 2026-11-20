use crate::game::{Cover, Game, GameSource, LaunchTarget};
use crate::vdf::{self, VdfValue};
use std::fs;
use std::path::{Path, PathBuf};

const FULLY_INSTALLED_FLAG: u32 = 4;
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
        .filter_map(|manifest| game_from_manifest(&manifest, &steam_root))
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

fn game_from_manifest(manifest: &Path, steam_root: &Path) -> Option<Game> {
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
    Some(Game {
        id: format!("steam:{app_id}"),
        title: name.to_string(),
        source: GameSource::Steam,
        cover: find_cover(steam_root, app_id),
        hidden: false,
        launch: LaunchTarget::SteamApp(app_id.to_string()),
    })
}

fn is_steam_tool(name: &str) -> bool {
    TOOL_NAME_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn find_cover(steam_root: &Path, app_id: &str) -> Option<Cover> {
    let cache = steam_root.join("appcache/librarycache");
    [
        cache.join(app_id).join("library_600x900.jpg"),
        cache.join(format!("{app_id}_library_600x900.jpg")),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .map(|path| Cover::File(path.to_string_lossy().into_owned()))
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
