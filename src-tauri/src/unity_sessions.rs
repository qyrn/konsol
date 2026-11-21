use crate::playtime::PlaySession;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const LOG_FILES: [&str; 2] = ["Player.log", "Player-prev.log"];
const MIN_SECONDS: i64 = 60;
const MAX_SECONDS: i64 = 24 * 3600;

pub struct UnityGame {
    pub id: String,
    pub title: String,
    pub wine_prefix: Option<PathBuf>,
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn subdirectories(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default()
}

fn product_folders(company_root: &Path) -> Vec<PathBuf> {
    subdirectories(company_root)
        .iter()
        .flat_map(|company| subdirectories(company))
        .filter(|product| LOG_FILES.iter().any(|log| product.join(log).is_file()))
        .collect()
}

fn prefix_product_folders(prefix: &Path) -> Vec<PathBuf> {
    ["drive_c/users", "pfx/drive_c/users"]
        .iter()
        .flat_map(|users| subdirectories(&prefix.join(users)))
        .flat_map(|user| product_folders(&user.join("AppData/LocalLow")))
        .collect()
}

fn matches_title(product: &Path, title: &str) -> bool {
    let product = product
        .file_name()
        .map(|name| compact(&name.to_string_lossy()))
        .unwrap_or_default();
    let title = compact(title);
    !product.is_empty()
        && !title.is_empty()
        && (product.contains(&title) || title.contains(&product))
}

fn session_of(game_id: &str, log: &Path) -> Option<PlaySession> {
    let metadata = fs::metadata(log).ok()?;
    let seconds_since_epoch = |time: std::time::SystemTime| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| i64::try_from(duration.as_secs()).ok())
    };
    let started_at = seconds_since_epoch(metadata.created().ok()?)?;
    let ended_at = seconds_since_epoch(metadata.modified().ok()?)?;
    let length = ended_at - started_at;
    (MIN_SECONDS..=MAX_SECONDS)
        .contains(&length)
        .then(|| PlaySession {
            id: format!("unity:{game_id}#{started_at}"),
            game_id: game_id.to_string(),
            seconds: length.unsigned_abs(),
            ended_at,
        })
}

pub fn read(home: &Path, games: &[UnityGame]) -> Vec<PlaySession> {
    let native_products = product_folders(&home.join(".config/unity3d"));
    games
        .iter()
        .flat_map(|game| {
            let products = match &game.wine_prefix {
                Some(prefix) => {
                    let found = prefix_product_folders(prefix);
                    if found.len() == 1 {
                        found
                    } else {
                        found
                            .into_iter()
                            .filter(|product| matches_title(product, &game.title))
                            .collect()
                    }
                }
                None => native_products
                    .iter()
                    .filter(|product| matches_title(product, &game.title))
                    .cloned()
                    .collect(),
            };
            products
                .into_iter()
                .flat_map(|product| LOG_FILES.map(|log| product.join(log)))
                .filter_map(|log| session_of(&game.id, &log))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_unity_log_of_a_prefixed_game() {
        let home = std::env::temp_dir().join(format!("konsol-unity-{}", std::process::id()));
        let prefix = home.join("prefixes/sss");
        let product = prefix.join(
            "drive_c/users/steamuser/AppData/LocalLow/Playstrom/Shipping Store Simulator 2037",
        );
        fs::create_dir_all(&product).unwrap();
        fs::write(product.join("Player.log"), b"log").unwrap();
        let games = [UnityGame {
            id: "desktop:sss".into(),
            title: "SSS".into(),
            wine_prefix: Some(prefix),
        }];
        let products = prefix_product_folders(games[0].wine_prefix.as_deref().unwrap());
        assert_eq!(products, [product]);
        assert!(
            read(&home, &games).is_empty(),
            "a log written just now is too short to be a session"
        );
        assert!(matches_title(
            Path::new("/x/Shipping Store Simulator 2037"),
            "Shipping Store Simulator 2037"
        ));
        fs::remove_dir_all(home).unwrap();
    }
}
