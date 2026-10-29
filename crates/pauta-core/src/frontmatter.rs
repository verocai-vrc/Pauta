//! Markdown files with a YAML header (frontmatter), edited without disturbing
//! anything the edit is not about.

use crate::yaml::{Y, YamlText};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc {
    /// Opening delimiter line, including its newline (`"---\n"`), or empty if the
    /// file has no frontmatter yet.
    open: String,
    fm: YamlText,
    /// Closing delimiter line including its newline.
    close: String,
    body: String,
}

impl Doc {
    /// Splits a file into frontmatter and body. Never fails: a file without a header
    /// is all body. Whether the header is valid YAML is checked by [`Doc::fields`].
    pub fn parse(raw: &str) -> Doc {
        // Some Windows editors prepend a UTF-8 byte-order mark. It stays glued to
        // the opening delimiter so the file still renders back byte for byte.
        let (bom, text) = match raw.strip_prefix('\u{feff}') {
            Some(rest) => ("\u{feff}", rest),
            None => ("", raw),
        };
        let mut doc = Doc::parse_unmarked(text);
        if doc.has_header() {
            doc.open.insert_str(0, bom);
        } else {
            doc = Doc::without_header(raw);
        }
        doc
    }

    fn parse_unmarked(raw: &str) -> Doc {
        let first_end = raw.find('\n').map_or(raw.len(), |i| i + 1);
        let first = &raw[..first_end];
        if first.trim_end() != "---" {
            return Doc::without_header(raw);
        }
        let mut offset = first_end;
        for line in raw[first_end..].split_inclusive('\n') {
            let trimmed = line.trim_end();
            if trimmed == "---" || trimmed == "..." {
                return Doc {
                    open: first.to_string(),
                    fm: YamlText::new(&raw[first_end..offset]),
                    close: line.to_string(),
                    body: raw[offset + line.len()..].to_string(),
                };
            }
            offset += line.len();
        }
        // Unterminated header: treat as plain body so we never lose text.
        Doc::without_header(raw)
    }

    fn without_header(raw: &str) -> Doc {
        Doc {
            open: String::new(),
            fm: YamlText::new(""),
            close: String::new(),
            body: raw.to_string(),
        }
    }

    pub fn new(body: &str) -> Doc {
        Doc {
            open: "---\n".into(),
            fm: YamlText::new(""),
            close: "---\n".into(),
            body: body.to_string(),
        }
    }

    pub fn has_header(&self) -> bool {
        !self.open.is_empty()
    }

    pub fn fields(&self) -> Result<serde_yaml_ng::Mapping, String> {
        if !self.has_header() {
            return Err("arquivo sem cabeçalho YAML (---)".into());
        }
        self.fm.parse()
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn set_body(&mut self, body: &str) {
        self.body = body.to_string();
    }

    pub fn set(&mut self, key: &str, value: &Y) {
        self.ensure_header();
        self.fm.set(key, value);
    }

    pub fn remove(&mut self, key: &str) {
        self.fm.remove(key);
    }

    fn ensure_header(&mut self) {
        if !self.has_header() {
            self.open = "---\n".into();
            self.close = "---\n".into();
            if !self.body.is_empty() && !self.body.starts_with('\n') {
                self.body.insert(0, '\n');
            }
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::with_capacity(self.open.len() + self.fm.as_str().len() + self.body.len() + 8);
        out.push_str(&self.open);
        out.push_str(self.fm.as_str());
        out.push_str(&self.close);
        out.push_str(&self.body);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: &str = "---\nid: 01JBX3K9Q2\ntitle: \"Por que Rust\"\nunknown_field: { a: 1 }\ncolumn: draft\n---\n\n## LinkedIn\n\nTexto...\n";

    #[test]
    fn round_trip_is_identical() {
        assert_eq!(Doc::parse(CARD).render(), CARD);
        for raw in [
            "",
            "no header\n",
            "---\nunterminated: 1\n",
            "---\r\na: 1\r\n---\r\nbody",
            "---\n---\n",
        ] {
            assert_eq!(Doc::parse(raw).render(), raw);
        }
    }

    #[test]
    fn edit_changes_only_that_field() {
        let mut d = Doc::parse(CARD);
        d.set("column", &Y::str("producing"));
        assert_eq!(d.render(), CARD.replace("column: draft", "column: producing"));
    }

    #[test]
    fn byte_order_mark_is_kept_and_does_not_hide_the_header() {
        let raw = "\u{feff}---\nid: x1\ntitle: \"A\"\n---\n\ncorpo\n";
        let mut d = Doc::parse(raw);
        assert!(d.has_header());
        assert_eq!(d.render(), raw);
        assert!(d.fields().is_ok());
        assert_eq!(d.body(), "\ncorpo\n");
        d.set("title", &Y::Quoted("B".into()));
        assert_eq!(d.render(), raw.replace("\"A\"", "\"B\""));
        // A BOM on a file with no header is just text.
        let plain = "\u{feff}sem cabeçalho\n";
        assert_eq!(Doc::parse(plain).render(), plain);
        assert!(!Doc::parse(plain).has_header());
    }

    #[test]
    fn header_is_added_when_missing() {
        let mut d = Doc::parse("just text\n");
        d.set("id", &Y::str("x1"));
        assert_eq!(d.render(), "---\nid: x1\n---\n\njust text\n");
    }
}
