use crate::achievement_watch::{self, WatchContext};
use crate::achievements;
use crate::artwork;
use crate::commands::manual::validate_title;
use crate::desktop_entry::DesktopPaths;
use crate::game::{Game, GameSource, LaunchTarget};
use crate::journal_sessions::JournalSession;
use crate::kde_history;
use crate::launcher;
use crate::metadata::{self, ApiKeys, EnrichContext, http};
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
const MERGE_EVERY: usize = 8;

#[tauri::command]
pub fn list_games(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    let config = lock(&state.config).load()?;
    let scanned = sources::scan_all(&state.home, &config, &state.steam_library.apps());
    let keys = config.api_keys();
    let steam_library_due = keys.steam_web.is_some() && state.steam_library.is_stale();
    let (mut games, needs_enrichment) = {
        let store = lock(&state.metadata);
        let games = metadata::apply(scanned.clone(), &store);
        let pending = steam_library_due || metadata::needs_enrichment(&scanned, &store, &keys);
        let mut games = games;
        let playtime = lock(&state.playtime);
        let desktop_launches = kde_history::last_launches(&state.home);
        for game in &mut games {
            if game.playtime_tracked_by_konsol() {
                let tracked = playtime.get(&game.id).unwrap_or_default();
                let launched_from_menu = game
                    .id
                    .strip_prefix("desktop:")
                    .and_then(|stem| desktop_launches.get(stem))
                    .copied()
                    .unwrap_or(0);
                let last_played = tracked.last_played.max(launched_from_menu);
                game.details.playtime_minutes = Some(tracked.seconds / 60);
                game.details.last_played = (last_played > 0).then_some(last_played);
            }
            game.achievements = game.achievement_source.as_ref().and_then(|source| {
                achievements::read(source, &state.home, store.emulator_schema(&game.id))
                    .map(|list| list.summary())
            });
        }
        (games, pending)
    };
    ignore_aborted_steam_launches(&mut games, &lock(&state.journal));
    for game in &mut games {
        if let Some(offset) = config.playtime_offsets.get(&game.id) {
            let minutes = game.details.playtime_minutes.unwrap_or(0) as i64 + offset;
            game.details.playtime_minutes = Some(minutes.max(0).unsigned_abs());
        }
    }
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
    let app = app.clone();
    thread::spawn(move || {
        let state = app.state::<AppState>();
        let agent = http::agent();
        if refresh_steam_library(&state, &agent, &keys) {
            state.enriching.store(false, Ordering::SeqCst);
            let _ = app.emit(LIBRARY_EVENT, ());
            return;
        }
        load_catalog(&app, &agent, &games);
        let (snapshot, images_dir) = {
            let store = lock(&state.metadata);
            (store.snapshot(), store.images_dir().to_path_buf())
        };
        let context = EnrichContext {
            agent: &agent,
            keys: &keys,
            images_dir: &images_dir,
            home: &state.home,
        };
        let mut updates = HashMap::new();
        for game in &games {
            let current = snapshot.get(&game.id).cloned().unwrap_or_default();
            if let Some(update) = metadata::enrich_game(game, &current, &context) {
                updates.insert(game.id.clone(), update);
            }
            if updates.len() >= MERGE_EVERY
                && lock(&state.metadata).merge(std::mem::take(&mut updates))
            {
                let _ = app.emit(LIBRARY_EVENT, ());
            }
        }
        if lock(&state.metadata).merge(updates) {
            let _ = app.emit(LIBRARY_EVENT, ());
        }
        load_catalog(&app, &agent, &games);
        state.enriching.store(false, Ordering::SeqCst);
    });
}

fn load_catalog(app: &AppHandle, agent: &ureq::Agent, games: &[Game]) {
    let state = app.state::<AppState>();
    let (requests, snapshot) = {
        let store = lock(&state.metadata);
        (metadata::catalog_requests(games, &store), store.snapshot())
    };
    if requests.is_empty() {
        return;
    }
    for updates in metadata::fetch_catalog(agent, &requests, &snapshot) {
        if lock(&state.metadata).merge(updates) {
            let _ = app.emit(LIBRARY_EVENT, ());
        }
    }
}

fn refresh_steam_library(state: &AppState, agent: &ureq::Agent, keys: &ApiKeys) -> bool {
    let (Some(key), Some(steam_id)) = (
        keys.steam_web.as_deref(),
        sources::logged_in_steam_id(&state.home),
    ) else {
        return false;
    };
    state.steam_library.is_stale() && state.steam_library.refresh(agent, key, &steam_id)
}

const ABORTED_LAUNCH_SECONDS: u64 = 120;
const SAME_SESSION_TOLERANCE: i64 = 300;

fn ignore_aborted_steam_launches(games: &mut [Game], journal: &[JournalSession]) {
    for game in games.iter_mut().filter(|game| game.installed) {
        let (GameSource::Steam, LaunchTarget::SteamApp(app_id), Some(last_played)) =
            (game.source, &game.launch, game.details.last_played)
        else {
            continue;
        };
        let stem = format!("steam-app{app_id}");
        let sessions: Vec<&JournalSession> = journal
            .iter()
            .filter(|session| session.desktop_stem == stem)
            .collect();
        let aborted = sessions.iter().any(|session| {
            (session.ended_at - last_played).abs() <= SAME_SESSION_TOLERANCE
                && session.seconds < ABORTED_LAUNCH_SECONDS
        });
        if aborted {
            game.details.last_played = sessions
                .iter()
                .filter(|session| session.seconds >= ABORTED_LAUNCH_SECONDS)
                .map(|session| session.ended_at)
                .max();
        }
    }
}

fn fill_accents(games: &mut [Game], cache: &Mutex<HashMap<PathBuf, Option<String>>>) {
    let mut cache = lock(cache);
    for game in games.iter_mut().filter(|game| game.accent.is_none()) {
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
    if !game.installed {
        return Ok(());
    }
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
    lock(&state.running).clone()
}

#[tauri::command]
pub fn uninstall_game(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let game = lock(&state.games)
        .get(&id)
        .cloned()
        .ok_or("Ce jeu n'est plus dans la liste, actualise la bibliothèque.")?;
    launcher::uninstall(&game)
}

#[tauri::command]
pub fn stop_game(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let marker = lock(&state.games)
        .get(&id)
        .map(|game| game.process_marker.clone())
        .ok_or("Ce jeu n'est plus dans la liste, actualise la bibliothèque.")?;
    process::stop_matching(&marker)
}
