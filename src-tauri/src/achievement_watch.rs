use crate::achievements::{self, Achievement, SchemaEntry};
use crate::desktop_entry::file_stem;
use crate::game::Game;
use crate::metadata::http;
use crate::process::any_process_matches;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Emitter};

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const STARTUP_GRACE: Duration = Duration::from_secs(180);
const FINAL_FLUSH_DELAY: Duration = Duration::from_secs(6);
pub const ACHIEVEMENT_EVENT: &str = "achievement-unlocked";

pub struct WatchContext {
    pub app: AppHandle,
    pub home: PathBuf,
    pub icons_dir: PathBuf,
    pub emulator_schema: Option<Vec<SchemaEntry>>,
}

pub fn watch_achievements(game: Game, context: WatchContext) {
    let Some(source) = game.achievement_source.clone() else {
        return;
    };
    thread::spawn(move || {
        let read = || {
            achievements::read(&source, &context.home, context.emulator_schema.as_deref())
                .map(|list| list.achievements)
                .unwrap_or_default()
        };
        let mut known: HashSet<String> = unlocked_ids(&read());
        let files = achievements::watched_files(&source, &context.home);
        let mut last_seen = modification_times(&files);
        let started = Instant::now();
        let mut seen_running = false;
        loop {
            thread::sleep(POLL_INTERVAL);
            let running = any_process_matches(&game.process_marker);
            seen_running |= running;
            let finished = !running && (seen_running || started.elapsed() > STARTUP_GRACE);
            if finished {
                thread::sleep(FINAL_FLUSH_DELAY);
            }
            let current = modification_times(&achievements::watched_files(&source, &context.home));
            if current != last_seen {
                last_seen = current;
                let unlocked_now: Vec<Achievement> = read()
                    .into_iter()
                    .filter(|achievement| achievement.unlocked && !known.contains(&achievement.id))
                    .collect();
                for achievement in &unlocked_now {
                    notify(&game, achievement, &context.icons_dir);
                    known.insert(achievement.id.clone());
                }
                if !unlocked_now.is_empty() {
                    let _ = context.app.emit(ACHIEVEMENT_EVENT, &game.id);
                }
            }
            if finished {
                return;
            }
        }
    });
}

fn unlocked_ids(achievements: &[Achievement]) -> HashSet<String> {
    achievements
        .iter()
        .filter(|achievement| achievement.unlocked)
        .map(|achievement| achievement.id.clone())
        .collect()
}

fn modification_times(files: &[PathBuf]) -> Vec<Option<SystemTime>> {
    files
        .iter()
        .map(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .ok()
        })
        .collect()
}

fn notify(game: &Game, achievement: &Achievement, icons_dir: &Path) {
    let icon = achievement
        .icon_url
        .as_deref()
        .and_then(|url| cached_icon(game, url, icons_dir))
        .map_or_else(
            || "konsol".to_string(),
            |path| path.to_string_lossy().into_owned(),
        );
    let mut body = format!("Succès débloqué dans {}", game.title);
    if !achievement.description.is_empty() {
        body.push('\n');
        body.push_str(&achievement.description);
    }
    let _ = Command::new("notify-send")
        .arg("--app-name=Konsol")
        .arg(format!("--icon={icon}"))
        .arg("--expire-time=8000")
        .arg(&achievement.name)
        .arg(escape_markup(&body))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|mut child| thread::spawn(move || child.wait()));
}

fn cached_icon(game: &Game, url: &str, icons_dir: &Path) -> Option<PathBuf> {
    let file_name: String = url
        .rsplit('/')
        .next()?
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '.' || *character == '_'
        })
        .collect();
    let destination = icons_dir.join(format!("{}-{file_name}", file_stem(&game.id)));
    (destination.is_file() || http::download(&http::agent(), url, &destination))
        .then_some(destination)
}

fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_notification_markup() {
        assert_eq!(
            escape_markup("<b>Tom & Jerry</b>"),
            "&lt;b&gt;Tom &amp; Jerry&lt;/b&gt;"
        );
    }
}
