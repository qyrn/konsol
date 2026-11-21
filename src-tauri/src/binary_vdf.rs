use crate::vdf::VdfValue;

const NESTED: u8 = 0;
const STRING: u8 = 1;
const INT32: u8 = 2;
const FLOAT32: u8 = 3;
const UINT64: u8 = 7;
const END: u8 = 8;

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Reader<'_> {
    fn byte(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.position)?;
        self.position += 1;
        Some(byte)
    }

    fn take<const SIZE: usize>(&mut self) -> Option<[u8; SIZE]> {
        let slice = self.bytes.get(self.position..self.position + SIZE)?;
        self.position += SIZE;
        slice.try_into().ok()
    }

    fn text(&mut self) -> Option<String> {
        let rest = self.bytes.get(self.position..)?;
        let length = rest.iter().position(|byte| *byte == 0)?;
        self.position += length + 1;
        Some(String::from_utf8_lossy(&rest[..length]).into_owned())
    }

    fn entries(&mut self) -> Option<Vec<(String, VdfValue)>> {
        let mut entries = Vec::new();
        loop {
            let kind = self.byte()?;
            if kind == END {
                return Some(entries);
            }
            let key = self.text()?;
            let value = match kind {
                NESTED => VdfValue::Section(self.entries()?),
                STRING => VdfValue::Text(self.text()?),
                INT32 => VdfValue::Text(i32::from_le_bytes(self.take()?).to_string()),
                FLOAT32 => VdfValue::Text(f32::from_le_bytes(self.take()?).to_string()),
                UINT64 => VdfValue::Text(u64::from_le_bytes(self.take()?).to_string()),
                _ => return None,
            };
            entries.push((key, value));
        }
    }
}

pub fn parse(bytes: &[u8]) -> Option<VdfValue> {
    let mut reader = Reader { bytes, position: 0 };
    let mut entries = Vec::new();
    while reader.position < bytes.len() {
        let kind = reader.byte()?;
        if kind == END {
            break;
        }
        let key = reader.text()?;
        if kind != NESTED {
            return None;
        }
        entries.push((key, VdfValue::Section(reader.entries()?)));
    }
    Some(VdfValue::Section(entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(key: &str, body: &[u8]) -> Vec<u8> {
        let mut bytes = vec![NESTED];
        bytes.extend_from_slice(key.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(body);
        bytes.push(END);
        bytes
    }

    #[test]
    fn parses_nested_strings_and_numbers() {
        let mut body = vec![STRING];
        body.extend_from_slice(b"name\0Magdalene\0");
        body.push(INT32);
        body.extend_from_slice(b"data\0");
        body.extend_from_slice(&(-5_i32).to_le_bytes());
        let root = parse(&section("250900", &body)).expect("valid vdf");
        let game = root.get("250900").expect("app section");
        assert_eq!(game.get_text("name"), Some("Magdalene"));
        assert_eq!(game.get_text("data"), Some("-5"));
    }

    #[test]
    fn rejects_truncated_or_unknown_data() {
        assert_eq!(parse(&[NESTED, b'a']), None);
        assert_eq!(parse(&section("a", &[42, b'k', 0])), None);
    }
}
