use super::http;
use crate::achievements::SchemaEntry;
use serde_json::Value;
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
