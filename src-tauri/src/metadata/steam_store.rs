use super::http;
use serde_json::{Value, json};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use ureq::Agent;

const STORE_CALL_SPACING: Duration = Duration::from_millis(1500);
const SOFTWARE_GENRE_IDS: std::ops::RangeInclusive<u32> = 50..=60;
const LABEL_GENRE_IDS: [u32; 7] = [23, 37, 70, 71, 72, 73, 74];
const CASUAL_GENRE_ID: u32 = 4;
const INDIE_GENRE_ID: u32 = 23;

static LAST_STORE_CALL: Mutex<Option<Instant>> = Mutex::new(None);

fn wait_for_store_turn() {
    let mut last_call = LAST_STORE_CALL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(previous) = *last_call {
        let elapsed = previous.elapsed();
        if elapsed < STORE_CALL_SPACING {
            thread::sleep(STORE_CALL_SPACING - elapsed);
        }
    }
    *last_call = Some(Instant::now());
}

const ASSET_BASE_URL: &str = "https://shared.akamai.steamstatic.com/store_item_assets/";
const COMMUNITY_ICON_BASE_URL: &str =
    "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps";

pub struct StoreArt {
    pub cover: Option<String>,
    pub hero: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoreDetails {
    pub developer: Option<String>,
    pub year: Option<u16>,
    pub genres: Vec<String>,
    pub is_software: bool,
}

fn game_genres(genres: &[(u32, String)]) -> Vec<String> {
    let mut chosen: Vec<String> = genres
        .iter()
        .filter(|(id, _)| {
            !LABEL_GENRE_IDS.contains(id)
                && !SOFTWARE_GENRE_IDS.contains(id)
                && *id != CASUAL_GENRE_ID
        })
        .map(|(_, name)| name.clone())
        .collect();
    for fallback in [CASUAL_GENRE_ID, INDIE_GENRE_ID] {
        if chosen.is_empty()
            && let Some((_, name)) = genres.iter().find(|(id, _)| *id == fallback)
        {
            chosen.push(name.clone());
        }
    }
    chosen
}

pub fn details(agent: &Agent, app_id: u32) -> Option<StoreDetails> {
    wait_for_store_turn();
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
    let Some(data) = response.get(&app_id)?.get("data") else {
        return Some(StoreDetails {
            developer: None,
            year: None,
            genres: Vec::new(),
            is_software: false,
        });
    };
    let genres: Vec<(u32, String)> = data
        .get("genres")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|genre| {
            let id = genre.get("id")?.as_str()?.parse().ok()?;
            Some((id, genre.get("description")?.as_str()?.to_string()))
        })
        .collect();
    Some(StoreDetails {
        developer: data
            .get("developers")
            .and_then(|developers| developers.get(0))
            .and_then(Value::as_str)
            .map(str::to_string),
        is_software: genres.iter().any(|(id, _)| SOFTWARE_GENRE_IDS.contains(id)),
        genres: game_genres(&genres),
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
    wait_for_store_turn();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_real_genres_before_labels_and_casual() {
        let genres = |list: &[(u32, &str)]| {
            game_genres(
                &list
                    .iter()
                    .map(|(id, name)| (*id, name.to_string()))
                    .collect::<Vec<_>>(),
            )
        };
        assert_eq!(
            genres(&[(23, "Indépendant"), (1, "Action"), (25, "Aventure")]),
            ["Action", "Aventure"]
        );
        assert_eq!(
            genres(&[(4, "Occasionnel"), (23, "Indépendant")]),
            ["Occasionnel"]
        );
        assert_eq!(genres(&[(23, "Indépendant")]), ["Indépendant"]);
        assert!(genres(&[(57, "Utilitaires")]).is_empty());
    }

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
