use crate::achievements::{self, Achievement};
use crate::game::{GameSource, LaunchTarget};
use crate::metadata::{self, http};
use crate::sources;
use crate::state::{AppState, lock};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn get_achievements(id: String, app: AppHandle) -> Result<Vec<Achievement>, String> {
    tauri::async_runtime::spawn_blocking(move || read_achievements(&id, &app))
        .await
        .map_err(|error| format!("Impossible de lire les succès : {error}"))?
}

fn read_achievements(id: &str, app: &AppHandle) -> Result<Vec<Achievement>, String> {
    let state = app.state::<AppState>();
    let game = lock(&state.games)
        .get(id)
        .cloned()
        .ok_or("Ce jeu n'est plus dans la liste, actualise la bibliothèque.")?;
    let local = game.achievement_source.as_ref().and_then(|source| {
        let store = lock(&state.metadata);
        achievements::read(source, &state.home, store.emulator_schema(id))
            .map(|list| list.achievements)
    });
    let mut list = match local {
        Some(list) => list,
        None => from_steam_account(&state, &game.source, &game.launch)
            .ok_or("Pas de succès pour ce jeu.")?,
    };
    let store = lock(&state.metadata);
    let rarity = store.achievement_rarity(id);
    for achievement in &mut list {
        achievement.rarity_tenths = rarity.and_then(|rarity| rarity.get(&achievement.id).copied());
    }
    Ok(list)
}

fn from_steam_account(
    state: &AppState,
    source: &GameSource,
    launch: &LaunchTarget,
) -> Option<Vec<Achievement>> {
    let (GameSource::Steam, LaunchTarget::SteamApp(app_id)) = (source, launch) else {
        return None;
    };
    let key = lock(&state.config).load().ok()?.steam_web_api_key?;
    let steam_id = sources::logged_in_steam_id(&state.home)?;
    metadata::player_achievements(&http::agent(), &key, &steam_id, app_id.parse().ok()?)
        .filter(|list| !list.is_empty())
}
