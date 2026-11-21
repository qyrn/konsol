use crate::artwork;
use crate::config::{ConfigStore, KonsolConfig, ManualGame};
use crate::desktop_entry::DesktopPaths;
use crate::game::Game;
use crate::launcher;
use crate::sources::{self, is_executable_file};
use crate::window_watch;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tauri::State;

const MAX_TITLE_LENGTH: usize = 120;

pub struct AppState {
    pub home: PathBuf,
    pub config: Mutex<ConfigStore>,
    pub games: Mutex<HashMap<String, Game>>,
    pub accents: Mutex<HashMap<PathBuf, Option<String>>>,
    pub applications_dir: PathBuf,
    pub icons_dir: PathBuf,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn update_config(
    state: &AppState,
    change: impl FnOnce(&mut KonsolConfig) -> Result<(), String>,
) -> Result<(), String> {
    let store = lock(&state.config);
    let mut config = store.load()?;
    change(&mut config)?;
    store.save(&config)
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    let config = lock(&state.config).load()?;
    let mut games = sources::scan_all(&state.home, &config);
    fill_accents(&mut games, &state.accents);
    *lock(&state.games) = games
        .iter()
        .map(|game| (game.id.clone(), game.clone()))
        .collect();
    Ok(games)
}

fn fill_accents(games: &mut [Game], cache: &Mutex<HashMap<PathBuf, Option<String>>>) {
    let mut cache = lock(cache);
    for game in games {
        let from_cover = game
            .cover
            .as_ref()
            .and_then(|cover| cover.local_path())
            .and_then(|path| {
                cache
                    .entry(path)
                    .or_insert_with_key(|path| artwork::dominant_accent(path))
                    .clone()
            });
        game.accent = Some(from_cover.unwrap_or_else(|| artwork::accent_from_title(&game.title)));
    }
}

#[tauri::command]
pub fn launch_game(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let game = lock(&state.games)
        .get(&id)
        .cloned()
        .ok_or("Ce jeu n'est plus dans la liste, actualise la bibliothèque.")?;
    launcher::launch(&game)?;
    window_watch::watch_for_game_window(
        game,
        DesktopPaths {
            applications: state.applications_dir.clone(),
            icons: state.icons_dir.clone(),
        },
    );
    Ok(())
}

#[tauri::command]
pub fn set_game_hidden(id: String, hidden: bool, state: State<'_, AppState>) -> Result<(), String> {
    update_config(&state, |config| {
        config.hidden_game_ids.retain(|hidden_id| hidden_id != &id);
        if hidden {
            config.hidden_game_ids.push(id);
        }
        Ok(())
    })
}

#[tauri::command]
pub fn add_manual_game(
    title: String,
    executable: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let title = validate_title(&title)?;
    let executable = validate_executable(executable.trim())?;
    let id = format!("manual:{}", executable.display());
    update_config(&state, |config| {
        if config.manual_games.iter().any(|game| game.id == id) {
            return Err("Ce jeu est déjà dans la liste.".to_string());
        }
        config.manual_games.push(ManualGame {
            id,
            title,
            executable,
        });
        Ok(())
    })
}

#[tauri::command]
pub fn remove_manual_game(id: String, state: State<'_, AppState>) -> Result<(), String> {
    update_config(&state, |config| {
        config.manual_games.retain(|game| game.id != id);
        config.hidden_game_ids.retain(|hidden_id| hidden_id != &id);
        Ok(())
    })
}

fn validate_title(title: &str) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Donne un nom au jeu.".to_string());
    }
    if title.chars().count() > MAX_TITLE_LENGTH {
        return Err(format!(
            "Le nom fait plus de {MAX_TITLE_LENGTH} caractères."
        ));
    }
    Ok(title.to_string())
}

fn validate_executable(executable: &str) -> Result<PathBuf, String> {
    let path = Path::new(executable);
    if !path.is_absolute() {
        return Err("Mets le chemin complet, en commençant par /.".to_string());
    }
    if !path.is_file() {
        return Err("Y'a pas de fichier à ce chemin.".to_string());
    }
    if !is_executable_file(path) {
        return Err("Ce fichier n'est pas exécutable (chmod +x).".to_string());
    }
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_or_overlong_titles() {
        assert!(validate_title("   ").is_err());
        assert!(validate_title(&"a".repeat(MAX_TITLE_LENGTH + 1)).is_err());
        assert_eq!(validate_title("  Skate 3  "), Ok("Skate 3".to_string()));
    }

    #[test]
    fn rejects_relative_missing_or_non_executable_paths() {
        assert!(validate_executable("bin/sh").is_err());
        assert!(validate_executable("/does/not/exist").is_err());
        assert!(validate_executable("/etc/hostname").is_err());
        assert!(validate_executable("/bin/sh").is_ok());
    }
}
