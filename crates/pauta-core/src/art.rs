//! Artwork for posts (see docs/adr/0007-arte-com-claude-design.md).
//!
//! Pauta does not talk to Claude Design: there is no API to call. What it does is
//! make the hand-off cheap and visible: a brief ready to paste, a status on the card,
//! the exported files kept next to it, and a check that a post is ready to go out.

use serde::Serialize;

use crate::config::{MediaFormat, NetworkConfig};
use crate::error::{Result, rule};
use crate::markdown::{charcount, post_text, section_text};
use crate::model::{ArtStatus, Card};
use crate::store::Index;

/// One reason a post is not ready to go out. `code` is stable (the UI translates it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    /// `no_text`, `over_limit`, `media_missing`, `art_pending`.
    pub code: &'static str,
    pub detail: Option<String>,
}

fn network_names(index: &Index, card: &Card) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(n) = card.network.as_deref() {
        if let Some(cfg) = index.config.network(n) {
            names.push(cfg.label.clone());
        }
        names.push(n.to_string());
    }
    names
}

/// The text to paste into the network: the network's own section, or the body minus notes.
/// A card's `## Arte` section is a brief for the design tool, never post text.
pub fn text_for_posting(index: &Index, card: &Card) -> String {
    let names = network_names(index, card);
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    post_text(&strip_section(&card.body, "Arte"), &refs)
}

fn strip_section(body: &str, title: &str) -> String {
    let secs = crate::markdown::sections(body);
    match secs.iter().find(|s| s.title.eq_ignore_ascii_case(title)) {
        Some(s) => format!("{}{}", &body[..s.heading.start], &body[s.content.end..]),
        None => body.to_string(),
    }
}

/// What still stands between this card and "post it now". Empty means ready.
pub fn readiness(index: &Index, card: &Card) -> Vec<Problem> {
    let Some(net) = card.network.as_deref().and_then(|n| index.config.network(n)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let text = text_for_posting(index, card);
    if text.trim().is_empty() {
        out.push(Problem {
            code: "no_text",
            detail: None,
        });
    } else if let Some(limit) = net.char_limit {
        let n = charcount(&text);
        if n > limit as usize {
            out.push(Problem {
                code: "over_limit",
                detail: Some(format!("{n}/{limit}")),
            });
        }
    }
    let art = card.art.as_ref().map(|a| a.status);
    match art {
        Some(ArtStatus::Requested) => out.push(Problem {
            code: "art_pending",
            // Files already in the folder just need to be marked as ready.
            detail: Some(
                if card.assets.is_empty() {
                    "waiting"
                } else {
                    "files_present"
                }
                .into(),
            ),
        }),
        _ if net.media.requires_media && card.assets.is_empty() => out.push(Problem {
            code: "media_missing",
            detail: None,
        }),
        _ => {}
    }
    out
}

/// The format chosen for the card (or the network's first one).
pub fn format_for<'a>(net: &'a NetworkConfig, card: &Card) -> Option<&'a MediaFormat> {
    card.art
        .as_ref()
        .and_then(|a| a.format.as_deref())
        .and_then(|id| net.media.formats.iter().find(|f| f.id == id))
        .or_else(|| net.media.formats.first())
}

/// The brief to paste into the design tool.
pub fn request_text(index: &Index, card: &Card) -> Result<String> {
    let Some(net) = card.network.as_deref().and_then(|n| index.config.network(n)) else {
        return rule("o cartão precisa de uma rede para pedir a arte");
    };
    let format = format_for(net, card);
    let post = text_for_posting(index, card);
    let direction = section_text(&card.body, "Arte").filter(|d| !d.trim().is_empty());
    let brand = index.config.art.brand.as_deref();

    let mut out = format!("Preciso da arte para um post do {}.\n\n", net.label);
    match format {
        Some(f) => {
            out.push_str(&format!("Formato: {} — {} px", f.label, f.size));
            if let Some(note) = &f.note {
                out.push_str(&format!(" ({note})"));
            }
            out.push_str(".\n\n");
        }
        None => out.push_str("Formato: a combinar.\n\n"),
    }
    out.push_str("Texto do post (a arte deve conversar com ele, sem repeti-lo palavra por palavra):\n\"\"\"\n");
    out.push_str(if post.trim().is_empty() {
        "(ainda sem texto)"
    } else {
        post.trim()
    });
    out.push_str("\n\"\"\"\n\nDireção visual:\n");
    out.push_str(
        direction
            .as_deref()
            .unwrap_or("(sem direção específica: proponha duas alternativas curtas e escolha a mais legível)"),
    );
    out.push_str("\n\nIdentidade visual:\n");
    out.push_str(brand.unwrap_or("(sem notas de identidade: visual sóbrio, limpo, com boa leitura em tela pequena)"));
    out.push_str("\n\nEntrega:\n");
    if let Some(f) = format {
        out.push_str(&format!("- Exportar em {} px.\n", f.size));
    }
    out.push_str(
        "- Texto legível em tela de celular e contraste alto; nada de informação só por cor.\n\
- Um texto alternativo curto (alt text) para cada imagem.\n\
- Um arquivo por slide, nomeados 01, 02, 03…; se houver mais de um formato, indique qual é qual.\n\
- Se não for possível exportar no formato pedido, exporte em PDF ou .zip e me diga.\n",
    );
    Ok(out)
}
