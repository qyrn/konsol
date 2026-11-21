use crate::game::{Game, LaunchTarget};
use crate::launcher;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct DesktopPaths {
    pub applications: PathBuf,
    pub icons: PathBuf,
}

pub fn file_stem(game_id: &str) -> String {
    let slug: String = game_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("konsol-{}", slug.trim_matches('-'))
}

pub fn register(game: &Game, window_class: &str, paths: &DesktopPaths) -> Result<(), String> {
    let stem = file_stem(&game.id);
    let entry_path = paths.applications.join(format!("{stem}.desktop"));
    if current_window_class(&entry_path).as_deref() == Some(window_class) {
        return Ok(());
    }
    fs::create_dir_all(&paths.icons)
        .and_then(|()| fs::create_dir_all(&paths.applications))
        .map_err(|error| format!("Impossible de créer les dossiers : {error}"))?;
    let icon = write_icon(game, &stem, &paths.icons)?;
    fs::write(&entry_path, render(game, window_class, &icon))
        .map_err(|error| format!("Impossible d'écrire le raccourci : {error}"))?;
    refresh_menu_cache();
    Ok(())
}

fn write_icon(game: &Game, stem: &str, icons_dir: &Path) -> Result<PathBuf, String> {
    if let Some(source) = &game.icon_source {
        let destination = icons_dir.join(format!("{stem}.png"));
        if crate::artwork::write_png_icon(source, &destination).is_ok() {
            return Ok(destination);
        }
    }
    let destination = icons_dir.join(format!("{stem}.svg"));
    crate::artwork::write_letter_icon(&game.title, game.accent.as_deref(), &destination)?;
    Ok(destination)
}

fn current_window_class(entry_path: &Path) -> Option<String> {
    fs::read_to_string(entry_path)
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("StartupWMClass="))
        .map(str::to_string)
}

fn refresh_menu_cache() {
    if let Ok(mut child) = Command::new("kbuildsycoca6")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        std::thread::spawn(move || child.wait());
    }
}

fn render(game: &Game, window_class: &str, icon: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nIcon={}\nStartupWMClass={}\nCategories=Game;\nX-Konsol-Id={}\n",
        single_line(&game.title),
        exec_line(&game.launch),
        single_line(&icon.to_string_lossy()),
        single_line(window_class),
        single_line(&game.id),
    )
}

fn single_line(value: &str) -> String {
    value.replace(['\n', '\r'], " ")
}

fn exec_line(target: &LaunchTarget) -> String {
    launcher::command_line(target)
        .iter()
        .map(|argument| quote_argument(argument))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_argument(argument: &str) -> String {
    let mut quoted = String::from("\"");
    for character in single_line(argument).chars() {
        match character {
            '"' | '`' | '$' | '\\' => {
                quoted.push_str("\\\\");
                quoted.push(character);
            }
            '%' => quoted.push_str("%%"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameSource;

    #[test]
    fn builds_file_names_from_game_ids() {
        assert_eq!(file_stem("steam:250900"), "konsol-steam-250900");
        assert_eq!(
            file_stem("desktop:Skate 3.Recomp"),
            "konsol-desktop-skate-3-recomp"
        );
    }

    #[test]
    fn quotes_exec_arguments_per_the_desktop_entry_spec() {
        assert_eq!(
            exec_line(&LaunchTarget::Heroic {
                runner: "gog",
                app_name: "1423049311".into()
            }),
            "\"heroic\" \"--no-gui\" \"heroic://launch?appName=1423049311&runner=gog\""
        );
        assert_eq!(
            exec_line(&LaunchTarget::Program("/home/a b/50%$x.sh".into())),
            "\"/home/a b/50%%\\\\$x.sh\""
        );
    }

    #[test]
    fn renders_a_complete_entry() {
        let game = Game::launched_by_konsol(
            "steam:250900".into(),
            "Isaac\nInjected=1".into(),
            GameSource::Steam,
            LaunchTarget::SteamApp("250900".into()),
        );
        let entry = render(&game, "steam_app_250900", Path::new("/icons/isaac.png"));
        assert!(entry.contains("Name=Isaac Injected=1\n"));
        assert!(entry.contains("StartupWMClass=steam_app_250900\n"));
        assert!(entry.contains("Exec=\"steam\" \"-silent\" \"steam://rungameid/250900\"\n"));
    }
}
