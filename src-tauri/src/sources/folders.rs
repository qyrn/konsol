use crate::game::{AchievementSource, Game, GameSource, LaunchTarget, ProcessMarker};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const DEFAULT_FOLDERS: [&str; 2] = ["Games", "Jeux"];
const EMULATOR_STATS_FILE: &str = "SteamData/user_stats.ini";
const MAX_DEPTH: usize = 2;
const SKIPPED_FILES: [&str; 37] = [
    "unins",
    "setup",
    "install",
    "crash",
    "redist",
    "dxsetup",
    "launcher",
    "update",
    "helper",
    "config",
    "benchmark",
    "server",
    "report",
    "cef",
    "webhelper",
    "7z",
    "python",
    "java",
    "dotnet",
    "notification",
    "overlay",
    "touchup",
    "cleanup",
    "handler",
    "repair",
    "register",
    "prereq",
    "easyanticheat",
    "battleye",
    "be_service",
    "vc_",
    "sfv",
    "editor",
    "patch",
    "uploader",
    "tool",
    "unity",
];
const SKIPPED_FOLDERS: [&str; 27] = [
    "galaxy",
    "launcher",
    "ea app",
    "social club",
    "ea desktop",
    "epic games",
    "uplay",
    "steam",
    "battle.net",
    "origin",
    "microsoft",
    "windows",
    "internet explorer",
    "common files",
    "heroic",
    "backup",
    "saves",
    "mods",
    "redist",
    "directx",
    "dotnet",
    "support",
    "installer",
    "engine",
    "prerequisites",
    "_commonredist",
    "traduction",
];
const PUBLISHER_FOLDERS: [&str; 6] = [
    "rockstar games",
    "electronic arts",
    "ea games",
    "ubisoft",
    "bethesda softworks",
    "2k games",
];
const PREFIX_GAME_FOLDERS: [&str; 4] =
    ["Games", "GOG Games", "Program Files", "Program Files (x86)"];

struct GameFolder {
    directory: PathBuf,
    wine_prefix: Option<PathBuf>,
}

pub fn default_folders(home: &Path) -> Vec<PathBuf> {
    DEFAULT_FOLDERS
        .iter()
        .map(|name| home.join(name))
        .filter(|path| path.is_dir())
        .collect()
}

pub fn scan(folders: &[PathBuf], known_targets: &HashSet<PathBuf>) -> Vec<Game> {
    folders
        .iter()
        .flat_map(|folder| game_folders(folder, 0))
        .filter_map(|folder| {
            let executable = main_executable(&folder.directory)?;
            let canonical = fs::canonicalize(&executable).unwrap_or_else(|_| executable.clone());
            if known_targets.contains(&canonical) {
                return None;
            }
            Some(game_from_folder(&folder, &executable))
        })
        .collect()
}

fn is_skipped(name: &str, words: &[&str]) -> bool {
    let name = name.to_lowercase();
    words.iter().any(|word| name.contains(word))
}

fn visible_directories(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| !name.starts_with('.') && !is_skipped(name, &SKIPPED_FOLDERS))
        })
        .collect()
}

fn is_publisher(directory: &Path) -> bool {
    directory
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .is_some_and(|name| PUBLISHER_FOLDERS.contains(&name.as_str()))
}

fn drive_c(prefix: &Path) -> Option<PathBuf> {
    ["drive_c", "pfx/drive_c"]
        .iter()
        .map(|relative| prefix.join(relative))
        .find(|path| path.is_dir())
}

fn game_folders(folder: &Path, depth: usize) -> Vec<GameFolder> {
    let mut found = Vec::new();
    for directory in visible_directories(folder) {
        if let Some(drive) = drive_c(&directory) {
            found.extend(
                PREFIX_GAME_FOLDERS
                    .iter()
                    .flat_map(|name| visible_directories(&drive.join(name)))
                    .flat_map(|candidate| {
                        if is_publisher(&candidate) {
                            visible_directories(&candidate)
                        } else {
                            vec![candidate]
                        }
                    })
                    .map(|game_directory| GameFolder {
                        directory: game_directory,
                        wine_prefix: Some(directory.clone()),
                    }),
            );
        } else if depth == 0
            && visible_directories(&directory)
                .iter()
                .any(|child| drive_c(child).is_some())
        {
            found.extend(game_folders(&directory, depth + 1));
        } else {
            found.push(GameFolder {
                directory,
                wine_prefix: None,
            });
        }
    }
    found
}

fn is_native_executable(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name.ends_with(".so") || name.contains(".so.") {
        return false;
    }
    let executable =
        fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0);
    let mut magic = [0u8; 4];
    executable
        && File::open(path)
            .and_then(|mut file| file.read_exact(&mut magic))
            .is_ok()
        && magic == *b"\x7fELF"
}

struct Candidate {
    path: PathBuf,
    depth: usize,
    size: u64,
}

