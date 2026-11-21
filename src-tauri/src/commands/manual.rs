use crate::config::ManualGame;
use crate::sources::is_executable_file;
use crate::state::{AppState, update_config};
use std::path::{Path, PathBuf};
use tauri::State;

const MAX_TITLE_LENGTH: usize = 120;

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
