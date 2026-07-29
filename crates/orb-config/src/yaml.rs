//! Flat `key: value` reader for `orb.yaml`.
//!
//! Not a YAML implementation: no nesting, sequences, anchors or multi-line
//! scalars. A real YAML crate would pull serde into a DLL that must stay free
//! of surprises, and the config is a flat list of scalars, so the subset is
//! parsed directly and unknown keys are rejected instead of ignored.

use std::collections::BTreeMap;
use std::fmt;

pub struct Document {
    entries: BTreeMap<String, Entry>,
    taken: std::cell::RefCell<Vec<String>>,
}

struct Entry {
    value: String,
    line: usize,
}

#[derive(Debug)]
pub struct Error {
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for Error {}

fn err(line: usize, message: impl Into<String>) -> Error {
    Error { line: Some(line), message: message.into() }
}

impl Document {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let mut entries = BTreeMap::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let content = strip_comment(raw);
            if content.trim().is_empty() {
                continue;
            }
            if content.starts_with([' ', '\t']) {
                return Err(err(line, "unexpected indentation; orb.yaml is a flat key: value list"));
            }
            let (key, value) = content
                .split_once(':')
                .ok_or_else(|| err(line, "expected `key: value`"))?;
            let key = key.trim();
            if key.is_empty() {
                return Err(err(line, "empty key"));
            }
            let value = unquote(value.trim(), line)?;
            if entries.insert(key.to_owned(), Entry { value, line }).is_some() {
                return Err(err(line, format!("duplicate key `{key}`")));
            }
        }
        Ok(Self { entries, taken: std::cell::RefCell::new(Vec::new()) })
    }

    /// `Some("")` for a key written with no value, which callers may read as a
    /// deliberate "none" distinct from the key being absent.
    pub fn string(&self, key: &str) -> Result<Option<&str>, Error> {
        Ok(self.entry(key).map(|entry| entry.value.as_str()))
    }

    pub fn bool(&self, key: &str) -> Result<Option<bool>, Error> {
        let Some(entry) = self.non_empty_entry(key) else { return Ok(None) };
        match entry.value.as_str() {
            "true" | "yes" | "on" => Ok(Some(true)),
            "false" | "no" | "off" => Ok(Some(false)),
            other => Err(err(entry.line, format!("`{key}`: expected true or false, got `{other}`"))),
        }
    }

    pub fn u32(&self, key: &str) -> Result<Option<u32>, Error> {
        let Some(entry) = self.non_empty_entry(key) else { return Ok(None) };
        entry
            .value
            .parse()
            .map(Some)
            .map_err(|_| err(entry.line, format!("`{key}`: expected a number, got `{}`", entry.value)))
    }

    fn entry(&self, key: &str) -> Option<&Entry> {
        self.taken.borrow_mut().push(key.to_owned());
        self.entries.get(key)
    }

    fn non_empty_entry(&self, key: &str) -> Option<&Entry> {
        self.entry(key).filter(|entry| !entry.value.is_empty())
    }

    /// Every key the caller never asked about is a typo or a leftover from an
    /// older build; both silently change behaviour, so they are hard errors.
    pub fn reject_unknown_keys(&self) -> Result<(), Error> {
        let taken = self.taken.borrow();
        for (key, entry) in &self.entries {
            if !taken.iter().any(|known| known == key) {
                return Err(err(entry.line, format!("unknown key `{key}`")));
            }
        }
        Ok(())
    }
}

fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut quote = None;
    for (index, &byte) in bytes.iter().enumerate() {
        match (quote, byte) {
            (None, b'"' | b'\'') => quote = Some(byte),
            (Some(open), byte) if byte == open => quote = None,
            (None, b'#') if index == 0 || bytes[index - 1].is_ascii_whitespace() => {
                return &line[..index];
            }
            _ => {}
        }
    }
    line
}

fn unquote(value: &str, line: usize) -> Result<String, Error> {
    let mut chars = value.chars();
    let Some(open @ ('"' | '\'')) = chars.next() else {
        return Ok(value.to_owned());
    };
    let rest = chars.as_str();
    match rest.strip_suffix(open) {
        Some(inner) if !inner.contains(open) => Ok(inner.to_owned()),
        _ => Err(err(line, "unterminated quoted value")),
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn reads_scalars_and_skips_comments() {
        let doc = Document::parse(
            "# leading comment\n\
             game_dir: L:\\game\\th06  # trailing comment\n\
             \n\
             self_check: true\n\
             quoted: \"a: b\"\n",
        )
        .unwrap();
        assert_eq!(doc.string("game_dir").unwrap(), Some("L:\\game\\th06"));
        assert_eq!(doc.bool("self_check").unwrap(), Some(true));
        assert_eq!(doc.string("quoted").unwrap(), Some("a: b"));
    }

    #[test]
    fn a_valueless_key_is_distinguishable_from_an_absent_one() {
        let doc = Document::parse("game_dir:\nself_check:\n").unwrap();
        assert_eq!(doc.string("game_dir").unwrap(), Some(""));
        assert_eq!(doc.string("missing").unwrap(), None);
        assert_eq!(doc.bool("self_check").unwrap(), None);
    }

    #[test]
    fn hash_inside_a_value_is_not_a_comment() {
        let doc = Document::parse("key: a#b\n").unwrap();
        assert_eq!(doc.string("key").unwrap(), Some("a#b"));
    }

    #[test]
    fn rejects_unknown_keys_only_after_the_known_ones_are_read() {
        let doc = Document::parse("self_check: true\nselfcheck: true\n").unwrap();
        doc.bool("self_check").unwrap();
        let error = doc.reject_unknown_keys().unwrap_err();
        assert_eq!(error.line, Some(2));
        assert!(error.message.contains("selfcheck"), "{}", error.message);
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(Document::parse("no colon here\n").is_err());
        assert!(Document::parse("  indented: 1\n").is_err());
        assert!(Document::parse("dup: 1\ndup: 2\n").is_err());
        assert!(Document::parse("bad: \"unterminated\n").is_err());
    }

    #[test]
    fn rejects_values_of_the_wrong_type() {
        let doc = Document::parse("self_check: maybe\n").unwrap();
        assert!(doc.bool("self_check").is_err());
    }
}