fn executables(directory: &Path, depth: usize) -> Vec<Candidate> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_lowercase();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if name.starts_with('.') || kind.is_symlink() {
            continue;
        }
        if kind.is_file() {
            let windows = name.ends_with(".exe");
            if !is_skipped(&name, &SKIPPED_FILES) && (windows || is_native_executable(&path)) {
                let size = entry.metadata().map_or(0, |metadata| metadata.len());
                found.push(Candidate { path, depth, size });
            }
        } else if kind.is_dir() && depth < MAX_DEPTH && !is_skipped(&name, &SKIPPED_FOLDERS) {
            found.extend(executables(&path, depth + 1));
        }
    }
    found
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn main_executable(directory: &Path) -> Option<PathBuf> {
    let folder_name = compact(&directory.file_name()?.to_string_lossy());
    let name_matches = |candidate: &Candidate| {
        let stem = candidate
            .path
            .file_stem()
            .map(|stem| compact(&stem.to_string_lossy()))
            .unwrap_or_default();
        !stem.is_empty() && (folder_name.contains(&stem) || stem.contains(&folder_name))
    };
    executables(directory, 0)
        .into_iter()
        .max_by_key(|candidate| {
            let starts_with_play = candidate
                .path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().to_lowercase().starts_with("play"));
            (
                name_matches(candidate),
                starts_with_play,
                MAX_DEPTH - candidate.depth,
                candidate.size,
            )
        })
        .map(|candidate| candidate.path)
}

pub fn pretty_title(directory_name: &str) -> String {
    let spaced: String = directory_name
        .chars()
        .map(|character| {
            if character == '_' || character == '-' {
                ' '
            } else {
                character
            }
        })
        .collect();
    let mut title = String::new();
    let mut previous: Option<char> = None;
    for character in spaced.chars() {
        if let Some(before) = previous
            && (before.is_lowercase() || before.is_ascii_digit())
            && character.is_uppercase()
        {
            title.push(' ');
        }
        title.push(character);
        previous = Some(character);
    }
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn launch_command(executable: &Path, wine_prefix: Option<&Path>) -> Vec<String> {
    let path = executable.to_string_lossy().into_owned();
    let is_windows = path.to_lowercase().ends_with(".exe");
    if !is_windows {
        return vec![path];
    }
    let mut command = vec!["env".to_string()];
    if let Some(prefix) = wine_prefix {
        command.push(format!("WINEPREFIX={}", prefix.display()));
    }
    if which("umu-run") {
        command.extend(["GAMEID=0".to_string(), "umu-run".to_string()]);
    } else {
        command.push("wine".to_string());
    }
    command.push(path);
    command
}

fn which(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|directory| directory.join(program).is_file())
    })
}

fn game_from_folder(folder: &GameFolder, executable: &Path) -> Game {
    let name = folder
        .directory
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let location = executable.to_string_lossy().into_owned();
    let mut game = Game::launched_by_konsol(
        format!("folder:{}", folder.directory.display()),
        pretty_title(&name),
        GameSource::Desktop,
        LaunchTarget::Command {
            arguments: launch_command(executable, folder.wine_prefix.as_deref()),
            working_directory: executable.parent().map(Path::to_path_buf),
        },
    );
    game.details.location = Some(location.clone());
    game.achievement_source = executable
        .parent()
        .map(|directory| directory.join(EMULATOR_STATS_FILE))
        .filter(|path| path.is_file())
        .map(|path| AchievementSource::EmulatorStats { path });
    game.also_running_when(ProcessMarker::Mentions(location))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_executable(path: &Path, content: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn finds_native_and_prefixed_games_but_skips_tools_and_launchers() {
        let root = std::env::temp_dir().join(format!("konsol-folders-{}", std::process::id()));
        let games = root.join("Games");
        write_executable(&games.join("Skate3Recomp/Linux/skate3"), b"\x7fELF-game");
        write_executable(&games.join("Skate3Recomp/Linux/crash_reporter"), b"\x7fELF");
        write_executable(
            &games.join("prefixes/sss/drive_c/Games/SSS2037/game.exe"),
            b"MZ",
        );
        write_executable(
            &games.join("prefixes/sss/drive_c/Games/SSS2037/unins000.exe"),
            b"MZ",
        );
        write_executable(
            &games.join("prefixes/gog/drive_c/Program Files (x86)/GOG Galaxy/GalaxyClient.exe"),
            b"MZ",
        );
        write_executable(&games.join("sims4-saves-backup/readme.txt"), b"x");
        let rockstar = games.join("prefixes/rockstar/drive_c/Program Files/Rockstar Games");
        write_executable(&rockstar.join("Grand Theft Auto V/PlayGTAV.exe"), b"MZ");
        write_executable(
            &rockstar.join("Grand Theft Auto V/GTA5.exe"),
            b"MZ-bigger-binary",
        );
        write_executable(&rockstar.join("Launcher/Launcher.exe"), b"MZ");

        let found = scan(std::slice::from_ref(&games), &HashSet::new());
        let titles: Vec<&str> = found.iter().map(|game| game.title.as_str()).collect();
        assert_eq!(found.len(), 3, "{titles:?}");
        let gta = found
            .iter()
            .find(|game| game.title == "Grand Theft Auto V")
            .expect("GTA V");
        assert!(
            gta.details
                .location
                .as_deref()
                .is_some_and(|path| path.ends_with("PlayGTAV.exe"))
        );
        assert!(titles.contains(&"Skate3 Recomp"));
        assert!(titles.contains(&"SSS2037"));
        let sss = found.iter().find(|game| game.title == "SSS2037").unwrap();
        let LaunchTarget::Command { arguments, .. } = &sss.launch else {
            panic!("expected a command");
        };
        assert!(
            arguments
                .iter()
                .any(|argument| argument.starts_with("WINEPREFIX="))
        );

        let known: HashSet<PathBuf> =
            [fs::canonicalize(games.join("Skate3Recomp/Linux/skate3")).unwrap()].into();
        assert_eq!(scan(&[games], &known).len(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn turns_folder_names_into_titles() {
        assert_eq!(pretty_title("Skate3Recomp"), "Skate3 Recomp");
        assert_eq!(
            pretty_title("hollow_knight-silksong"),
            "hollow knight silksong"
        );
    }
}
