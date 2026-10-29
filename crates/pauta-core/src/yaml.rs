//! Surgical YAML editing.
//!
//! Pauta never re-serializes a whole YAML document. It splits the text into
//! top-level key blocks and replaces only the block being changed, so unknown
//! keys, comments, ordering and formatting elsewhere survive byte for byte
//! (invariant 4, section 14.4). See docs/adr/0002-edicao-cirurgica-do-cabecalho.md.

use serde_yaml_ng::Value;

/// A value to be written. This is deliberately not `serde_yaml_ng::Value`:
/// we control the output style (flow lists, quoted titles, unquoted dates)
/// so files look like the examples in the data-format doc.
#[derive(Debug, Clone, PartialEq)]
pub enum Y {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    /// Quoted only when needed.
    Str(String),
    /// Always double-quoted (titles, colors, times).
    Quoted(String),
    /// Written verbatim, unquoted. Only for values we generate (dates, timestamps).
    Plain(String),
    Seq(Vec<Y>),
    Map(Vec<(String, Y)>),
}

impl Y {
    pub fn str(s: impl Into<String>) -> Y {
        Y::Str(s.into())
    }

    pub fn opt_str(s: Option<impl Into<String>>) -> Y {
        s.map_or(Y::Null, |s| Y::Str(s.into()))
    }

    pub fn str_list<S: AsRef<str>>(items: &[S]) -> Y {
        Y::Seq(items.iter().map(|s| Y::Str(s.as_ref().to_string())).collect())
    }

    /// Converts a parsed value back into a writable one, keeping every field.
    pub fn from_value(v: &Value) -> Y {
        match v {
            Value::Null => Y::Null,
            Value::Bool(b) => Y::Bool(*b),
            Value::Number(n) => match n.as_i64() {
                Some(i) => Y::Int(i),
                None => Y::Float(n.as_f64().unwrap_or(0.0)),
            },
            Value::String(s) if looks_like_date(s) => Y::Plain(s.clone()),
            Value::String(s) => Y::Str(s.clone()),
            Value::Sequence(items) => Y::Seq(items.iter().map(Y::from_value).collect()),
            Value::Mapping(m) => Y::Map(m.iter().map(|(k, v)| (scalar_to_string(k), Y::from_value(v))).collect()),
            Value::Tagged(t) => Y::Str(format!("{} {:?}", t.tag, t.value)),
        }
    }

    fn is_scalar(&self) -> bool {
        !matches!(self, Y::Seq(_) | Y::Map(_))
    }

    /// Small enough to write on one line: `{ a: 1, b: [x, y] }`.
    fn is_flat(&self) -> bool {
        match self {
            Y::Seq(items) => items.iter().all(Y::is_scalar),
            Y::Map(entries) => entries
                .iter()
                .all(|(_, v)| v.is_scalar() || matches!(v, Y::Seq(i) if i.iter().all(Y::is_scalar))),
            _ => true,
        }
    }
}

pub fn scalar_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => serde_yaml_ng::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

pub fn looks_like_date(s: &str) -> bool {
    s.len() >= 10
        && s.as_bytes()[4] == b'-'
        && (s.parse::<jiff::civil::Date>().is_ok()
            || s.parse::<jiff::Timestamp>().is_ok()
            || s.parse::<jiff::civil::DateTime>().is_ok())
}

