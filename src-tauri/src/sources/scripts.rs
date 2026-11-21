use crate::game::{Game, GameSource};
use crate::sources::is_executable_file;
use std::fs;
use std::path::Path;

pub fn scan(home: &Path) -> Vec<Game> {
    let Ok(entries) = fs::read_dir(home.join("Games")) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "sh"))
        .filter(|path| is_executable_file(path))
        .filter_map(|path| {
            let file_name = path.file_name()?.to_str()?.to_string();
            let stem = path.file_stem()?.to_str()?;
            let title = title_from_file_stem(stem);
            Some(Game::program(
                format!("script:{file_name}"),
                title,
                GameSource::Script,
                path,
            ))
        })
        .collect()
}

fn title_from_file_stem(stem: &str) -> String {
    stem.split(['-', '_', ' '])
        .filter(|word| !word.is_empty())
        .map(|word| {
            if word.chars().count() <= 2 {
                return word.to_uppercase();
            }
            let mut characters = word.chars();
            characters
                .next()
                .map(|first| first.to_uppercase().chain(characters).collect())
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_readable_titles_from_script_names() {
        assert_eq!(
            title_from_file_stem("rockstar-launcher"),
            "Rockstar Launcher"
        );
        assert_eq!(title_from_file_stem("skyrim-ae"), "Skyrim AE");
        assert_eq!(title_from_file_stem("the_sims__4"), "The Sims 4");
    }
}
