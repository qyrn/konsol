use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::Duration;
use ureq::Agent;

const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

pub fn agent() -> Agent {
    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .user_agent("Konsol")
        .build()
        .into()
}

pub fn get_json(
    agent: &Agent,
    url: &str,
    query: &[(&str, &str)],
    bearer: Option<&str>,
) -> Option<Value> {
    let mut request = agent.get(url);
    for (name, value) in query {
        request = request.query(*name, *value);
    }
    if let Some(token) = bearer {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    request.call().ok()?.body_mut().read_json().ok()
}

pub fn download(agent: &Agent, url: &str, destination: &Path) -> bool {
    if !url.starts_with("https://") {
        return false;
    }
    let Ok(mut response) = agent.get(url).call() else {
        return false;
    };
    let Ok(bytes) = response
        .body_mut()
        .with_config()
        .limit(MAX_IMAGE_BYTES)
        .read_to_vec()
    else {
        return false;
    };
    destination
        .parent()
        .is_some_and(|directory| fs::create_dir_all(directory).is_ok())
        && fs::write(destination, bytes).is_ok()
}

pub fn encode_path_segment(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_titles_for_url_paths() {
        assert_eq!(
            encode_path_segment("Skate 3/Recomp?"),
            "Skate%203%2FRecomp%3F"
        );
    }
}
