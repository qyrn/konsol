use crate::achievements::{self, Achievement};
use crate::state::{AppState, lock};
use tauri::State;

#[tauri::command]
pub fn get_achievements(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Achievement>, String> {
    let source = lock(&state.games)
        .get(&id)
        .and_then(|game| game.achievement_source.clone())
        .ok_or("Pas de succès pour ce jeu.")?;
    let store = lock(&state.metadata);
    let rarity = store.achievement_rarity(&id);
    let mut list = achievements::read(&source, &state.home, store.emulator_schema(&id))
        .map(|list| list.achievements)
        .unwrap_or_default();
    for achievement in &mut list {
        achievement.rarity_tenths = rarity.and_then(|rarity| rarity.get(&achievement.id).copied());
    }
    Ok(list)
}
