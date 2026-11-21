use crate::config::ManualGame;
use crate::desktop_entry::file_stem;
use crate::desktop_file::split_exec;
use crate::sources::is_executable_file;
use crate::state::{AppState, lock, update_config};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

const MAX_TITLE_LENGTH: usize = 120;
const MAX_ARGUMENTS_LENGTH: usize = 500;
const MAX_COVER_BYTES: u64 = 20 * 1024 * 1024;
const COVER_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewManualGame {
    title: String,
    executable: String,
    arguments: String,
    accent: Option<String>,
    cover: Option<String>,
}

#[tauri::command]
pub fn add_manual_game(game: NewManualGame, state: State<'_, AppState>) -> Result<String, String> {
    let title = validate_title(&game.title)?;
    let executable = validate_executable(game.executable.trim())?;
    let arguments = validate_arguments(&game.arguments)?;
    let accent = game
        .accent
        .map(|accent| validate_accent(&accent))
        .transpose()?;
    let id = format!("manual:{}", executable.display());
    let cover = game
        .cover
        .filter(|path| !path.trim().is_empty())
        .map(|path| {
            let images_dir = lock(&state.metadata).images_dir().to_path_buf();
            import_cover(Path::new(path.trim()), &images_dir, &id)
        })
        .transpose()?;
    update_config(&state, |config| {
        if config.manual_games.iter().any(|game| game.id == id) {
            return Err("Ce jeu est déjà dans la liste.".to_string());
        }
        config.manual_games.push(ManualGame {
            id: id.clone(),
            title,
            executable,
            arguments,
            accent,
            cover,
        });
        Ok(())
    })?;
    Ok(id)
}

#[tauri::command]
pub fn remove_manual_game(id: String, state: State<'_, AppState>) -> Result<(), String> {
    update_config(&state, |config| {
        if let Some(cover) = config
            .manual_games
            .iter()
            .find(|game| game.id == id)
            .and_then(|game| game.cover.as_ref())
        {
            let _ = fs::remove_file(cover);
        }
        config.manual_games.retain(|game| game.id != id);
        config.hidden_game_ids.retain(|hidden_id| hidden_id != &id);
        Ok(())
    })
}

pub fn validate_title(title: &str) -> Result<String, String> {
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

fn validate_arguments(arguments: &str) -> Result<Vec<String>, String> {
    let arguments = arguments.trim();
    if arguments.is_empty() {
        return Ok(Vec::new());
    }
    if arguments.chars().count() > MAX_ARGUMENTS_LENGTH {
        return Err("Les arguments sont trop longs.".to_string());
    }
    split_exec(arguments)
        .ok_or_else(|| "Y'a un guillemet pas fermé dans les arguments.".to_string())
}

fn validate_accent(accent: &str) -> Result<String, String> {
    let hex = accent.strip_prefix('#').unwrap_or_default();
    if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        Ok(format!("#{}", hex.to_ascii_lowercase()))
    } else {
        Err("Cette couleur n'est pas valide.".to_string())
    }
}

fn import_cover(source: &Path, images_dir: &Path, game_id: &str) -> Result<PathBuf, String> {
    let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|extension| COVER_EXTENSIONS.contains(&extension.as_str()))
        .ok_or("La jaquette doit être une image PNG, JPG ou WebP.")?;
    let size = fs::metadata(source)
        .ok()
        .filter(|metadata| metadata.is_file())
        .ok_or("Y'a pas d'image à ce chemin.")?
        .len();
    if size > MAX_COVER_BYTES {
        return Err("L'image fait plus de 20 Mo.".to_string());
    }
    let destination = images_dir.join(format!("{}-custom-cover.{extension}", file_stem(game_id)));
    fs::create_dir_all(images_dir)
        .and_then(|()| fs::copy(source, &destination))
        .map_err(|error| format!("Impossible de copier la jaquette : {error}"))?;
    Ok(destination)
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
    fn validates_arguments_and_colors() {
        assert_eq!(validate_arguments("  "), Ok(Vec::new()));
        assert_eq!(
            validate_arguments("--fullscreen \"--name=Mon jeu\""),
            Ok(vec![
                "--fullscreen".to_string(),
                "--name=Mon jeu".to_string()
            ])
        );
        assert!(validate_arguments("\"open").is_err());
        assert_eq!(validate_accent("#4A7BFF"), Ok("#4a7bff".to_string()));
        assert!(validate_accent("red").is_err());
        assert!(validate_accent("#12345").is_err());
    }

    #[test]
    fn rejects_relative_missing_or_non_executable_paths() {
        assert!(validate_executable("bin/sh").is_err());
        assert!(validate_executable("/does/not/exist").is_err());
        assert!(validate_executable("/etc/hostname").is_err());
        assert!(validate_executable("/bin/sh").is_ok());
    }
}
