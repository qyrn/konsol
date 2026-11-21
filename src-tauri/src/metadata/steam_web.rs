use super::http;
use crate::achievements::SchemaEntry;
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
                icon_url: text(item, "icon").filter(|url| url.starts_with("https://")),
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
