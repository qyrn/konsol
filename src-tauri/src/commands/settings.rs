use crate::config::Preferences;
use crate::state::{AppState, lock, update_config};
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::State;

const MAX_KEY_LENGTH: usize = 128;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    has_steam_grid_db_key: bool,
    has_steam_web_api_key: bool,
    preferences: Preferences,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsView, String> {
    let config = lock(&state.config).load()?;
    Ok(SettingsView {
        has_steam_grid_db_key: config.steam_grid_db_api_key.is_some(),
        has_steam_web_api_key: config.steam_web_api_key.is_some(),
        preferences: config.preferences,
    })
}

#[tauri::command]
pub fn save_settings(
    steam_grid_db_key: Option<String>,
    steam_web_api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let steam_grid_db_key = steam_grid_db_key
        .map(|key| validate_key(&key))
        .transpose()?;
    let steam_web_api_key = steam_web_api_key
        .map(|key| validate_key(&key))
        .transpose()?;
    update_config(&state, |config| {
        if let Some(key) = steam_grid_db_key {
            config.steam_grid_db_api_key = key;
        }
        if let Some(key) = steam_web_api_key {
            config.steam_web_api_key = key;
        }
        Ok(())
    })?;
    lock(&state.metadata).reset_key_lookups();
    Ok(())
}

#[tauri::command]
pub fn save_preferences(
    preferences: Preferences,
    state: State<'_, AppState>,
) -> Result<(), String> {
    set_launch_at_login(&state.home, preferences.launch_at_login)?;
    update_config(&state, |config| {
        config.preferences = preferences;
        Ok(())
    })
}

fn set_launch_at_login(home: &Path, enabled: bool) -> Result<(), String> {
    let entry_path = home.join(".config/autostart/konsol.desktop");
    if !enabled {
        return match fs::remove_file(&entry_path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                Err(format!("Impossible de retirer le démarrage auto : {error}"))
            }
            _ => Ok(()),
        };
    }
    let executable = std::env::current_exe()
        .map_err(|error| format!("Impossible de trouver Konsol : {error}"))?;
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Konsol\nExec=\"{}\"\nIcon=konsol\nX-GNOME-Autostart-enabled=true\n",
        executable.display()
    );
    entry_path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(&entry_path, entry))
        .map_err(|error| format!("Impossible d'activer le démarrage auto : {error}"))
}

fn validate_key(key: &str) -> Result<Option<String>, String> {
    let key = key.trim();
    if key.is_empty() {
        return Ok(None);
    }
    if key.len() > MAX_KEY_LENGTH
        || !key
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return Err("Cette clé a un format bizarre, recopie-la sans espace.".to_string());
    }
    Ok(Some(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_alphanumeric_keys_and_clears_on_empty() {
        assert_eq!(validate_key("  ABCdef123  "), Ok(Some("ABCdef123".into())));
        assert_eq!(validate_key("   "), Ok(None));
        assert!(validate_key("abc&key=1").is_err());
        assert!(validate_key(&"a".repeat(MAX_KEY_LENGTH + 1)).is_err());
    }
}
