use crate::commands::library::LIBRARY_EVENT;
use crate::game::{Game, KONSOL_GAME_ID_VARIABLE, ProcessMarker};
use crate::journal_sessions;
use crate::process;
use crate::state::{AppState, lock};
use crate::time::now_unix;
use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const TICK: Duration = Duration::from_secs(2);
const SAVE_EVERY: Duration = Duration::from_secs(60);
const MAIN_WINDOW: &str = "main";

const IMPORT_JOURNAL_EVERY: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Purpose {
    Running,
    Playtime,
}

fn playtime_marker(game: &Game) -> Option<ProcessMarker> {
    if !game.playtime_tracked_by_konsol() {
        return None;
    }
    if game.id.starts_with("desktop:") {
        return Some(ProcessMarker::EnvVar {
            name: KONSOL_GAME_ID_VARIABLE,
            value: game.id.clone(),
        });
    }
    Some(game.process_marker.clone())
}

fn import_menu_sessions(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let desktop_ids: HashSet<String> = lock(&state.games)
        .keys()
        .filter(|id| id.starts_with("desktop:"))
        .cloned()
        .collect();
    if desktop_ids.is_empty() {
        return false;
    }
    let sessions = journal_sessions::read();
    let mut playtime = lock(&state.playtime);
    let imported = playtime.import(&sessions, &desktop_ids);
    if imported {
        playtime.save();
    }
    imported
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut previously_running: HashSet<String> = HashSet::new();
        let mut minimized_by_konsol = false;
        let mut last_tick = Instant::now();
        let mut last_save = Instant::now();
        let mut last_import: Option<Instant> = None;
        loop {
            thread::sleep(TICK);
            let state = app.state::<AppState>();
            if last_import.is_none_or(|imported| imported.elapsed() >= IMPORT_JOURNAL_EVERY) {
                last_import = Some(Instant::now());
                if import_menu_sessions(&app) {
                    let _ = app.emit(LIBRARY_EVENT, ());
                }
            }
            let markers: Vec<((String, Purpose), ProcessMarker)> = lock(&state.games)
                .values()
                .filter(|game| game.installed)
                .flat_map(|game| {
                    let running = (
                        (game.id.clone(), Purpose::Running),
                        game.process_marker.clone(),
                    );
                    let playtime = playtime_marker(game)
                        .map(|marker| ((game.id.clone(), Purpose::Playtime), marker));
                    std::iter::once(running).chain(playtime)
                })
                .collect();
            let matched = process::running_keys(&markers);
            let running: HashSet<String> = matched
                .iter()
                .filter(|(_, purpose)| *purpose == Purpose::Running)
                .map(|(id, _)| id.clone())
                .collect();

            let elapsed = last_tick.elapsed().min(TICK * 3).as_secs();
            last_tick = Instant::now();
            let tracked_running: Vec<&str> = matched
                .iter()
                .filter(|(_, purpose)| *purpose == Purpose::Playtime)
                .map(|(id, _)| id.as_str())
                .collect();
            let session_ended = previously_running.difference(&running).next().is_some();
            {
                let mut playtime = lock(&state.playtime);
                for id in &tracked_running {
                    playtime.add(id, elapsed, now_unix());
                }
                if session_ended
                    || (!tracked_running.is_empty() && last_save.elapsed() >= SAVE_EVERY)
                {
                    playtime.save();
                    last_save = Instant::now();
                }
            }

            let session_started = running.difference(&previously_running).next().is_some();
            let minimize_wanted = lock(&state.config)
                .load()
                .is_ok_and(|config| config.preferences.minimize_during_session);
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                if session_started && minimize_wanted && !minimized_by_konsol {
                    minimized_by_konsol = window.minimize().is_ok();
                }
                if running.is_empty() && minimized_by_konsol {
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    minimized_by_konsol = false;
                }
            }

            *lock(&state.running) = running.iter().cloned().collect();
            if session_ended {
                let _ = app.emit(LIBRARY_EVENT, ());
            }
            previously_running = running;
        }
    });
}
