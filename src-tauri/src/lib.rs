mod commands;
mod config;
mod game;
mod launcher;
mod sources;
mod vdf;

use commands::AppState;
use config::ConfigStore;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let home = app.path().home_dir()?;
            let config_path = app.path().app_config_dir()?.join("config.json");
            app.manage(AppState {
                home,
                config: Mutex::new(ConfigStore::new(config_path)),
                games: Mutex::new(HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::launch_game,
            commands::set_game_hidden,
            commands::add_manual_game,
            commands::remove_manual_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
