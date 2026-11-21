use crate::game::{Game, KONSOL_GAME_ID_VARIABLE, LaunchTarget};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

pub fn launch(game: &Game) -> Result<(), String> {
    let arguments = if game.installed {
        command_line(&game.launch)
    } else {
        match &game.install_command {
            Some(command) => command.clone(),
            None => install_command_line(&game.launch)?,
        }
    };
    spawn(game, arguments)
}

pub fn uninstall(game: &Game) -> Result<(), String> {
    let arguments = match &game.launch {
        LaunchTarget::SteamApp(app_id) => {
            vec!["steam".into(), format!("steam://uninstall/{app_id}")]
        }
        LaunchTarget::Heroic { .. } => vec!["heroic".into()],
        LaunchTarget::Program(_) | LaunchTarget::Command { .. } => {
            return Err(
                "Konsol sait pas désinstaller ce jeu, supprime son dossier à la main.".to_string(),
            );
        }
    };
    spawn(game, arguments)
}

fn spawn(game: &Game, arguments: Vec<String>) -> Result<(), String> {
    let (program, rest) = arguments
        .split_first()
        .ok_or("Ce jeu n'a pas de commande de lancement.")?;
    let mut command = Command::new(program);
    command
        .args(rest)
        .env(KONSOL_GAME_ID_VARIABLE, &game.id)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    if let Some(directory) = working_directory(&game.launch) {
        command.current_dir(directory);
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("Impossible de lancer le jeu : {error}"))?;
    thread::spawn(move || child.wait());
    Ok(())
}

pub fn command_line(target: &LaunchTarget) -> Vec<String> {
    match target {
        LaunchTarget::SteamApp(app_id) => vec![
            "steam".into(),
            "-silent".into(),
            format!("steam://rungameid/{app_id}"),
        ],
        LaunchTarget::Heroic { runner, app_name } => vec![
            "heroic".into(),
            "--no-gui".into(),
            format!("heroic://launch?appName={app_name}&runner={runner}"),
        ],
        LaunchTarget::Program(path) => vec![path.to_string_lossy().into_owned()],
        LaunchTarget::Command { arguments, .. } => arguments.clone(),
    }
}

fn install_command_line(target: &LaunchTarget) -> Result<Vec<String>, String> {
    match target {
        LaunchTarget::SteamApp(app_id) => {
            Ok(vec!["steam".into(), format!("steam://install/{app_id}")])
        }
        LaunchTarget::Heroic { runner, app_name } => Ok(vec![
            "heroic".into(),
            format!("heroic://launch?appName={app_name}&runner={runner}"),
        ]),
        LaunchTarget::Program(_) | LaunchTarget::Command { .. } => {
            Err("Ce jeu n'est pas installé.".to_string())
        }
    }
}

fn working_directory(target: &LaunchTarget) -> Option<PathBuf> {
    match target {
        LaunchTarget::Program(path) => path.parent().map(Path::to_path_buf),
        LaunchTarget::Command {
            working_directory, ..
        } => working_directory
            .clone()
            .filter(|directory| directory.is_dir()),
        LaunchTarget::SteamApp(_) | LaunchTarget::Heroic { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_store_launchers_out_of_sight() {
        assert_eq!(
            command_line(&LaunchTarget::SteamApp("250900".into())),
            ["steam", "-silent", "steam://rungameid/250900"]
        );
        assert_eq!(
            command_line(&LaunchTarget::Heroic {
                runner: "gog",
                app_name: "1423049311".into()
            })[1],
            "--no-gui"
        );
    }
}
