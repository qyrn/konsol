pub const COMMUNITY_IMAGES_URL: &str =
    "https://shared.akamai.steamstatic.com/community_assets/images/apps";
pub const STORE_ASSETS_URL: &str = "https://shared.akamai.steamstatic.com/store_item_assets";

const RETIRED_COMMUNITY_PATH: &str = "/steamcommunity/public/images/apps/";

pub fn community_image(url: &str) -> String {
    match url.split_once(RETIRED_COMMUNITY_PATH) {
        Some((_, rest)) => format!("{COMMUNITY_IMAGES_URL}/{rest}"),
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_retired_community_urls_to_the_current_cdn() {
        assert_eq!(
            community_image(
                "https://steamcdn-a.akamaihd.net/steamcommunity/public/images/apps/4098030/ff.jpg"
            ),
            "https://shared.akamai.steamstatic.com/community_assets/images/apps/4098030/ff.jpg"
        );
        assert_eq!(
            community_image("https://example.test/a.jpg"),
            "https://example.test/a.jpg"
        );
    }
}
