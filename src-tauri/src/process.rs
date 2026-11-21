use crate::game::ProcessMarker;
use std::fs;
use std::process::Command;

struct ProcessSnapshot {
    environment: Vec<u8>,
    command_line: Vec<u8>,
}

fn read_process(pid: u32) -> ProcessSnapshot {
    ProcessSnapshot {
        environment: fs::read(format!("/proc/{pid}/environ")).unwrap_or_default(),
        command_line: fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default(),
    }
}

fn running_pids() -> Vec<u32> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    let own_pid = std::process::id();
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
        .filter(|pid| *pid != own_pid)
        .collect()
}

fn snapshot_matches(snapshot: &ProcessSnapshot, marker: &ProcessMarker) -> bool {
    match marker {
        ProcessMarker::EnvVar { name, value } => {
            let expected = format!("{name}={value}");
            snapshot
                .environment
                .split(|byte| *byte == 0)
                .any(|entry| entry == expected.as_bytes())
        }
        ProcessMarker::Mentions(text) => {
            contains(&snapshot.environment, text.as_bytes())
                || contains(&snapshot.command_line, text.as_bytes())
        }
        ProcessMarker::AnyOf(markers) => markers
            .iter()
            .any(|marker| snapshot_matches(snapshot, marker)),
    }
}

pub fn process_matches(pid: u32, marker: &ProcessMarker) -> bool {
    snapshot_matches(&read_process(pid), marker)
}

pub fn any_process_matches(marker: &ProcessMarker) -> bool {
    running_pids()
        .into_iter()
        .any(|pid| process_matches(pid, marker))
}

pub fn stop_matching(marker: &ProcessMarker) -> Result<(), String> {
    let pids: Vec<String> = running_pids()
        .into_iter()
        .filter(|pid| process_matches(*pid, marker))
        .map(|pid| pid.to_string())
        .collect();
    if pids.is_empty() {
        return Err("Le jeu ne tourne plus.".to_string());
    }
    let status = Command::new("kill")
        .arg("-TERM")
        .args(&pids)
        .status()
        .map_err(|error| format!("Impossible d'arrêter le jeu : {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("Le jeu n'a pas voulu s'arrêter.".to_string())
    }
}

pub fn running_keys<K>(markers: &[(K, ProcessMarker)]) -> Vec<&K> {
    let mut found = vec![false; markers.len()];
    for pid in running_pids() {
        if found.iter().all(|matched| *matched) {
            break;
        }
        let snapshot = read_process(pid);
        for (index, (_, marker)) in markers.iter().enumerate() {
            if !found[index] && snapshot_matches(&snapshot, marker) {
                found[index] = true;
            }
        }
    }
    markers
        .iter()
        .zip(found)
        .filter_map(|((key, _), matched)| matched.then_some(key))
        .collect()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_running_markers_and_skips_missing_ones() {
        let parent = std::os::unix::process::parent_id();
        let parent_path = String::from_utf8_lossy(&read_process(parent).environment)
            .split('\0')
            .find_map(|entry| entry.strip_prefix("PATH=").map(str::to_string))
            .expect("parent has PATH");
        let markers = [
            (
                "parent",
                ProcessMarker::EnvVar {
                    name: "PATH",
                    value: parent_path,
                },
            ),
            (
                "ghost",
                ProcessMarker::EnvVar {
                    name: "KONSOL_GAME_ID",
                    value: "nope".into(),
                },
            ),
        ];
        assert_eq!(running_keys(&markers), [&"parent"]);
    }

    #[test]
    fn matches_the_current_process_by_environment() {
        let pid = std::process::id();
        let path = std::env::var("PATH").expect("PATH is set");
        assert!(process_matches(
            pid,
            &ProcessMarker::EnvVar {
                name: "PATH",
                value: path
            }
        ));
        assert!(!process_matches(
            pid,
            &ProcessMarker::EnvVar {
                name: "PATH",
                value: "nope".into()
            }
        ));
        assert!(!process_matches(
            pid,
            &ProcessMarker::Mentions(String::new())
        ));
    }
}
