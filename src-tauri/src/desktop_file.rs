use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct DesktopFile {
    pub name: String,
    pub command: Vec<String>,
    pub categories: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub no_display: bool,
}

pub fn parse(content: &str) -> Option<DesktopFile> {
    let mut in_main_group = false;
    let mut name = None;
    let mut exec = None;
    let mut categories = Vec::new();
    let mut working_directory = None;
    let mut no_display = false;
    for line in content.lines().map(str::trim) {
        if line.starts_with('[') {
            in_main_group = line == "[Desktop Entry]";
            continue;
        }
        if !in_main_group {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = unescape_value(value.trim());
        match key.trim() {
            "Name" => name = Some(value),
            "Exec" => exec = Some(value),
            "Categories" => {
                categories = value
                    .split(';')
                    .filter(|category| !category.is_empty())
                    .map(str::to_string)
                    .collect();
            }
            "Path" if !value.is_empty() => working_directory = Some(PathBuf::from(value)),
            "NoDisplay" | "Hidden" => no_display |= value == "true",
            _ => {}
        }
    }
    let command = split_exec(&exec?)?;
    Some(DesktopFile {
        name: name?,
        command,
        categories,
        working_directory,
        no_display,
    })
}

fn unescape_value(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            result.push(character);
            continue;
        }
        match characters.next() {
            Some('s') => result.push(' '),
            Some('n') => result.push('\n'),
            Some('t') => result.push('\t'),
            Some('r') => result.push('\r'),
            Some('\\') => result.push('\\'),
            Some(other) => {
                result.push('\\');
                result.push(other);
            }
            None => result.push('\\'),
        }
    }
    result
}

pub fn split_exec(exec: &str) -> Option<Vec<String>> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut has_token = false;
    let mut in_quotes = false;
    let mut characters = exec.chars();
    while let Some(character) = characters.next() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                has_token = true;
            }
            '\\' if in_quotes => current.push(characters.next()?),
            '%' => match characters.next()? {
                '%' => current.push('%'),
                _ => has_token |= !current.is_empty(),
            },
            whitespace if whitespace.is_whitespace() && !in_quotes => {
                if has_token || !current.is_empty() {
                    arguments.push(std::mem::take(&mut current));
                }
                has_token = false;
            }
            other => current.push(other),
        }
    }
    if in_quotes {
        return None;
    }
    if has_token || !current.is_empty() {
        arguments.push(current);
    }
    (!arguments.is_empty()).then_some(arguments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_wine_game_entry() {
        let content = "[Desktop Entry]\nName=Shipping Store Simulator 2037\nExec=env WINEPREFIX=/home/q/pfx GAMEID=0 umu-run \"/home/q/pfx/drive_c/Games/SSS2037/game.exe\"\nIcon=applications-games\nCategories=Game;\n[Desktop Action x]\nName=Other\n";
        let entry = parse(content).expect("valid entry");
        assert_eq!(entry.name, "Shipping Store Simulator 2037");
        assert_eq!(
            entry.command,
            [
                "env",
                "WINEPREFIX=/home/q/pfx",
                "GAMEID=0",
                "umu-run",
                "/home/q/pfx/drive_c/Games/SSS2037/game.exe"
            ]
        );
        assert_eq!(entry.categories, ["Game"]);
        assert!(!entry.no_display);
    }

    #[test]
    fn handles_quotes_escapes_and_field_codes() {
        assert_eq!(
            split_exec(r#""/opt/my game/run" --level "a \"b\"" %U 100%%"#).unwrap(),
            ["/opt/my game/run", "--level", "a \"b\"", "100%"]
        );
        assert_eq!(split_exec("\"unterminated"), None);
        assert_eq!(unescape_value(r"a\sb\\c"), r"a b\c");
    }
}
