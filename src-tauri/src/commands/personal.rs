use crate::config::Collection;
use crate::state::{AppState, lock, update_config};
use tauri::State;

const MAX_COLLECTIONS: usize = 40;
const MAX_COLLECTION_NAME: usize = 40;
const MAX_OFFSET_MINUTES: i64 = 100_000;

#[tauri::command]
pub fn get_collections(state: State<'_, AppState>) -> Result<Vec<Collection>, String> {
    Ok(lock(&state.config).load()?.collections)
}

#[tauri::command]
pub fn save_collections(
    collections: Vec<Collection>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if collections.len() > MAX_COLLECTIONS {
        return Err("Ça fait beaucoup de collections, garde les principales.".to_string());
    }
    let mut cleaned: Vec<Collection> = Vec::new();
    for collection in collections {
        let name = collection.name.trim().to_string();
        if name.is_empty() || name.chars().count() > MAX_COLLECTION_NAME {
            return Err(format!(
                "Donne un nom de {MAX_COLLECTION_NAME} caractères max à ta collection."
            ));
        }
        if cleaned
            .iter()
            .any(|existing| existing.name.eq_ignore_ascii_case(&name))
        {
            return Err(format!("T'as déjà une collection « {name} »."));
        }
        let mut game_ids = collection.game_ids;
        game_ids.dedup();
        cleaned.push(Collection { name, game_ids });
    }
    update_config(&state, |config| {
        config.collections = cleaned;
        Ok(())
    })
}

#[tauri::command]
pub fn add_playtime(id: String, minutes: i64, state: State<'_, AppState>) -> Result<(), String> {
    update_config(&state, |config| {
        let total = config.playtime_offsets.get(&id).copied().unwrap_or(0) + minutes;
        if total.abs() > MAX_OFFSET_MINUTES {
            return Err("Ce temps de jeu est un peu trop grand pour être vrai.".to_string());
        }
        if total == 0 {
            config.playtime_offsets.remove(&id);
        } else {
            config.playtime_offsets.insert(id, total);
        }
        Ok(())
    })
}
