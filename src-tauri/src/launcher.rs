use crate::game::{Game, KONSOL_GAME_ID_VARIABLE, LaunchTarget};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;

pub fn launch(game: &Game) -> Result<(), String> {
    let mut command = build_command(&game.launch);
    command
        .env(KONSOL_GAME_ID_VARIABLE, &game.id)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let mut child = command
        .spawn()
        .map_err(|error| format!("Impossible de lancer le jeu : {error}"))?;
    thread::spawn(move || child.wait());
    Ok(())
}

fn build_command(target: &LaunchTarget) -> Command {
    match target {
        LaunchTarget::SteamApp(app_id) => {
            let mut command = Command::new("steam");
            command.arg(format!("steam://rungameid/{app_id}"));
            command
        }
        LaunchTarget::Heroic { runner, app_name } => {
            let mut command = Command::new("heroic");
            command.arg(format!(
                "heroic://launch?appName={app_name}&runner={runner}"
            ));
            command
        }
        LaunchTarget::Program(path) => {
            let mut command = Command::new(path);
            if let Some(directory) = path.parent() {
                command.current_dir(directory);
            }
            command
        }
    }
}
