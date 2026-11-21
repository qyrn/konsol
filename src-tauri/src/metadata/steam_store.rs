use super::http;
use serde_json::{Value, json};
use ureq::Agent;

const ASSET_BASE_URL: &str = "https://shared.akamai.steamstatic.com/store_item_assets/";
const COMMUNITY_ICON_BASE_URL: &str =
    "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps";
const SOFTWARE_GENRES: [&str; 11] = [
    "Utilities",
    "Design & Illustration",
    "Animation & Modeling",
    "Photo Editing",
    "Video Production",
    "Audio Production",
    "Software Training",
    "Web Publishing",
    "Education",
    "Game Development",
    "Accounting",
];

pub struct StoreArt {
    pub cover: Option<String>,
    pub hero: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoreDetails {
    pub developer: Option<String>,
    pub year: Option<u16>,
    pub genre: Option<String>,
}

pub fn details(agent: &Agent, app_id: u32) -> Option<StoreDetails> {
    let app_id = app_id.to_string();
    let response = http::get_json(
        agent,
        "https://store.steampowered.com/api/appdetails",
        &[
            ("appids", &app_id),
            ("filters", "genres,release_date,developers"),
            ("l", "french"),
        ],
        None,
    )?;
    let data = response.get(&app_id)?.get("data")?;
    let first_text = |list: Option<&Value>, field: Option<&str>| {
        let item = list?.as_array()?.first()?;
        let text = match field {
            Some(field) => item.get(field)?,
            None => item,
        };
        text.as_str().map(str::to_string)
    };
    Some(StoreDetails {
        developer: first_text(data.get("developers"), None),
        genre: first_text(data.get("genres"), Some("description")),
        year: data
            .get("release_date")
            .and_then(|release| release.get("date"))
            .and_then(Value::as_str)
            .and_then(release_year),
    })
}

fn release_year(date: &str) -> Option<u16> {
    date.split(|character: char| !character.is_ascii_digit())
        .filter(|part| part.len() == 4)
        .filter_map(|part| part.parse().ok())
        .next_back()
}

pub fn normalized_title(title: &str) -> String {
    title
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn find_app_id(agent: &Agent, title: &str) -> Option<u32> {
    let response = http::get_json(
        agent,
        "https://store.steampowered.com/api/storesearch/",
        &[("term", title), ("l", "english"), ("cc", "US")],
        None,
    )?;
    let wanted = normalized_title(title);
    response
        .get("items")?
        .as_array()?
        .iter()
        .find(|item| {
            item.get("name")
                .and_then(Value::as_str)
                .map(normalized_title)
                == Some(wanted.clone())
        })?
        .get("id")?
        .as_u64()
        .and_then(|id| u32::try_from(id).ok())
}

pub fn art(agent: &Agent, app_id: u32) -> Option<StoreArt> {
    let input = json!({
        "ids": [{ "appid": app_id }],
        "context": { "language": "english", "country_code": "US" },
        "data_request": { "include_assets": true }
    })
    .to_string();
    let response = http::get_json(
        agent,
        "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/",
        &[("input_json", &input)],
        None,
    )?;
    let assets = response
        .get("response")?
        .get("store_items")?
        .get(0)?
        .get("assets")?;
    let format = assets.get("asset_url_format")?.as_str()?;
    let asset = |key: &str| {
        let file_name = assets.get(key)?.as_str()?;
        Some(format!(
            "{ASSET_BASE_URL}{}",
            format.replace("${FILENAME}", file_name)
        ))
    };
    Some(StoreArt {
        cover: asset("library_capsule"),
        hero: asset("library_hero"),
        icon: assets
            .get("community_icon")
            .and_then(Value::as_str)
            .filter(|hash| hash.chars().all(|character| character.is_ascii_hexdigit()))
            .map(|hash| format!("{COMMUNITY_ICON_BASE_URL}/{app_id}/{hash}.jpg")),
    })
}

pub fn is_software(agent: &Agent, app_id: &str) -> Option<bool> {
    let response = http::get_json(
        agent,
        "https://store.steampowered.com/api/appdetails",
        &[("appids", app_id), ("filters", "genres"), ("l", "english")],
        None,
    )?;
    let data = response.get(app_id)?.get("data")?;
    let genres = data
        .get("genres")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Some(genres.iter().any(|genre| {
        genre
            .get("description")
            .and_then(Value::as_str)
            .is_some_and(|description| SOFTWARE_GENRES.contains(&description))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_year_in_localized_release_dates() {
        assert_eq!(release_year("4 nov. 2014"), Some(2014));
        assert_eq!(release_year("Nov 4, 2014"), Some(2014));
        assert_eq!(release_year("Bientôt"), None);
    }

    #[test]
    fn normalizes_titles_for_exact_matching() {
        assert_eq!(
            normalized_title("The Binding of Isaac: Rebirth"),
            "thebindingofisaacrebirth"
        );
        assert_eq!(
            normalized_title("Shipping Store Simulator 2037"),
            "shippingstoresimulator2037"
        );
    }
}
