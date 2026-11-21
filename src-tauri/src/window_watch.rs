use crate::desktop_entry::{self, DesktopPaths};
use crate::game::Game;
use crate::process::process_matches;
use std::collections::HashSet;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const WATCH_DURATION: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn watch_for_game_window(game: Game, paths: DesktopPaths) {
    thread::spawn(move || {
        let Some(existing) = list_windows() else {
            return;
        };
        let mut seen: HashSet<String> = existing.into_iter().collect();
        let started = Instant::now();
        while started.elapsed() < WATCH_DURATION {
            thread::sleep(POLL_INTERVAL);
            for window in list_windows().unwrap_or_default() {
                if !seen.insert(window.clone()) {
                    continue;
                }
                let belongs_to_game = window_pid(&window)
                    .is_some_and(|pid| process_matches(pid, &game.process_marker));
                if !belongs_to_game {
                    continue;
                }
                if let Some(class) = window_class(&window) {
                    if let Err(error) = desktop_entry::register(&game, &class, &paths) {
                        eprintln!("{error}");
                    }
                    return;
                }
            }
        }
    });
}

fn kdotool(arguments: &[&str]) -> Option<String> {
    let output = Command::new("kdotool").args(arguments).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn list_windows() -> Option<Vec<String>> {
    let output = kdotool(&["search", "--class", "."])?;
    Some(output.lines().map(str::to_string).collect())
}

fn window_pid(window: &str) -> Option<u32> {
    kdotool(&["getwindowpid", window])?.parse().ok()
}

fn window_class(window: &str) -> Option<String> {
    kdotool(&["getwindowclassname", window]).filter(|class| !class.is_empty())
}