fn plain_ok(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else { return false };
    if !first.is_alphabetic() {
        return false;
    }
    if !s
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
    {
        return false;
    }
    let lower = s.to_lowercase();
    !matches!(
        lower.as_str(),
        "null" | "true" | "false" | "yes" | "no" | "on" | "off" | "y" | "n"
    )
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn scalar(v: &Y) -> String {
    match v {
        Y::Null => "null".into(),
        Y::Bool(b) => b.to_string(),
        Y::Int(i) => i.to_string(),
        Y::Float(f) => format!("{f:?}"),
        Y::Str(s) if plain_ok(s) => s.clone(),
        Y::Str(s) | Y::Quoted(s) => quote(s),
        Y::Plain(s) => s.clone(),
        Y::Seq(_) | Y::Map(_) => flow(v),
    }
}

fn flow(v: &Y) -> String {
    match v {
        Y::Seq(items) => {
            let parts: Vec<String> = items.iter().map(flow).collect();
            format!("[{}]", parts.join(", "))
        }
        Y::Map(entries) if entries.is_empty() => "{}".into(),
        Y::Map(entries) => {
            let parts: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("{}: {}", key(k), flow(v)))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
        other => scalar(other),
    }
}

fn key(k: &str) -> String {
    if plain_ok(k) { k.to_string() } else { quote(k) }
}

/// Renders `key: value` (plus nested lines) at the given indentation.
pub fn render_entry(k: &str, v: &Y, indent: usize, nl: &str) -> String {
    let pad = " ".repeat(indent);
    let mut out = String::new();
    match v {
        // Small nested maps stay on one line: `quiet_hours: { from: "23:00", to: "07:00" }`.
        Y::Map(entries) if !entries.is_empty() && !(indent > 0 && v.is_flat() && entries.len() <= 3) => {
            out.push_str(&format!("{pad}{}:{nl}", key(k)));
            for (ck, cv) in entries {
                out.push_str(&render_entry(ck, cv, indent + 2, nl));
            }
        }
        Y::Seq(items) if !items.is_empty() && !v.is_flat() => {
            out.push_str(&format!("{pad}{}:{nl}", key(k)));
            for item in items {
                out.push_str(&render_item(item, indent + 2, nl));
            }
        }
        other => out.push_str(&format!("{pad}{}: {}{nl}", key(k), flow(other))),
    }
    out
}

fn render_item(v: &Y, indent: usize, nl: &str) -> String {
    let pad = " ".repeat(indent);
    match v {
        Y::Map(entries) if !entries.is_empty() && !v.is_flat() => {
            // First entry shares the line with the dash.
            let mut out = String::new();
            for (i, (k, cv)) in entries.iter().enumerate() {
                let rendered = render_entry(k, cv, indent + 2, nl);
                if i == 0 {
                    out.push_str(&format!("{pad}- {}", &rendered[indent + 2..]));
                } else {
                    out.push_str(&rendered);
                }
            }
            out
        }
        other => format!("{pad}- {}{nl}", flow(other)),
    }
}

/// A YAML mapping document kept as text, edited one top-level key at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlText {
    text: String,
}

struct Block {
    key: String,
    /// Byte range of the key line plus its nested lines (no trailing blanks/comments).
    start: usize,
    end: usize,
}

impl YamlText {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn into_string(self) -> String {
        self.text
    }

