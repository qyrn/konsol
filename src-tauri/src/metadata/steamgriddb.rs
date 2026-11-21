use super::http;
use serde_json::Value;
use ureq::Agent;

const API_BASE_URL: &str = "https://www.steamgriddb.com/api/v2";

fn first_url(agent: &Agent, key: &str, path: &str, query: &[(&str, &str)]) -> Option<String> {
    let response = http::get_json(agent, &format!("{API_BASE_URL}{path}"), query, Some(key))?;
    response
        .get("data")?
        .as_array()?
        .iter()
        .filter_map(|item| item.get("url").and_then(Value::as_str))
        .find(|url| url.starts_with("https://"))
        .map(str::to_string)
}

pub fn find_game(agent: &Agent, key: &str, title: &str) -> Option<u64> {
    let path = format!(
        "{API_BASE_URL}/search/autocomplete/{}",
        http::encode_path_segment(title)
    );
    http::get_json(agent, &path, &[], Some(key))?
        .get("data")?
        .get(0)?
        .get("id")?
        .as_u64()
}

pub fn grid(agent: &Agent, key: &str, game_id: u64) -> Option<String> {
    first_url(
        agent,
        key,
        &format!("/grids/game/{game_id}"),
        &[
            ("dimensions", "600x900"),
            ("types", "static"),
            ("mimes", "image/jpeg,image/png"),
        ],
    )
}

pub fn hero(agent: &Agent, key: &str, game_id: u64) -> Option<String> {
    first_url(
        agent,
        key,
        &format!("/heroes/game/{game_id}"),
        &[("types", "static"), ("mimes", "image/jpeg,image/png")],
    )
}
