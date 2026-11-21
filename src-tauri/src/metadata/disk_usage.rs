use std::fs;
use std::path::Path;

const MAX_ENTRIES: usize = 500_000;

pub fn game_folder_size(home: &Path, location: &Path) -> Option<u64> {
    let folder = location.parent()?;
    if !folder.starts_with(home) || folder == home {
        return None;
    }
    directory_size(folder)
}

fn directory_size(root: &Path) -> Option<u64> {
    let mut total = 0u64;
    let mut pending = vec![root.to_path_buf()];
    let mut visited = 0usize;
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).ok()?.filter_map(Result::ok) {
            visited += 1;
            if visited > MAX_ENTRIES {
                return None;
            }
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                total += metadata.len();
            }
        }
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_game_folders_inside_home_only() {
        let home = std::env::temp_dir().join(format!("konsol-disk-{}", std::process::id()));
        let game = home.join("Games/Skate");
        fs::create_dir_all(game.join("data")).unwrap();
        fs::write(game.join("skate3"), [0u8; 10]).unwrap();
        fs::write(game.join("data/pack"), [0u8; 32]).unwrap();
        assert_eq!(game_folder_size(&home, &game.join("skate3")), Some(42));
        assert_eq!(game_folder_size(&home, &home.join("game")), None);
        assert_eq!(game_folder_size(&home, Path::new("/usr/bin/true")), None);
        fs::remove_dir_all(home).unwrap();
    }
}
