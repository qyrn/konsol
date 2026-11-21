use crate::state::{AppState, lock, update_config};
use serde::Serialize;
use tauri::State;

const MAX_KEY_LENGTH: usize = 128;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    has_steam_grid_db_key: bool,
    has_steam_web_api_key: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsView, String> {
    let config = lock(&state.config).load()?;
    Ok(SettingsView {
        has_steam_grid_db_key: config.steam_grid_db_api_key.is_some(),
        has_steam_web_api_key: config.steam_web_api_key.is_some(),
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
