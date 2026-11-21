use crate::metadata::discover::{self, Discovery, DiscoveryCache};
use crate::metadata::http;
use crate::state::{AppState, lock};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn discover_games(app: AppHandle) -> Result<Vec<Discovery>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let cache = DiscoveryCache::new(&state.cache_dir);
        if let Some(items) = cache.fresh() {
            return items;
        }
        let games: Vec<_> = lock(&state.games).values().cloned().collect();
        let items = discover::discover(&http::agent(), &games);
        if !items.is_empty() {
            cache.store(&items);
        }
        items
    })
    .await
    .map_err(|error| format!("Impossible de chercher des jeux : {error}"))
}

#[tauri::command]
pub fn open_store_page(app_id: u32) -> Result<(), String> {
    Command::new("steam")
        .arg(format!("steam://store/{app_id}"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Impossible d'ouvrir Steam : {error}"))
}
