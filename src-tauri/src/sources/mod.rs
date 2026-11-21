mod desktop;
mod heroic;
mod steam;

use crate::config::KonsolConfig;
use crate::game::{Game, GameSource, LaunchTarget};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn scan_all(home: &Path, config: &KonsolConfig) -> Vec<Game> {
    let mut games: Vec<Game> = steam::scan(home)
        .into_iter()
        .chain(heroic::scan(home))
        .chain(desktop::scan(home))
        .chain(manual_games(config))
        .collect();
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
        Game::launched_by_konsol(
            manual.id.clone(),
            manual.title.clone(),
            GameSource::Manual,
            LaunchTarget::Program(manual.executable.clone()),
        )
    })
}

pub fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}
