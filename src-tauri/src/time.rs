use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
        })
}

pub fn iso_timestamp(text: &str) -> Option<i64> {
    let (date, time) = text.split_once('T')?;
    let mut date_parts = date.split('-').map(str::parse::<i64>);
    let year = date_parts.next()?.ok()?;
    let month = date_parts.next()?.ok()?;
    let day = date_parts.next()?.ok()?;
    let time = time.trim_end_matches('Z');
    let time = time.split_once('.').map_or(time, |(whole, _)| whole);
    let mut time_parts = time.split(':').map(str::parse::<i64>);
    let hours = time_parts.next()?.ok()?;
    let minutes = time_parts.next()?.ok()?;
    let seconds = time_parts.next().and_then(Result::ok).unwrap_or(0);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400 + hours * 3_600 + minutes * 60 + seconds)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heroic_timestamps() {
        assert_eq!(iso_timestamp("1970-01-01T00:00:00.000Z"), Some(0));
        assert_eq!(
            iso_timestamp("2026-11-20T16:06:25.868Z"),
            Some(1_795_190_785)
        );
        assert_eq!(iso_timestamp(""), None);
        assert_eq!(iso_timestamp("2026-13-01T00:00:00Z"), None);
    }
}
