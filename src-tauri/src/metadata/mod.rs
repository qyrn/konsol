mod disk_usage;
pub mod http;
mod steam_store;
mod steam_web;
mod steamgriddb;

use crate::achievements::SchemaEntry;
use crate::desktop_entry::file_stem;
use crate::game::{AchievementSource, Game, GameSource, ImageSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameMetadata {
    pub steam_app_id: Option<u32>,
    pub is_software: Option<bool>,
    pub cover: Option<PathBuf>,
    pub hero: Option<PathBuf>,
    pub icon: Option<PathBuf>,
    pub achievement_schema: Option<Vec<SchemaEntry>>,
    pub developer: Option<String>,
    pub year: Option<u16>,
    pub genre: Option<String>,
    pub size_bytes: Option<u64>,
    pub achievement_rarity: Option<HashMap<String, u16>>,
    pub store_lookup_done: bool,
    pub grid_lookup_done: bool,
    pub schema_lookup_done: bool,
    pub details_lookup_done: bool,
    pub size_lookup_done: bool,
    pub rarity_lookup_done: bool,
}

impl GameMetadata {
    fn store_app_id(&self, game: &Game) -> Option<u32> {
        if game.source == GameSource::Steam {
            return game.id.trim_start_matches("steam:").parse().ok();
        }
        self.steam_app_id
    }
}

#[derive(Clone, Default)]
pub struct ApiKeys {
    pub steam_grid_db: Option<String>,
    pub steam_web: Option<String>,
}

pub struct MetadataStore {
    path: PathBuf,
    images_dir: PathBuf,
    entries: HashMap<String, GameMetadata>,
}

impl MetadataStore {
    pub fn load(cache_dir: &Path) -> Self {
        let path = cache_dir.join("metadata.json");
        let entries = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();
        Self {
            path,
            images_dir: cache_dir.join("images"),
            entries,
        }
    }

    pub fn get(&self, game_id: &str) -> Option<&GameMetadata> {
        self.entries.get(game_id)
    }

    fn save(&self) {
        if let Some(directory) = self.path.parent()
            && fs::create_dir_all(directory).is_ok()
            && let Ok(content) = serde_json::to_string(&self.entries)
        {
            let _ = fs::write(&self.path, content);
        }
    }

    pub fn achievement_rarity(&self, game_id: &str) -> Option<&HashMap<String, u16>> {
        self.get(game_id)?.achievement_rarity.as_ref()
    }

    pub fn emulator_schema(&self, game_id: &str) -> Option<&[SchemaEntry]> {
        self.get(game_id)?.achievement_schema.as_deref()
    }

    pub fn snapshot(&self) -> HashMap<String, GameMetadata> {
        self.entries.clone()
    }

    pub fn images_dir(&self) -> &Path {
        &self.images_dir
    }

    pub fn merge(&mut self, updates: HashMap<String, GameMetadata>) -> bool {
        let changed = updates.iter().any(|(game_id, metadata)| {
            self.entries.get(game_id).cloned().unwrap_or_default() != *metadata
        });
        self.entries.extend(updates);
        if changed {
            self.save();
        }
        changed
    }

    pub fn forget(&mut self, game_id: &str) {
        if self.entries.remove(game_id).is_some() {
            self.save();
        }
    }

    pub fn reset_key_lookups(&mut self) {
        for metadata in self.entries.values_mut() {
            metadata.grid_lookup_done = false;
            metadata.schema_lookup_done = false;
        }
        self.save();
    }
}

pub fn apply(games: Vec<Game>, store: &MetadataStore) -> Vec<Game> {
    games
        .into_iter()
        .filter(|game| {
            store
                .get(&game.id)
                .and_then(|metadata| metadata.is_software)
                != Some(true)
        })
        .map(|mut game| {
            let Some(metadata) = store.get(&game.id) else {
                return game;
            };
            if game.needs_store_artwork() && game.cover.is_none() {
                let existing = |path: &Option<PathBuf>| path.clone().filter(|path| path.is_file());
                if let Some(cover) = existing(&metadata.cover) {
                    game.cover = Some(ImageSource::File(cover.to_string_lossy().into_owned()));
                }
                if let Some(hero) = existing(&metadata.hero) {
                    game.hero = Some(ImageSource::File(hero.to_string_lossy().into_owned()));
                }
                game.icon_source = existing(&metadata.icon).or_else(|| existing(&metadata.cover));
            }
            let details = &mut game.details;
            details.developer = details.developer.take().or(metadata.developer.clone());
            details.genre = metadata.genre.clone().or(details.genre.take());
            details.year = details.year.or(metadata.year);
            details.size_bytes = details.size_bytes.or(metadata.size_bytes);
            game
        })
        .collect()
}

pub fn needs_enrichment(games: &[Game], store: &MetadataStore, keys: &ApiKeys) -> bool {
    games.iter().any(|game| {
        let metadata = store.get(&game.id).cloned().unwrap_or_default();
        pending_steps(game, &metadata, keys).any_pending()
    })
}

struct PendingSteps {
    software_check: bool,
    store_lookup: bool,
    grid_lookup: bool,
    schema_lookup: bool,
    details_lookup: bool,
    size_lookup: bool,
    rarity_lookup: bool,
}

impl PendingSteps {
    fn any_pending(&self) -> bool {
        self.software_check
            || self.store_lookup
            || self.grid_lookup
            || self.schema_lookup
            || self.details_lookup
            || self.size_lookup
            || self.rarity_lookup
    }
}

fn pending_steps(game: &Game, metadata: &GameMetadata, keys: &ApiKeys) -> PendingSteps {
    let artwork_game = game.needs_store_artwork() && game.cover.is_none();
    let has_emulator_stats = matches!(
        game.achievement_source,
        Some(AchievementSource::EmulatorStats { .. })
    );
    let has_steam_achievements = has_emulator_stats
        || matches!(
            game.achievement_source,
            Some(AchievementSource::Steam { .. })
        );
    let app_id_known = metadata.store_app_id(game).is_some();
    let is_game = metadata.is_software != Some(true);
    PendingSteps {
        software_check: game.source == GameSource::Steam && metadata.is_software.is_none(),
        store_lookup: game.source != GameSource::Steam && !metadata.store_lookup_done,
        grid_lookup: artwork_game
            && keys.steam_grid_db.is_some()
            && metadata.cover.is_none()
            && !metadata.grid_lookup_done,
        schema_lookup: has_emulator_stats
            && keys.steam_web.is_some()
            && !metadata.schema_lookup_done
            && (metadata.steam_app_id.is_some() || !metadata.store_lookup_done),
        details_lookup: is_game
            && !metadata.details_lookup_done
            && (app_id_known || (game.source != GameSource::Steam && !metadata.store_lookup_done)),
        size_lookup: game.details.size_bytes.is_none()
            && game.details.location.is_some()
            && !metadata.size_lookup_done,
        rarity_lookup: is_game
            && has_steam_achievements
            && !metadata.rarity_lookup_done
            && (app_id_known || !metadata.store_lookup_done),
    }
}

pub fn enrich(
    games: &[Game],
    snapshot: HashMap<String, GameMetadata>,
    keys: &ApiKeys,
    images_dir: &Path,
    home: &Path,
) -> HashMap<String, GameMetadata> {
    let agent = http::agent();
    let mut updates = HashMap::new();
    for game in games {
        let mut metadata = snapshot.get(&game.id).cloned().unwrap_or_default();
        if !pending_steps(game, &metadata, keys).any_pending() {
            continue;
        }
        enrich_game(&agent, game, &mut metadata, keys, images_dir);
        if pending_steps(game, &metadata, keys).size_lookup {
            metadata.size_bytes = game
                .details
                .location
                .as_deref()
                .and_then(|location| disk_usage::game_folder_size(home, Path::new(location)));
            metadata.size_lookup_done = true;
        }
        updates.insert(game.id.clone(), metadata);
    }
    updates
}

fn enrich_game(
    agent: &ureq::Agent,
    game: &Game,
    metadata: &mut GameMetadata,
    keys: &ApiKeys,
    images_dir: &Path,
) {
    let steps = pending_steps(game, metadata, keys);
    if steps.software_check {
        let app_id = game.id.trim_start_matches("steam:");
        metadata.is_software = steam_store::is_software(agent, app_id);
    }
    let stem = file_stem(&game.id);
    let image_path = |kind: &str| images_dir.join(format!("{stem}-{kind}"));
    let fetch = |url: Option<String>, kind: &str| {
        let destination = image_path(kind);
        url.filter(|url| http::download(agent, url, &destination))
            .map(|_| destination)
    };
    if steps.store_lookup {
        metadata.steam_app_id = steam_store::find_app_id(agent, &game.title);
        if let Some(art) = metadata
            .steam_app_id
            .filter(|_| game.needs_store_artwork() && game.cover.is_none())
            .and_then(|app_id| steam_store::art(agent, app_id))
        {
            metadata.cover = fetch(art.cover, "cover");
            metadata.hero = fetch(art.hero, "hero");
            metadata.icon = fetch(art.icon, "icon");
        }
        metadata.store_lookup_done = true;
    }
    if let Some(key) = keys.steam_grid_db.as_deref()
        && pending_steps(game, metadata, keys).grid_lookup
    {
        if let Some(grid_id) = steamgriddb::find_game(agent, key, &game.title) {
            metadata.cover = fetch(steamgriddb::grid(agent, key, grid_id), "cover");
            if metadata.hero.is_none() {
                metadata.hero = fetch(steamgriddb::hero(agent, key, grid_id), "hero");
            }
        }
        metadata.grid_lookup_done = true;
    }
    if let (Some(key), Some(app_id)) = (keys.steam_web.as_deref(), metadata.steam_app_id)
        && pending_steps(game, metadata, keys).schema_lookup
    {
        metadata.achievement_schema = steam_web::achievement_schema(agent, key, app_id);
        metadata.schema_lookup_done = true;
    }
    let app_id = metadata.store_app_id(game);
    if pending_steps(game, metadata, keys).details_lookup {
        if let Some(details) = app_id.and_then(|app_id| steam_store::details(agent, app_id)) {
            metadata.developer = details.developer;
            metadata.year = details.year;
            metadata.genre = details.genre;
        }
        metadata.details_lookup_done = true;
    }
    if pending_steps(game, metadata, keys).rarity_lookup {
        metadata.achievement_rarity =
            app_id.and_then(|app_id| steam_web::achievement_rarity(agent, app_id));
        metadata.rarity_lookup_done = true;
    }
}
