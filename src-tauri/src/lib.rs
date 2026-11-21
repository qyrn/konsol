mod artwork;
mod commands;
mod config;
mod desktop_entry;
mod game;
mod launcher;
mod sources;
mod vdf;
mod window_watch;

use commands::AppState;
use config::ConfigStore;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let paths = app.path();
            app.manage(AppState {
                home: paths.home_dir()?,
                config: Mutex::new(ConfigStore::new(
                    paths.app_config_dir()?.join("config.json"),
                )),
                games: Mutex::new(HashMap::new()),
                accents: Mutex::new(HashMap::new()),
                applications_dir: paths.data_dir()?.join("applications"),
                icons_dir: paths.app_data_dir()?.join("icons"),
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
