use super::http;
use super::steam_catalog::GAME_TYPE;
use crate::game::{Game, TagWeight};
use crate::steam_cdn::STORE_ASSETS_URL;
use crate::time::now_unix;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use ureq::Agent;

const PROFILE_TAGS: usize = 4;
const SEARCH_COUNT: &str = "60";
const MIN_REVIEWS: u64 = 2000;
const MIN_POSITIVE: u64 = 85;
const PLAYABLE_ON_DECK: u64 = 2;
const PLAYED_MINUTES: u64 = 60;
const MAX_RESULTS: usize = 12;
const REFRESH_AFTER_SECONDS: i64 = 24 * 3600;
const IGNORED_TAG_NAMES: [&str; 6] = [
    "Solo",
    "Multijoueur",
    "Indépendant",
    "Excellente bande-son",
    "Atmosphérique",
    "Free-to-play",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discovery {
    pub app_id: u32,
    pub name: String,
    pub capsule: Option<String>,
    pub review_label: Option<String>,
    pub because: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedDiscoveries {
    fetched_at: i64,
    items: Vec<Discovery>,
}

pub struct DiscoveryCache {
    path: PathBuf,
}

type TagVector = HashMap<u32, f64>;

fn vector(tags: &[TagWeight]) -> TagVector {
    let kept: Vec<&TagWeight> = tags
        .iter()
        .filter(|tag| tag.id != 0 && !IGNORED_TAG_NAMES.contains(&tag.name.as_str()))
        .collect();
    let strongest = kept.iter().map(|tag| tag.weight).max().unwrap_or(1).max(1) as f64;
    kept.iter()
        .map(|tag| (tag.id, f64::from(tag.weight) / strongest))
        .collect()
}

fn cosine(first: &TagVector, second: &TagVector) -> f64 {
    let dot: f64 = first
        .iter()
        .map(|(id, weight)| weight * second.get(id).copied().unwrap_or(0.0))
        .sum();
    let norm = |vector: &TagVector| {
        vector
            .values()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt()
    };
    let denominator = norm(first) * norm(second);
    if denominator == 0.0 {
        0.0
    } else {
        dot / denominator
    }
}

fn top_sellers(agent: &Agent, tag_id: u32) -> Vec<u32> {
    let tag = tag_id.to_string();
    http::get_json(
        agent,
        "https://store.steampowered.com/search/results/",
        &[
            ("json", "1"),
            ("tags", &tag),
            ("filter", "topsellers"),
            ("category1", "998"),
            ("count", SEARCH_COUNT),
            ("cc", "FR"),
        ],
        None,
    )
    .and_then(|response| response.get("items")?.as_array().cloned())
    .unwrap_or_default()
    .iter()
    .filter_map(|item| {
        let logo = item.get("logo")?.as_str()?;
        let rest = logo.split("/apps/").nth(1)?;
        rest.split('/').next()?.parse().ok()
    })
    .collect()
}

struct Candidate {
    app_id: u32,
    name: String,
    capsule: Option<String>,
    review_label: Option<String>,
    tags: TagVector,
}

fn store_items(agent: &Agent, app_ids: &[u32]) -> Vec<Candidate> {
    app_ids
        .chunks(50)
        .filter_map(|chunk| {
            let ids: Vec<Value> = chunk
                .iter()
                .map(|app_id| json!({ "appid": app_id }))
                .collect();
            let input = json!({
                "ids": ids,
                "context": { "language": "french", "country_code": "FR" },
                "data_request": {
                    "include_assets": true,
                    "include_tag_count": 20,
                    "include_reviews": true,
                    "include_platforms": true
                }
            })
            .to_string();
            let response = http::get_json(
                agent,
                "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/",
                &[("input_json", &input)],
                None,
            )?;
            response
                .get("response")?
                .get("store_items")?
                .as_array()
                .cloned()
        })
        .flatten()
        .filter_map(|item| candidate(&item))
        .collect()
}

fn candidate(item: &Value) -> Option<Candidate> {
    if item.get("type").and_then(Value::as_u64) != Some(u64::from(GAME_TYPE)) {
        return None;
    }
    let reviews = item.get("reviews")?.get("summary_filtered")?;
    let review_count = reviews.get("review_count")?.as_u64()?;
    let positive = reviews.get("percent_positive")?.as_u64()?;
    let deck = item
        .get("platforms")
        .and_then(|platforms| platforms.get("steam_deck_compat_category"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if review_count < MIN_REVIEWS || positive < MIN_POSITIVE || deck < PLAYABLE_ON_DECK {
        return None;
    }
    let assets = item.get("assets");
    let capsule = assets.and_then(|assets| {
        let format = assets.get("asset_url_format")?.as_str()?;
        let file = assets
            .get("main_capsule")
            .or_else(|| assets.get("header"))?
            .as_str()?;
        Some(format!(
            "{STORE_ASSETS_URL}/{}",
            format.replace("${FILENAME}", file)
        ))
    });
    Some(Candidate {
        app_id: u32::try_from(item.get("appid")?.as_u64()?).ok()?,
        name: item.get("name")?.as_str()?.to_string(),
        capsule,
        review_label: reviews
            .get("review_score_label")
            .and_then(Value::as_str)
            .map(|label| format!("Évaluations {label}")),
        tags: normalized(
            item.get("tags")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|tag| {
                    let id = u32::try_from(tag.get("tagid")?.as_u64()?).ok()?;
                    Some((id, tag.get("weight")?.as_f64()?))
                })
                .collect(),
        ),
    })
}

fn normalized(tags: Vec<(u32, f64)>) -> TagVector {
    let strongest = tags.iter().map(|(_, weight)| *weight).fold(1.0, f64::max);
    tags.into_iter()
        .map(|(id, weight)| (id, weight / strongest))
        .collect()
}

pub fn discover(agent: &Agent, games: &[Game]) -> Vec<Discovery> {
    let played: Vec<(&Game, TagVector, f64)> = games
        .iter()
        .filter(|game| game.details.playtime_minutes.unwrap_or(0) >= PLAYED_MINUTES)
        .map(|game| {
            let hours = game.details.playtime_minutes.unwrap_or(0) as f64 / 60.0;
            (game, vector(&game.details.tags), (1.0 + hours).ln())
        })
        .filter(|(_, tags, _)| !tags.is_empty())
        .collect();
    let mut profile: TagVector = HashMap::new();
    for (_, tags, pull) in &played {
        for (id, weight) in tags {
            *profile.entry(*id).or_default() += weight * pull;
        }
    }
    let mut ranked_tags: Vec<(u32, f64)> = profile.into_iter().collect();
    ranked_tags.sort_by(|first, second| second.1.total_cmp(&first.1));
    let owned: HashSet<u32> = games
        .iter()
        .filter_map(|game| game.id.strip_prefix("steam:")?.parse().ok())
        .collect();
    let mut seen = HashSet::new();
    let app_ids: Vec<u32> = ranked_tags
        .iter()
        .take(PROFILE_TAGS)
        .flat_map(|(tag_id, _)| top_sellers(agent, *tag_id))
        .filter(|app_id| !owned.contains(app_id) && seen.insert(*app_id))
        .collect();
    let mut scored: Vec<(f64, Discovery)> = store_items(agent, &app_ids)
        .into_iter()
        .filter_map(|candidate| {
            let (game, similarity) = played
                .iter()
                .map(|(game, tags, pull)| {
                    (*game, cosine(&candidate.tags, tags) * (1.0 + pull / 10.0))
                })
                .max_by(|first, second| first.1.total_cmp(&second.1))?;
            Some((
                similarity,
                Discovery {
                    app_id: candidate.app_id,
                    name: candidate.name,
                    capsule: candidate.capsule,
                    review_label: candidate.review_label,
                    because: game.title.clone(),
                },
            ))
        })
        .collect();
    scored.sort_by(|first, second| second.0.total_cmp(&first.0));
    scored
        .into_iter()
        .take(MAX_RESULTS)
        .map(|(_, item)| item)
        .collect()
}

impl DiscoveryCache {
    pub fn new(cache_dir: &Path) -> Self {
        Self {
            path: cache_dir.join("discoveries.json"),
        }
    }

    fn read(&self) -> CachedDiscoveries {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn fresh(&self) -> Option<Vec<Discovery>> {
        let cached = self.read();
        (now_unix() - cached.fetched_at < REFRESH_AFTER_SECONDS && !cached.items.is_empty())
            .then_some(cached.items)
    }

    pub fn store(&self, items: &[Discovery]) {
        let cached = CachedDiscoveries {
            fetched_at: now_unix(),
            items: items.to_vec(),
        };
        if let Some(directory) = self.path.parent()
            && fs::create_dir_all(directory).is_ok()
            && let Ok(content) = serde_json::to_string(&cached)
        {
            let _ = fs::write(&self.path, content);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_popular_well_rated_playable_games() {
        let item = |count: u64, positive: u64, deck: u64| {
            json!({
                "appid": 10, "name": "Jeu", "type": 0,
                "reviews": { "summary_filtered": { "review_count": count, "percent_positive": positive, "review_score_label": "très positives" } },
                "platforms": { "steam_deck_compat_category": deck },
                "tags": [{ "tagid": 1716, "weight": 900 }]
            })
        };
        assert!(candidate(&item(50_000, 95, 3)).is_some());
        assert!(candidate(&item(300, 99, 3)).is_none());
        assert!(candidate(&item(50_000, 70, 3)).is_none());
        assert!(candidate(&item(50_000, 95, 1)).is_none());
        assert_eq!(
            candidate(&item(50_000, 95, 3)).and_then(|found| found.review_label),
            Some("Évaluations très positives".to_string())
        );
    }
}
