use super::http;
use crate::steam_cdn::STORE_ASSETS_URL;
use serde_json::{Value, json};
use std::collections::HashMap;
use ureq::Agent;

pub const BATCH_SIZE: usize = 50;
pub const GAME_TYPE: u32 = 0;

const GENERIC_CATEGORIES: [&str; 3] = ["Action", "Adventure", "Casual"];
const CATEGORY_TAGS: [&str; 34] = [
    "Action",
    "Adventure",
    "RPG",
    "Strategy",
    "Simulation",
    "Sports",
    "Racing",
    "FPS",
    "Shooter",
    "Platformer",
    "Survival",
    "Horror",
    "Puzzle",
    "Roguelike",
    "Roguelite",
    "Sandbox",
    "Fighting",
    "Open World",
    "Management",
    "Visual Novel",
    "Card Game",
    "MMORPG",
    "Battle Royale",
    "Rhythm",
    "Point & Click",
    "City Builder",
    "Metroidvania",
    "Souls-like",
    "Party Game",
    "Tower Defense",
    "Stealth",
    "Hack and Slash",
    "Casual",
    "Automobile Sim",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CatalogEntry {
    pub app_type: Option<u32>,
    pub cover: Option<String>,
    pub hero: Option<String>,
    pub capsule: Option<String>,
    pub developer: Option<String>,
    pub year: Option<u16>,
    pub tags: Vec<(u32, u32)>,
}

pub struct TagDirectory {
    pub names: HashMap<u32, String>,
    pub categories: Vec<u32>,
    pub generic: Vec<u32>,
}

impl TagDirectory {
    pub fn category(&self, tags: &[(u32, u32)]) -> Option<String> {
        tags.iter()
            .filter(|(id, _)| self.categories.contains(id))
            .max_by_key(|(id, weight)| {
                if self.generic.contains(id) {
                    weight * 6 / 10
                } else {
                    *weight
                }
            })
            .and_then(|(id, _)| self.names.get(id).cloned())
    }
}

fn tag_list(agent: &Agent, language: &str) -> Option<HashMap<u32, String>> {
    let response = http::get_json(
        agent,
        "https://api.steampowered.com/IStoreService/GetTagList/v1/",
        &[("language", language)],
        None,
    )?;
    Some(
        response
            .get("response")?
            .get("tags")?
            .as_array()?
            .iter()
            .filter_map(|tag| {
                let id = u32::try_from(tag.get("tagid")?.as_u64()?).ok()?;
                Some((id, tag.get("name")?.as_str()?.to_string()))
            })
            .collect(),
    )
}

pub fn tag_directory(agent: &Agent) -> Option<TagDirectory> {
    let french = tag_list(agent, "french")?;
    let english = tag_list(agent, "english")?;
    let ids_named = |names: &[&str]| -> Vec<u32> {
        english
            .iter()
            .filter(|(_, name)| names.contains(&name.as_str()))
            .map(|(id, _)| *id)
            .collect()
    };
    Some(TagDirectory {
        categories: ids_named(&CATEGORY_TAGS),
        generic: ids_named(&GENERIC_CATEGORIES),
        names: french,
    })
}

fn asset_url(assets: &Value, key: &str) -> Option<String> {
    let format = assets.get("asset_url_format")?.as_str()?;
    let file = assets.get(key)?.as_str()?;
    Some(format!(
        "{STORE_ASSETS_URL}/{}",
        format.replace("${FILENAME}", file)
    ))
}

fn release_year(item: &Value) -> Option<u16> {
    let release = item.get("release")?;
    let timestamp = ["steam_release_date", "original_release_date"]
        .iter()
        .filter_map(|key| release.get(key).and_then(Value::as_i64))
        .find(|timestamp| *timestamp > 0)?;
    let days = timestamp.div_euclid(86_400);
    let years_since_1970 = (days as f64 / 365.2425).floor() as i64;
    u16::try_from(1970 + years_since_1970).ok()
}

fn entry_from(item: &Value) -> CatalogEntry {
    let assets = item.get("assets");
    let asset = |keys: &[&str]| {
        assets.and_then(|assets| keys.iter().find_map(|key| asset_url(assets, key)))
    };
    CatalogEntry {
        app_type: item
            .get("type")
            .and_then(Value::as_u64)
            .and_then(|kind| u32::try_from(kind).ok()),
        cover: asset(&["library_capsule", "library_capsule_2x"]),
        hero: asset(&["library_hero", "page_background"]),
        capsule: asset(&["main_capsule", "header"]),
        developer: item
            .get("basic_info")
            .and_then(|info| info.get("developers"))
            .and_then(|developers| developers.get(0))
            .and_then(|developer| developer.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string),
        year: release_year(item),
        tags: item
            .get("tags")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|tag| {
                let id = u32::try_from(tag.get("tagid")?.as_u64()?).ok()?;
                let weight = u32::try_from(tag.get("weight")?.as_u64()?).ok()?;
                Some((id, weight))
            })
            .collect(),
    }
}

pub fn fetch(agent: &Agent, app_ids: &[u32]) -> Option<HashMap<u32, CatalogEntry>> {
    let ids: Vec<Value> = app_ids
        .iter()
        .map(|app_id| json!({ "appid": app_id }))
        .collect();
    let input = json!({
        "ids": ids,
        "context": { "language": "french", "country_code": "FR" },
        "data_request": {
            "include_assets": true,
            "include_tag_count": 20,
            "include_basic_info": true,
            "include_release": true
        }
    })
    .to_string();
    let response = http::get_json(
        agent,
        "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/",
        &[("input_json", &input)],
        None,
    )?;
    let items = response.get("response")?.get("store_items")?.as_array()?;
    let mut entries: HashMap<u32, CatalogEntry> = app_ids
        .iter()
        .map(|app_id| (*app_id, CatalogEntry::default()))
        .collect();
    for item in items {
        if let Some(app_id) = item
            .get("appid")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok())
        {
            entries.insert(app_id, entry_from(item));
        }
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_assets_tags_and_release_from_a_store_item() {
        let item = json!({
            "appid": 3551340,
            "type": 0,
            "assets": {
                "asset_url_format": "steam/apps/3551340/${FILENAME}?t=1",
                "library_capsule": "834e/library_capsule.jpg",
                "library_hero": "beff/library_hero.jpg"
            },
            "tags": [{ "tagid": 599, "weight": 1895 }],
            "basic_info": { "developers": [{ "name": "Sports Interactive" }] },
            "release": { "steam_release_date": 1762272060 }
        });
        let entry = entry_from(&item);
        assert_eq!(entry.app_type, Some(0));
        assert_eq!(
            entry.cover.as_deref(),
            Some(
                "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/3551340/834e/library_capsule.jpg?t=1"
            )
        );
        assert_eq!(entry.developer.as_deref(), Some("Sports Interactive"));
        assert_eq!(entry.year, Some(2025));
        assert_eq!(entry.tags, [(599, 1895)]);
    }

    #[test]
    fn prefers_specific_categories_over_generic_ones() {
        let directory = TagDirectory {
            names: [(19, "Action".to_string()), (1716, "Roguelike".to_string())].into(),
            categories: vec![19, 1716],
            generic: vec![19],
        };
        assert_eq!(
            directory.category(&[(19, 1000), (1716, 800)]).as_deref(),
            Some("Roguelike")
        );
        assert_eq!(
            directory.category(&[(19, 1000), (1716, 400)]).as_deref(),
            Some("Action")
        );
        assert_eq!(directory.category(&[]), None);
    }
}