    pub fn parse(&self) -> Result<serde_yaml_ng::Mapping, String> {
        if self.text.trim().is_empty() {
            return Ok(serde_yaml_ng::Mapping::new());
        }
        match serde_yaml_ng::from_str::<Value>(&self.text) {
            Ok(Value::Mapping(m)) => Ok(m),
            Ok(Value::Null) => Ok(serde_yaml_ng::Mapping::new()),
            Ok(_) => Err("o cabeçalho YAML não é um mapa de chaves".into()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn newline(&self) -> &'static str {
        if self.text.contains("\r\n") { "\r\n" } else { "\n" }
    }

    fn blocks(&self) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut offset = 0;
        // End of the last line that belongs to the current block's content.
        let mut content_end = 0;
        for line in self.text.split_inclusive('\n') {
            let start = offset;
            offset += line.len();
            if let Some(k) = key_of_line(line) {
                if let Some(last) = blocks.last_mut() {
                    last.end = content_end;
                }
                blocks.push(Block {
                    key: k,
                    start,
                    end: offset,
                });
                content_end = offset;
                continue;
            }
            let trimmed = line.trim();
            let is_trailing_noise = trimmed.is_empty() || (trimmed.starts_with('#') && !line.starts_with(' '));
            if !is_trailing_noise {
                content_end = offset;
            }
        }
        if let Some(last) = blocks.last_mut() {
            last.end = content_end.max(last.start);
            if last.end < last.start {
                last.end = offset;
            }
        }
        blocks
    }

    pub fn has(&self, k: &str) -> bool {
        self.blocks().iter().any(|b| b.key == k)
    }

    /// Replaces the block for `k`, or appends it at the end.
    pub fn set(&mut self, k: &str, v: &Y) {
        let nl = self.newline();
        let rendered = render_entry(k, v, 0, nl);
        if let Some(b) = self.blocks().into_iter().find(|b| b.key == k) {
            if self.text[b.start..b.end] != rendered {
                self.text.replace_range(b.start..b.end, &rendered);
            }
        } else {
            if !self.text.is_empty() && !self.text.ends_with('\n') {
                self.text.push_str(nl);
            }
            self.text.push_str(&rendered);
        }
    }

    pub fn remove(&mut self, k: &str) {
        if let Some(b) = self.blocks().into_iter().find(|b| b.key == k) {
            self.text.replace_range(b.start..b.end, "");
        }
    }
}

/// Returns the key if `line` starts a top-level mapping entry.
fn key_of_line(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if first.is_whitespace() || matches!(first, '#' | '-' | '[' | '{' | '}' | ']') {
        return None;
    }
    let line = line.trim_end_matches(['\n', '\r']);
    if let Some(q) = line
        .strip_prefix('"')
        .map(|r| (r, '"'))
        .or_else(|| line.strip_prefix('\'').map(|r| (r, '\'')))
    {
        let (rest, delim) = q;
        let close = rest.find(delim)?;
        let after = &rest[close + 1..];
        return after.starts_with(':').then(|| rest[..close].to_string());
    }
    let mut search = 0;
    while let Some(pos) = line[search..].find(':') {
        let idx = search + pos;
        let next = line[idx + 1..].chars().next();
        if next.is_none() || next == Some(' ') || next == Some('\t') {
            return Some(line[..idx].trim_end().to_string());
        }
        search = idx + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_only_touches_one_block() {
        let src = "id: 1\n# comment about title\ntitle: \"Old\"\nweird_unknown:\n  nested: [1, 2]   # keep me\n\ncolumn: draft\n";
        let mut y = YamlText::new(src);
        y.set("column", &Y::str("doing"));
        assert_eq!(
            y.as_str(),
            "id: 1\n# comment about title\ntitle: \"Old\"\nweird_unknown:\n  nested: [1, 2]   # keep me\n\ncolumn: doing\n"
        );
        y.set("title", &Y::Quoted("New".into()));
        assert!(
            y.as_str()
                .contains("# comment about title\ntitle: \"New\"\nweird_unknown:")
        );
    }

    #[test]
    fn set_identical_value_is_noop() {
        let src = "tags: [rust, divulgacao]\n";
        let mut y = YamlText::new(src);
        y.set("tags", &Y::str_list(&["rust", "divulgacao"]));
        assert_eq!(y.as_str(), src);
    }

    #[test]
    fn nested_maps_and_items() {
        let v = Y::Map(vec![
            ("at".into(), Y::Plain("2026-11-04T09:00:00-03:00".into())),
            ("status".into(), Y::str("scheduled")),
            ("url".into(), Y::Null),
        ]);
        assert_eq!(
            render_entry("publication", &v, 0, "\n"),
            "publication:\n  at: 2026-11-04T09:00:00-03:00\n  status: scheduled\n  url: null\n"
        );
        let cols = Y::Seq(vec![Y::Map(vec![
            ("id".into(), Y::str("idea")),
            ("name".into(), Y::Quoted("Ideia".into())),
        ])]);
        assert_eq!(
            render_entry("columns", &cols, 0, "\n"),
            "columns:\n  - { id: idea, name: \"Ideia\" }\n"
        );
    }

    #[test]
    fn quoting_rules() {
        assert_eq!(scalar(&Y::str("-1h")), "\"-1h\"");
        assert_eq!(scalar(&Y::str("0")), "\"0\"");
        assert_eq!(scalar(&Y::str("09:00")), "\"09:00\"");
        assert_eq!(scalar(&Y::str("rust")), "rust");
        assert_eq!(scalar(&Y::str("yes")), "\"yes\"");
        assert_eq!(scalar(&Y::str("divulgação")), "divulgação");
        assert_eq!(scalar(&Y::str("a \"b\"\n")), "\"a \\\"b\\\"\\n\"");
    }

    #[test]
    fn rendered_values_parse_back() {
        let v = Y::Map(vec![
            ("s".into(), Y::str("x: y # z")),
            ("l".into(), Y::str_list(&["-1h", "0", "09:00"])),
            ("q".into(), Y::Quoted("tab\there".into())),
        ]);
        let text = render_entry("root", &v, 0, "\n");
        let parsed: Value = serde_yaml_ng::from_str(&text).unwrap();
        assert_eq!(Y::from_value(&parsed["root"]["s"]), Y::str("x: y # z"));
        assert_eq!(parsed["root"]["q"].as_str(), Some("tab\there"));
        assert_eq!(parsed["root"]["l"][1].as_str(), Some("0"));
    }

    #[test]
    fn remove_and_append() {
        let mut y = YamlText::new("a: 1\nb:\n  - x\n  - y\nc: 3\n");
        y.remove("b");
        assert_eq!(y.as_str(), "a: 1\nc: 3\n");
        y.set("d", &Y::Int(4));
        assert_eq!(y.as_str(), "a: 1\nc: 3\nd: 4\n");
    }

    #[test]
    fn block_sequence_at_column_zero_stays_in_block() {
        let mut y = YamlText::new("tags:\n- a\n- b\nnext: 1\n");
        y.set("tags", &Y::str_list(&["c"]));
        assert_eq!(y.as_str(), "tags: [c]\nnext: 1\n");
    }
}
