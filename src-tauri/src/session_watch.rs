use crate::commands::library::LIBRARY_EVENT;
use crate::game::ProcessMarker;
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

struct WatchedGame {
    id: String,
    marker: ProcessMarker,
    tracked_by_konsol: bool,
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut previously_running: HashSet<String> = HashSet::new();
        let mut minimized_by_konsol = false;
        let mut last_tick = Instant::now();
        let mut last_save = Instant::now();
        loop {
            thread::sleep(TICK);
            let state = app.state::<AppState>();
            let watched: Vec<WatchedGame> = lock(&state.games)
                .values()
                .filter(|game| game.installed)
                .map(|game| WatchedGame {
                    id: game.id.clone(),
                    marker: game.process_marker.clone(),
                    tracked_by_konsol: game.playtime_tracked_by_konsol(),
                })
                .collect();
            let markers: Vec<(&WatchedGame, ProcessMarker)> = watched
                .iter()
                .map(|game| (game, game.marker.clone()))
                .collect();
            let running_games: Vec<&WatchedGame> = process::running_keys(&markers)
                .into_iter()
                .copied()
                .collect();
            let running: HashSet<String> =
                running_games.iter().map(|game| game.id.clone()).collect();

            let elapsed = last_tick.elapsed().min(TICK * 3).as_secs();
            last_tick = Instant::now();
            let tracked_running: Vec<&str> = running_games
                .iter()
                .filter(|game| game.tracked_by_konsol)
                .map(|game| game.id.as_str())
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
