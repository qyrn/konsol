mod desktop;
mod folders;
mod heroic;
mod rockstar;
mod steam;

pub use folders::default_folders;
pub use steam::logged_in_steam_id;

use crate::config::KonsolConfig;
use crate::game::{Game, GameSource, ImageSource, LaunchTarget, ProcessMarker};
use crate::metadata::OwnedApp;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn watched_paths(home: &Path, config: &KonsolConfig) -> Vec<PathBuf> {
    let heroic = home.join(".config/heroic");
    let mut paths = vec![
        home.join(".local/share/applications"),
        heroic.join("gog_store/installed.json"),
        heroic.join("legendaryConfig/legendary/installed.json"),
        heroic.join("nile_config/nile/installed.json"),
    ];
    paths.extend(steam::watched_paths(home));
    let folders = game_folders(home, config);
    paths.extend(folders::rockstar_watched_paths(&folders));
    for folder in folders {
        paths.extend(nested_directories(&folder, 3));
        paths.push(folder);
    }
    paths
}

fn nested_directories(folder: &Path, depth: usize) -> Vec<PathBuf> {
    if depth == 0 {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .flat_map(|entry| {
            let path = entry.path();
            let mut found = nested_directories(&path, depth - 1);
            found.push(path);
            found
        })
        .collect()
}

pub fn game_folders(home: &Path, config: &KonsolConfig) -> Vec<PathBuf> {
    config
        .game_folders
        .clone()
        .unwrap_or_else(|| default_folders(home))
}

pub fn scan_all(home: &Path, config: &KonsolConfig, owned_steam: &[OwnedApp]) -> Vec<Game> {
    let mut games: Vec<Game> = steam::scan(home)
        .into_iter()
        .chain(heroic::scan(home))
        .chain(desktop::scan(home))
        .chain(manual_games(config))
        .collect();
    let known_targets: HashSet<PathBuf> = games
        .iter()
        .filter_map(|game| game.details.location.as_deref())
        .map(|location| fs::canonicalize(location).unwrap_or_else(|_| PathBuf::from(location)))
        .collect();
    games.extend(folders::scan(&game_folders(home, config), &known_targets));
    let installed_ids: HashSet<String> = games.iter().map(|game| game.id.clone()).collect();
    games.extend(steam::owned_not_installed(owned_steam, &installed_ids));
    for game in &mut games {
        game.hidden = config.hidden_game_ids.contains(&game.id);
        if let Some(title) = config.custom_titles.get(&game.id) {
            game.title = title.clone();
        }
    }
    games.sort_by_cached_key(|game| game.title.to_lowercase());
    games
}

fn manual_games(config: &KonsolConfig) -> impl Iterator<Item = Game> + '_ {
    config.manual_games.iter().map(|manual| {
        let launch = if manual.arguments.is_empty() {
            LaunchTarget::Program(manual.executable.clone())
        } else {
            LaunchTarget::Command {
                arguments: std::iter::once(manual.executable.to_string_lossy().into_owned())
                    .chain(manual.arguments.iter().cloned())
                    .collect(),
                working_directory: manual.executable.parent().map(Path::to_path_buf),
            }
        };
        let mut game = Game::launched_by_konsol(
            manual.id.clone(),
            manual.title.clone(),
            GameSource::Manual,
            launch,
        );
        let location = manual.executable.to_string_lossy().into_owned();
        game.details.location = Some(location.clone());
        let mut game = game.also_running_when(ProcessMarker::Mentions(location));
        game.accent = manual.accent.clone();
        game.cover = manual
            .cover
            .as_ref()
            .filter(|path| path.is_file())
            .map(|path| ImageSource::File(path.to_string_lossy().into_owned()));
        game.icon_source = manual.cover.clone().filter(|path| path.is_file());
        game
    })
}

pub fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}
