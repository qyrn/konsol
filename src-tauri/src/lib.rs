mod achievement_watch;
mod achievements;
mod artwork;
mod binary_vdf;
mod commands;
mod config;
mod desktop_entry;
mod desktop_file;
mod game;
mod launcher;
mod metadata;
mod process;
mod sources;
mod state;
mod vdf;
mod window_watch;

use config::ConfigStore;
use metadata::MetadataStore;
use state::AppState;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let paths = app.path();
            let cache_dir = paths.app_cache_dir()?;
            app.manage(AppState {
                home: paths.home_dir()?,
                config: Mutex::new(ConfigStore::new(
                    paths.app_config_dir()?.join("config.json"),
                )),
                games: Mutex::new(HashMap::new()),
                accents: Mutex::new(HashMap::new()),
                metadata: Mutex::new(MetadataStore::load(&cache_dir)),
                enriching: AtomicBool::new(false),
                applications_dir: paths.data_dir()?.join("applications"),
                taskbar_icons_dir: paths.app_data_dir()?.join("icons"),
                achievement_icons_dir: cache_dir.join("achievement-icons"),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::list_games,
            commands::library::launch_game,
            commands::library::set_game_hidden,
            commands::library::rename_game,
            commands::library::running_game_ids,
            commands::manual::add_manual_game,
            commands::manual::remove_manual_game,
            commands::achievements::get_achievements,
            commands::settings::get_settings,
            commands::settings::save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
