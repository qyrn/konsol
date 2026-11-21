use crate::config::{ConfigStore, KonsolConfig};
use crate::game::Game;
use crate::journal_sessions::JournalSession;
use crate::metadata::{MetadataStore, SteamLibraryCache};
use crate::playtime::PlaytimeStore;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard};

pub struct AppState {
    pub home: PathBuf,
    pub config: Mutex<ConfigStore>,
    pub games: Mutex<HashMap<String, Game>>,
    pub accents: Mutex<HashMap<PathBuf, Option<String>>>,
    pub metadata: Mutex<MetadataStore>,
    pub playtime: Mutex<PlaytimeStore>,
    pub steam_library: SteamLibraryCache,
    pub cache_dir: PathBuf,
    pub running: Mutex<Vec<String>>,
    pub journal: Mutex<Vec<JournalSession>>,
    pub enriching: AtomicBool,
    pub applications_dir: PathBuf,
    pub taskbar_icons_dir: PathBuf,
    pub achievement_icons_dir: PathBuf,
}

pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn update_config(
    state: &AppState,
    change: impl FnOnce(&mut KonsolConfig) -> Result<(), String>,
) -> Result<(), String> {
    let store = lock(&state.config);
    let mut config = store.load()?;
    change(&mut config)?;
    store.save(&config)
}
