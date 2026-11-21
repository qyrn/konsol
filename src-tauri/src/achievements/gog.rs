use super::{Achievement, AchievementList};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn cache_file(home: &Path) -> PathBuf {
    home.join(".config/heroic/store_cache/gog_achievements.json")
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

pub fn read(home: &Path, app_name: &str) -> Option<AchievementList> {
    let content = fs::read_to_string(cache_file(home)).ok()?;
    let cache: Value = serde_json::from_str(&content).ok()?;
    let achievements: Vec<Achievement> = cache
        .get(app_name)?
        .as_array()?
        .iter()
        .filter_map(|item| {
            let unlocked_at = item.get("date_unlocked").and_then(parse_unlock_date);
            let unlocked = !item.get("date_unlocked").is_none_or(Value::is_null);
            let icon_key = if unlocked {
                "image_url_unlocked"
            } else {
                "image_url_locked"
            };
            Some(Achievement {
                id: text(item, "achievement_key")?,
                name: text(item, "name")?,
                description: text(item, "description").unwrap_or_default(),
                icon_url: text(item, icon_key).filter(|url| url.starts_with("https://")),
                unlocked,
                unlocked_at,
            })
        })
        .collect();
    (!achievements.is_empty()).then_some(AchievementList {
        achievements,
        total_known: true,
    })
}

fn parse_unlock_date(value: &Value) -> Option<i64> {
    if let Some(seconds) = value.as_i64() {
        return Some(seconds);
    }
    parse_iso_datetime(value.as_str()?)
}

fn parse_iso_datetime(text: &str) -> Option<i64> {
    let number = |range: std::ops::Range<usize>| text.get(range)?.parse::<i64>().ok();
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    let shifted_year = if month <= 2 { year - 1 } else { year };
    let era = shifted_year.div_euclid(400);
    let year_of_era = shifted_year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_iso_dates_to_unix_seconds() {
        assert_eq!(parse_iso_datetime("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_iso_datetime("2026-11-19T11:13:48+0000"),
            Some(1_795_086_828)
        );
        assert_eq!(parse_iso_datetime("bad"), None);
    }
}
