use super::http;
use crate::achievements::{Achievement, SchemaEntry};
use crate::steam_cdn::community_image;
use serde_json::Value;
use std::collections::HashMap;
use ureq::Agent;

pub fn achievement_schema(agent: &Agent, key: &str, app_id: u32) -> Option<Vec<SchemaEntry>> {
    let app_id = app_id.to_string();
    let response = http::get_json(
        agent,
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/",
        &[("key", key), ("appid", &app_id), ("l", "french")],
        None,
    )?;
    let text =
        |item: &Value, field: &str| item.get(field).and_then(Value::as_str).map(str::to_string);
    let entries = response
        .get("game")?
        .get("availableGameStats")?
        .get("achievements")?
        .as_array()?
        .iter()
        .filter_map(|item| {
            Some(SchemaEntry {
                api_name: text(item, "name")?,
                name: text(item, "displayName")?,
                description: text(item, "description").unwrap_or_default(),
                icon_url: text(item, "icon")
                    .filter(|url| url.starts_with("https://"))
                    .map(|url| community_image(&url)),
            })
        })
        .collect();
    Some(entries)
}

pub fn achievement_rarity(agent: &Agent, app_id: u32) -> Option<HashMap<String, u16>> {
    let app_id = app_id.to_string();
    let response = http::get_json(
        agent,
        "https://api.steampowered.com/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v2/",
        &[("gameid", &app_id)],
        None,
    )?;
    let rarity = response
        .get("achievementpercentages")?
        .get("achievements")?
        .as_array()?
        .iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.to_string();
            let percent = item.get("percent")?;
            let percent = percent
                .as_f64()
                .or_else(|| percent.as_str()?.parse().ok())?;
            let tenths = (percent.clamp(0.0, 100.0) * 10.0).round() as u16;
            Some((name, tenths))
        })
        .collect();
    Some(rarity)
}

pub fn player_achievements(
    agent: &Agent,
    key: &str,
    steam_id: &str,
    app_id: u32,
) -> Option<Vec<Achievement>> {
    let schema = achievement_schema(agent, key, app_id).unwrap_or_default();
    let app_id = app_id.to_string();
    let response = http::get_json(
        agent,
        "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/",
        &[
            ("key", key),
            ("steamid", steam_id),
            ("appid", &app_id),
            ("l", "french"),
        ],
        None,
    )?;
    let list = response
        .get("playerstats")?
        .get("achievements")?
        .as_array()?;
    Some(
        list.iter()
            .filter_map(|item| {
                let api_name = item.get("apiname")?.as_str()?.to_string();
                let entry = schema.iter().find(|entry| entry.api_name == api_name);
                let unlocked = item.get("achieved").and_then(Value::as_u64) == Some(1);
                let text =
                    |field: &str| item.get(field).and_then(Value::as_str).map(str::to_string);
                Some(Achievement {
                    name: text("name")
                        .or_else(|| entry.map(|entry| entry.name.clone()))
                        .unwrap_or_else(|| api_name.clone()),
                    description: text("description")
                        .or_else(|| entry.map(|entry| entry.description.clone()))
                        .unwrap_or_default(),
                    icon_url: entry.and_then(|entry| entry.icon_url.clone()),
                    unlocked,
                    unlocked_at: item
                        .get("unlocktime")
                        .and_then(Value::as_i64)
                        .filter(|time| unlocked && *time > 0),
                    rarity_tenths: None,
                    id: api_name,
                })
            })
            .collect(),
    )
}
