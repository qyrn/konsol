use crate::achievement_watch::{self, WatchContext};
use crate::achievements;
use crate::artwork;
use crate::commands::manual::validate_title;
use crate::desktop_entry::DesktopPaths;
use crate::game::{Game, ProcessMarker};
use crate::launcher;
use crate::metadata::{self, ApiKeys};
use crate::process;
use crate::sources;
use crate::state::{AppState, lock, update_config};
use crate::window_watch;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

pub const LIBRARY_EVENT: &str = "library-changed";

#[tauri::command]
pub fn list_games(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    let config = lock(&state.config).load()?;
    let scanned = sources::scan_all(&state.home, &config);
    let keys = config.api_keys();
    let (mut games, needs_enrichment) = {
        let store = lock(&state.metadata);
        let games = metadata::apply(scanned.clone(), &store);
        let pending = metadata::needs_enrichment(&scanned, &store, &keys);
        let mut games = games;
        for game in &mut games {
            game.achievements = game.achievement_source.as_ref().and_then(|source| {
                achievements::read(source, &state.home, store.emulator_schema(&game.id))
                    .map(|list| list.summary())
            });
        }
        (games, pending)
    };
    fill_accents(&mut games, &state.accents);
    *lock(&state.games) = games
        .iter()
        .map(|game| (game.id.clone(), game.clone()))
        .collect();
    if needs_enrichment {
        start_enrichment(&app, scanned, keys);
    }
    Ok(games)
}

fn start_enrichment(app: &AppHandle, games: Vec<Game>, keys: ApiKeys) {
    let state = app.state::<AppState>();
    if state.enriching.swap(true, Ordering::SeqCst) {
        return;
    }
    let (snapshot, images_dir) = {
        let store = lock(&state.metadata);
        (store.snapshot(), store.images_dir().to_path_buf())
    };
    let app = app.clone();
    thread::spawn(move || {
        let updates = metadata::enrich(&games, snapshot, &keys, &images_dir);
        let state = app.state::<AppState>();
        let changed = lock(&state.metadata).merge(updates);
        state.enriching.store(false, Ordering::SeqCst);
        if changed {
            let _ = app.emit(LIBRARY_EVENT, ());
        }
    });
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
pub fn launch_game(id: String, app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let game = lock(&state.games)
        .get(&id)
        .cloned()
        .ok_or("Ce jeu n'est plus dans la liste, actualise la bibliothèque.")?;
    launcher::launch(&game)?;
    let emulator_schema = lock(&state.metadata)
        .emulator_schema(&game.id)
        .map(<[_]>::to_vec);
    achievement_watch::watch_achievements(
        game.clone(),
        WatchContext {
            app,
            home: state.home.clone(),
            icons_dir: state.achievement_icons_dir.clone(),
            emulator_schema,
        },
    );
    window_watch::watch_for_game_window(
        game,
        DesktopPaths {
            applications: state.applications_dir.clone(),
            icons: state.taskbar_icons_dir.clone(),
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
pub fn rename_game(id: String, title: String, state: State<'_, AppState>) -> Result<(), String> {
    let title = validate_title(&title)?;
    update_config(&state, |config| {
        config.custom_titles.insert(id.clone(), title);
        Ok(())
    })?;
    lock(&state.metadata).forget(&id);
    Ok(())
}

#[tauri::command]
pub fn running_game_ids(state: State<'_, AppState>) -> Vec<String> {
    let markers: Vec<(String, ProcessMarker)> = lock(&state.games)
        .values()
        .map(|game| (game.id.clone(), game.process_marker.clone()))
        .collect();
    process::running_keys(&markers)
        .into_iter()
        .cloned()
        .collect()
}
