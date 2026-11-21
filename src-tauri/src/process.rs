use crate::game::ProcessMarker;
use std::fs;

pub fn any_process_matches(marker: &ProcessMarker) -> bool {
    let Ok(entries) = fs::read_dir("/proc") else {
        return false;
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
        .any(|pid| process_matches(pid, marker))
}

pub fn process_matches(pid: u32, marker: &ProcessMarker) -> bool {
    let environment = fs::read(format!("/proc/{pid}/environ")).unwrap_or_default();
    match marker {
        ProcessMarker::EnvVar { name, value } => {
            let expected = format!("{name}={value}");
            environment
                .split(|byte| *byte == 0)
                .any(|entry| entry == expected.as_bytes())
        }
        ProcessMarker::Mentions(text) => {
            let command_line = fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
            contains(&environment, text.as_bytes()) || contains(&command_line, text.as_bytes())
        }
    }
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
