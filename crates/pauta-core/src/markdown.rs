//! Reading the Markdown body: `## Section`s and `- [ ]` checklists.
//! The body is never re-rendered; edits splice text into the original string.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Checklist {
    pub done: usize,
    pub total: usize,
}

pub fn checklist(body: &str) -> Checklist {
    let mut c = Checklist::default();
    for ev in Parser::new_ext(body, Options::ENABLE_TASKLISTS) {
        if let Event::TaskListMarker(checked) = ev {
            c.total += 1;
            c.done += usize::from(checked);
        }
    }
    c
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    /// Byte range of the heading line.
    pub heading: std::ops::Range<usize>,
    /// Byte range of the content, up to the next `##` heading or the end.
    pub content: std::ops::Range<usize>,
}

/// Level-2 sections, found with a real Markdown parser so `##` inside code
/// blocks does not count.
pub fn sections(body: &str) -> Vec<Section> {
    let mut heads: Vec<(String, std::ops::Range<usize>)> = Vec::new();
    let mut current: Option<(String, std::ops::Range<usize>)> = None;
    for (ev, range) in Parser::new_ext(body, Options::empty()).into_offset_iter() {
        match ev {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => current = Some((String::new(), range)),
            Event::Text(t) | Event::Code(t) => {
                if let Some((title, _)) = current.as_mut() {
                    title.push_str(&t);
                }
            }
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                if let Some(h) = current.take() {
                    heads.push(h);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::with_capacity(heads.len());
    for (i, (title, heading)) in heads.iter().enumerate() {
        let start = heading.end;
        let end = heads.get(i + 1).map_or(body.len(), |(_, r)| r.start);
        out.push(Section {
            title: title.trim().to_string(),
            heading: heading.clone(),
            content: start..end,
        });
    }
    out
}

fn find<'a>(secs: &'a [Section], title: &str) -> Option<&'a Section> {
    secs.iter().find(|s| s.title.eq_ignore_ascii_case(title))
}

/// Characters as platforms count them (code points, not bytes).
pub fn charcount(s: &str) -> usize {
    s.chars().count()
}

pub fn section_text(body: &str, title: &str) -> Option<String> {
    let secs = sections(body);
    find(&secs, title).map(|s| body[s.content.clone()].trim().to_string())
}

/// The text to copy when posting: the `## <Network>` section if present,
/// otherwise the whole body minus the `## Notas` section.
pub fn post_text(body: &str, network_names: &[&str]) -> String {
    let secs = sections(body);
    for name in network_names {
        if let Some(s) = find(&secs, name) {
            return body[s.content.clone()].trim().to_string();
        }
    }
    match find(&secs, "Notas") {
        Some(notes) => {
            let mut text = body[..notes.heading.start].to_string();
            text.push_str(&body[notes.content.end..]);
            text.trim().to_string()
        }
        None => body.trim().to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionMode {
    Replace,
    Append,
}

/// Replaces or appends to `## title`, creating the section at the end if missing.
pub fn set_section(body: &str, title: &str, text: &str, mode: SectionMode) -> String {
    let secs = sections(body);
    let text = text.trim();
    match find(&secs, title) {
        Some(s) => {
            let existing = body[s.content.clone()].trim();
            let new_content = match mode {
                SectionMode::Append if !existing.is_empty() => format!("{existing}\n\n{text}"),
                _ => text.to_string(),
            };
            let rest = &body[s.content.end..];
            let sep = if rest.is_empty() { "\n" } else { "\n\n" };
            format!(
                "{}\n\n{new_content}{sep}{rest}",
                body[..s.content.start].trim_end_matches('\n'),
            )
        }
        None => {
            let mut out = body.trim_end().to_string();
            if !out.is_empty() {
                out.push_str("\n\n");
            } else {
                out.push('\n');
            }
            out.push_str(&format!("## {title}\n\n{text}\n"));
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = "\n## LinkedIn\n\nTexto do post.\n\n## Instagram\n\nLegenda.\n\n## Notas\n\nFontes.\n";

    #[test]
    fn finds_sections() {
        assert_eq!(section_text(BODY, "linkedin").as_deref(), Some("Texto do post."));
        assert_eq!(section_text(BODY, "Notas").as_deref(), Some("Fontes."));
        assert_eq!(section_text("```\n## not\n```\n", "not"), None);
    }

    #[test]
    fn post_text_rules() {
        assert_eq!(post_text(BODY, &["Instagram"]), "Legenda.");
        assert_eq!(
            post_text("Olá mundo\n\n## Notas\n\nprivado\n", &["LinkedIn"]),
            "Olá mundo"
        );
    }

    #[test]
    fn checklist_counts() {
        let c = checklist("- [x] a\n- [ ] b\n- [X] c\n");
        assert_eq!(c, Checklist { done: 2, total: 3 });
    }

    #[test]
    fn set_section_replace_append_create() {
        let b = set_section(BODY, "Instagram", "Nova legenda", SectionMode::Replace);
        assert_eq!(section_text(&b, "Instagram").as_deref(), Some("Nova legenda"));
        assert_eq!(section_text(&b, "LinkedIn").as_deref(), Some("Texto do post."));
        assert_eq!(section_text(&b, "Notas").as_deref(), Some("Fontes."));
        let b = set_section(&b, "Notas", "Mais", SectionMode::Append);
        assert_eq!(section_text(&b, "Notas").as_deref(), Some("Fontes.\n\nMais"));
        let b = set_section("", "Rascunho", "x", SectionMode::Replace);
        assert_eq!(b, "\n## Rascunho\n\nx\n");
    }
}
