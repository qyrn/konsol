use crate::commands::library::LIBRARY_EVENT;
use crate::sources;
use crate::state::{AppState, lock};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter, Manager};

const CHECK_EVERY: Duration = Duration::from_secs(10);

type Fingerprint = BTreeMap<PathBuf, Option<SystemTime>>;

fn fingerprint(app: &AppHandle) -> Fingerprint {
    let state = app.state::<AppState>();
    let Ok(config) = lock(&state.config).load() else {
        return Fingerprint::new();
    };
    sources::watched_paths(&state.home, &config)
        .into_iter()
        .map(|path| {
            let modified = fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .ok();
            (path, modified)
        })
        .collect()
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut previous = fingerprint(&app);
        loop {
            thread::sleep(CHECK_EVERY);
            let current = fingerprint(&app);
            if current != previous {
                let _ = app.emit(LIBRARY_EVENT, ());
                previous = current;
            }
        }
    });
}
