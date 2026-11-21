use serde_json::Value;
use std::process::Command;

pub struct JournalSession {
    pub id: String,
    pub desktop_stem: String,
    pub seconds: u64,
    pub ended_at: i64,
}

pub fn read() -> Vec<JournalSession> {
    let Ok(output) = Command::new("journalctl")
        .args([
            "--user",
            "--no-pager",
            "-o",
            "json",
            "--grep",
            "wall clock time",
        ])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|entry| {
            let message = entry.get("MESSAGE")?.as_str()?;
            let microseconds: i64 = entry.get("__REALTIME_TIMESTAMP")?.as_str()?.parse().ok()?;
            session_from(message, microseconds / 1_000_000)
        })
        .collect()
}

fn session_from(message: &str, ended_at: i64) -> Option<JournalSession> {
    let (unit, rest) = message.split_once(": ")?;
    let duration = rest
        .split_once(" over ")?
        .1
        .split_once(" wall clock time")?
        .0;
    Some(JournalSession {
        id: format!("{unit}#{ended_at}"),
        desktop_stem: desktop_stem(unit)?,
        seconds: parse_duration(duration)?,
        ended_at,
    })
}

fn desktop_stem(unit: &str) -> Option<String> {
    let name = unit.strip_prefix("app-")?;
    let escaped = match name.split_once('@') {
        Some((stem, _)) => stem,
        None => {
            let scope = name.strip_suffix(".scope")?;
            let (stem, pid) = scope.rsplit_once('-')?;
            pid.chars()
                .all(|character| character.is_ascii_digit())
                .then_some(stem)?
        }
    };
    Some(unescape(escaped))
}

fn unescape(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(index) = rest.find("\\x") {
        result.push_str(&rest[..index]);
        let code = rest
            .get(index + 2..index + 4)
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match code {
            Some(byte) => {
                result.push(char::from(byte));
                rest = &rest[index + 4..];
            }
            None => {
                result.push_str("\\x");
                rest = &rest[index + 2..];
            }
        }
    }
    result.push_str(rest);
    result
}

fn parse_duration(text: &str) -> Option<u64> {
    let mut seconds = 0.0;
    for part in text.split_whitespace() {
        let split = part.find(|character: char| character.is_ascii_alphabetic())?;
        let (amount, unit) = part.split_at(split);
        let amount: f64 = amount.parse().ok()?;
        seconds += amount
            * match unit {
                "ms" => 0.001,
                "s" => 1.0,
                "min" => 60.0,
                "h" => 3600.0,
                "d" => 86_400.0,
                _ => return None,
            };
    }
    Some(seconds.round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sessions_started_from_the_menu() {
        let session = session_from(
            "app-skate3recomp@3ca4b601b5974f3299bc66a2f86ffcd7.service: Consumed 5h 28min 4.846s CPU time over 1h 23min 2.372s wall clock time, 1.7G memory peak.",
            1_795_000_000,
        )
        .expect("a session");
        assert_eq!(session.desktop_stem, "skate3recomp");
        assert_eq!(session.seconds, 4982);
        assert_eq!(session.ended_at, 1_795_000_000);
    }

    #[test]
    fn unescapes_scope_units() {
        let session = session_from(
            "app-claude\\x2ddesktop-301131.scope: Consumed 1min CPU time over 44.072s wall clock time.",
            1,
        )
        .expect("a session");
        assert_eq!(session.desktop_stem, "claude-desktop");
        assert_eq!(session.seconds, 44);
        assert!(
            session_from(
                "dbus.service: Consumed 1s CPU time over 2s wall clock time.",
                1
            )
            .is_none()
        );
    }
}
