use crate::desktop_file::{self, DesktopFile};
use crate::game::{AchievementSource, Game, GameSource, LaunchTarget};
use std::fs;
use std::path::{Path, PathBuf};

const EMULATOR_STATS_FILE: &str = "SteamData/user_stats.ini";

pub fn scan(home: &Path) -> Vec<Game> {
    let Ok(entries) = fs::read_dir(home.join(".local/share/applications")) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "desktop")
        })
        .filter_map(|path| {
            let stem = path.file_stem()?.to_str()?.to_string();
            if stem.starts_with("konsol") {
                return None;
            }
            let entry = desktop_file::parse(&fs::read_to_string(&path).ok()?)?;
            game_from_entry(&stem, entry)
        })
        .collect()
}

fn game_from_entry(stem: &str, entry: DesktopFile) -> Option<Game> {
    if entry.no_display
        || !entry.categories.iter().any(|category| category == "Game")
        || is_store_shortcut(&entry.command)
        || is_launcher(&entry)
    {
        return None;
    }
    let target = launch_target_file(&entry.command)?;
    let working_directory = entry
        .working_directory
        .or_else(|| target.parent().map(Path::to_path_buf));
    let mut game = Game::launched_by_konsol(
        format!("desktop:{stem}"),
        entry.name,
        GameSource::Desktop,
        LaunchTarget::Command {
            arguments: entry.command,
            working_directory,
        },
    );
    game.achievement_source = target
        .parent()
        .map(|directory| directory.join(EMULATOR_STATS_FILE))
        .filter(|path| path.is_file())
        .map(|path| AchievementSource::EmulatorStats { path });
    Some(game)
}

fn is_store_shortcut(command: &[String]) -> bool {
    command
        .iter()
        .any(|argument| argument.starts_with("steam://") || argument.starts_with("heroic://"))
}

fn is_launcher(entry: &DesktopFile) -> bool {
    entry.name.to_lowercase().contains("launcher")
        || entry.command.iter().any(|argument| {
            Path::new(argument)
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.to_lowercase().contains("launcher"))
        })
}

fn launch_target_file(command: &[String]) -> Option<PathBuf> {
    command
        .iter()
        .rev()
        .find(|argument| argument.starts_with('/'))
        .map(PathBuf::from)
        .filter(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, command: &[&str]) -> DesktopFile {
        DesktopFile {
            name: name.into(),
            command: command
                .iter()
                .map(|argument| argument.to_string())
                .collect(),
            categories: vec!["Game".into()],
            working_directory: None,
            no_display: false,
        }
    }

    #[test]
    fn skips_launchers_store_shortcuts_and_missing_targets() {
        assert!(game_from_entry("a", entry("Rockstar Games Launcher", &["/bin/sh"])).is_none());
        assert!(
            game_from_entry("b", entry("Isaac", &["steam", "steam://rungameid/250900"])).is_none()
        );
        assert!(
            game_from_entry("c", entry("Gone", &["env", "A=1", "/does/not/exist.exe"])).is_none()
        );
    }

    #[test]
    fn keeps_real_games_with_their_full_command() {
        let game =
            game_from_entry("skate", entry("Skate 3", &["env", "X=1", "/bin/sh"])).expect("a game");
        assert_eq!(game.id, "desktop:skate");
        assert_eq!(
            game.launch,
            LaunchTarget::Command {
                arguments: vec!["env".into(), "X=1".into(), "/bin/sh".into()],
                working_directory: Some(PathBuf::from("/bin")),
            }
        );
    }
}
