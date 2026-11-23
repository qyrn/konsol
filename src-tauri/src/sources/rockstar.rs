use std::fs;
use std::path::{Path, PathBuf};

const LAUNCHER_DOCUMENTS: &str = "Documents/Rockstar Games/Launcher";
const SIGNED_IN_MARKER: &str = "' is now signed in";
const BUILD_MARKERS: [&str; 2] = ["Using build collection for ", "Using regular build for "];
const LAUNCHER_CODE: &str = "launcher";
const ROTATED_LOGS: usize = 9;
const TITLES: [(&str, &str); 14] = [
    ("gta5", "Grand Theft Auto V"),
    ("gta5_gen9", "Grand Theft Auto V Enhanced"),
    ("rdr2", "Red Dead Redemption 2"),
    ("rdr", "Red Dead Redemption"),
    ("gta4", "Grand Theft Auto IV"),
    ("gtaiv", "Grand Theft Auto IV"),
    ("lanoire", "L.A. Noire"),
    ("lanoirevr", "L.A. Noire The VR Case Files"),
    ("mp3", "Max Payne 3"),
    ("bully", "Bully Scholarship Edition"),
    ("gtasa", "Grand Theft Auto San Andreas"),
    ("gta3unreal", "Grand Theft Auto III The Definitive Edition"),
    (
        "gtavcunreal",
        "Grand Theft Auto Vice City The Definitive Edition",
    ),
    (
        "gtasaunreal",
        "Grand Theft Auto San Andreas The Definitive Edition",
    ),
];

pub struct OwnedTitle {
    pub code: String,
    pub title: &'static str,
}

fn launcher_directories(drive: &Path) -> Vec<PathBuf> {
    let Ok(users) = fs::read_dir(drive.join("users")) else {
        return Vec::new();
    };
    users
        .filter_map(Result::ok)
        .map(|user| user.path().join(LAUNCHER_DOCUMENTS))
        .filter(|directory| directory.is_dir())
        .collect()
}

pub fn watched_paths(drive: &Path) -> Vec<PathBuf> {
    launcher_directories(drive)
        .iter()
        .filter_map(|directory| fs::read_dir(directory.join("Profiles")).ok())
        .flat_map(|profiles| profiles.filter_map(Result::ok))
        .map(|profile| profile.path().join("profiletitles.dat"))
        .filter(|path| path.is_file())
        .collect()
}

fn log_files(directory: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    std::iter::once(directory.join("launcher.log")).chain(
        (1..=ROTATED_LOGS).map(move |index| directory.join(format!("launcher.{index:02}.log"))),
    )
}

fn codes_after_last_sign_in(log: &str) -> Option<Vec<String>> {
    let start = log.rfind(SIGNED_IN_MARKER)?;
    let mut codes: Vec<String> = Vec::new();
    for line in log[start..].lines() {
        let Some(code) = BUILD_MARKERS
            .iter()
            .find_map(|marker| line.split_once(marker))
            .map(|(_, code)| code.trim())
        else {
            continue;
        };
        if code != LAUNCHER_CODE && !codes.iter().any(|known| known == code) {
            codes.push(code.to_string());
        }
    }
    Some(codes)
}

fn signed_in_account_codes(directory: &Path) -> Option<Vec<String>> {
    log_files(directory)
        .filter_map(|path| fs::read(path).ok())
        .find_map(|bytes| codes_after_last_sign_in(&String::from_utf8_lossy(&bytes)))
}

pub fn owned_titles(drive: &Path) -> Vec<OwnedTitle> {
    launcher_directories(drive)
        .iter()
        .find_map(|directory| signed_in_account_codes(directory))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|code| {
            let title = TITLES
                .iter()
                .find(|(known, _)| *known == code)
                .map(|(_, title)| *title)?;
            Some(OwnedTitle { code, title })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "\
[16:08:53] [launcher] User 'first' is now signed in
[16:08:56] [mergedmanifest] Using build collection for gta5_gen9
[16:09:27] [launcher] User 'second' is now signed in
[16:09:55] [mergedmanifest] Using build collection for gta5
[16:09:56] [mergedmanifest] Using build collection for gta5_gen9
[16:09:56] [mergedmanifest] Using regular build for launcher
[16:09:57] [mergedmanifest] Using build collection for rdr2
[16:09:58] [mergedmanifest] Using build collection for rdr2
";

    #[test]
    fn keeps_only_the_titles_of_the_last_signed_in_account() {
        assert_eq!(
            codes_after_last_sign_in(LOG).unwrap(),
            ["gta5", "gta5_gen9", "rdr2"]
        );
        assert!(codes_after_last_sign_in("[launcher] Starting").is_none());
    }

    #[test]
    fn falls_back_to_rotated_logs_and_maps_codes_to_titles() {
        let drive = std::env::temp_dir().join(format!("konsol-rockstar-{}", std::process::id()));
        let directory = drive.join("users/steamuser").join(LAUNCHER_DOCUMENTS);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("launcher.log"), "[launcher] Starting\n").unwrap();
        fs::write(
            directory.join("launcher.01.log"),
            format!("{LOG}Using build collection for unknown_title\n"),
        )
        .unwrap();

        let titles: Vec<&str> = owned_titles(&drive)
            .iter()
            .map(|owned| owned.title)
            .collect();
        assert_eq!(
            titles,
            [
                "Grand Theft Auto V",
                "Grand Theft Auto V Enhanced",
                "Red Dead Redemption 2"
            ]
        );
        fs::remove_dir_all(drive).unwrap();
    }
}
