use crate::desktop_entry::{self, DesktopPaths};
use crate::game::{Game, ProcessMarker};
use std::collections::HashSet;
use std::fs;
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

fn process_matches(pid: u32, marker: &ProcessMarker) -> bool {
    let environment = fs::read(format!("/proc/{pid}/environ")).unwrap_or_default();
    match marker {
        ProcessMarker::EnvVar { name, value } => {
            let expected = format!("{name}={value}");
            environment
                .split(|byte| *byte == 0)
                .any(|entry| entry == expected.as_bytes())
        }
        ProcessMarker::Mentions(text) => {
            let command_line = fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
            contains(&environment, text.as_bytes()) || contains(&command_line, text.as_bytes())
        }
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_current_process_by_environment() {
        let pid = std::process::id();
        let path = std::env::var("PATH").expect("PATH is set");
        assert!(process_matches(
            pid,
            &ProcessMarker::EnvVar {
                name: "PATH",
                value: path
            }
        ));
        assert!(!process_matches(
            pid,
            &ProcessMarker::EnvVar {
                name: "PATH",
                value: "nope".into()
            }
        ));
        assert!(!process_matches(
            pid,
            &ProcessMarker::Mentions(String::new())
        ));
    }
}
