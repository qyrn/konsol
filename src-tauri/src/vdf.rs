#[derive(Debug, PartialEq)]
pub enum VdfValue {
    Text(String),
    Section(Vec<(String, VdfValue)>),
}

enum Token {
    Text(String),
    Open,
    Close,
}

impl VdfValue {
    pub fn get(&self, key: &str) -> Option<&VdfValue> {
        self.entries()
            .iter()
            .find(|(entry_key, _)| entry_key.eq_ignore_ascii_case(key))
            .map(|(_, value)| value)
    }

    pub fn get_text(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            VdfValue::Text(text) => Some(text),
            VdfValue::Section(_) => None,
        }
    }

    pub fn entries(&self) -> &[(String, VdfValue)] {
        match self {
            VdfValue::Section(entries) => entries,
            VdfValue::Text(_) => &[],
        }
    }
}

pub fn parse(input: &str) -> VdfValue {
    let mut tokens = tokenize(input).into_iter();
    VdfValue::Section(parse_entries(&mut tokens))
}

fn parse_entries(tokens: &mut impl Iterator<Item = Token>) -> Vec<(String, VdfValue)> {
    let mut entries = Vec::new();
    while let Some(Token::Text(key)) = tokens.next() {
        match tokens.next() {
            Some(Token::Text(value)) => entries.push((key, VdfValue::Text(value))),
            Some(Token::Open) => entries.push((key, VdfValue::Section(parse_entries(tokens)))),
            Some(Token::Close) | None => break,
        }
    }
    entries
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '{' => tokens.push(Token::Open),
            '}' => tokens.push(Token::Close),
            '"' => {
                let mut text = String::new();
                while let Some(inner) = chars.next() {
                    match inner {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('n') => text.push('\n'),
                            Some('t') => text.push('\t'),
                            Some(escaped) => text.push(escaped),
                            None => break,
                        },
                        other => text.push(other),
                    }
                }
                tokens.push(Token::Text(text));
            }
            '/' if chars.peek() == Some(&'/') => {
                for skipped in chars.by_ref() {
                    if skipped == '\n' {
                        break;
                    }
                }
            }
            whitespace if whitespace.is_whitespace() => {}
            first => {
                let mut text = String::from(first);
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() || next == '{' || next == '}' || next == '"' {
                        break;
                    }
                    text.push(next);
                    chars.next();
                }
                tokens.push(Token::Text(text));
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_sections_and_escapes() {
        let input = r#"
"AppState"
{
    // comment
    "appid"     "250900"
    "name"      "The Binding of Isaac: \"Rebirth\""
    "UserConfig" { "language" "french" }
}
"#;
        let root = parse(input);
        let app_state = root.get("appstate").expect("AppState section");
        assert_eq!(app_state.get_text("appid"), Some("250900"));
        assert_eq!(
            app_state.get_text("name"),
            Some("The Binding of Isaac: \"Rebirth\"")
        );
        assert_eq!(
            app_state
                .get("userconfig")
                .and_then(|config| config.get_text("language")),
            Some("french")
        );
    }

    #[test]
    fn tolerates_truncated_input() {
        let root = parse("\"libraryfolders\" { \"0\" { \"path\" \"/a\"");
        let library = root
            .get("libraryfolders")
            .and_then(|folders| folders.get("0"));
        assert_eq!(library.and_then(|entry| entry.get_text("path")), Some("/a"));
    }
}
