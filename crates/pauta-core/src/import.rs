//! Metrics from a CSV the author exported from a network's own analytics (section 17.1,
//! the alternative that needs no API). The exports' columns differ between networks and
//! change over time, so headers are matched by name (Portuguese or English) rather than
//! by a fixed layout, and the result is previewed before anything is written.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::{Error, Result};
use crate::model::{Card, Metrics, PubStatus};
use crate::store::{Index, MetricsInput};

const FIELDS: [&str; 6] = ["impressions", "reactions", "comments", "shares", "saves", "clicks"];

/// Header names (lowercase, accents and punctuation removed) per field, best choice first.
/// `reach` comes after `impressions` and is only used when nothing better is present;
/// the preview shows which column fed which field so a substitution is never silent.
const SYNONYMS: [(&str, &[&str]); 7] = [
    (
        "url",
        &[
            "url",
            "link",
            "postlink",
            "posturl",
            "permalink",
            "linkdapublicacao",
            "linkdopost",
            "urldapublicacao",
            "urldopost",
        ],
    ),
    (
        "impressions",
        &[
            "impressions",
            "impressoes",
            "views",
            "visualizacoes",
            "reach",
            "alcance",
        ],
    ),
    ("reactions", &["reactions", "reacoes", "likes", "curtidas"]),
    ("comments", &["comments", "comentarios"]),
    ("shares", &["shares", "compartilhamentos", "reposts", "republicacoes"]),
    ("saves", &["saves", "salvamentos", "salvos"]),
    ("clicks", &["clicks", "cliques", "linkclicks"]),
];

#[derive(Debug, Clone, Serialize)]
pub struct Row {
    /// 1-based line in the file (the header is line 1).
    pub line: usize,
    pub url: Option<String>,
    #[serde(skip)]
    pub metrics: MetricsInput,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnMap {
    pub field: String,
    pub header: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Parsed {
    pub columns: Vec<ColumnMap>,
    #[serde(skip)]
    pub rows: Vec<Row>,
    /// Cells that were not whole numbers (`1,5 mil`, `12%`) and were left out: (line, header, text).
    pub skipped_cells: Vec<(usize, String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub line: usize,
    pub card_id: String,
    pub title: String,
    pub network: Option<String>,
    pub before: Option<Metrics>,
    /// What the card will have afterwards: the file's numbers over the existing ones.
    pub after: Metrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct Preview {
    pub columns: Vec<ColumnMap>,
    pub matched: Vec<Match>,
    /// Rows whose URL matched no published card (line, url as written).
    pub unmatched: Vec<(usize, String)>,
    pub skipped_cells: Vec<(usize, String, String)>,
}

fn norm(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            let c = match c {
                'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' => 'a',
                'é' | 'ê' | 'è' | 'É' | 'Ê' => 'e',
                'í' | 'ì' | 'Í' => 'i',
                'ó' | 'ô' | 'õ' | 'ò' | 'Ó' | 'Ô' | 'Õ' => 'o',
                'ú' | 'ù' | 'ü' | 'Ú' => 'u',
                'ç' | 'Ç' => 'c',
                c => c,
            };
            c.is_alphanumeric().then(|| c.to_ascii_lowercase())
        })
        .collect()
}

/// `https://www.linkedin.com/posts/x/?utm=1#a` and `linkedin.com/posts/x` are the same post.
pub fn normalize_url(u: &str) -> String {
    let u = u.trim().to_lowercase();
    let u = u.split(['?', '#']).next().unwrap_or("");
    let u = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))
        .unwrap_or(u);
    let u = u.strip_prefix("www.").unwrap_or(u);
    u.trim_end_matches('/').to_string()
}

/// A whole number, tolerating `1.234`, `1,234` and spaces as thousands separators.
fn number(cell: &str) -> Option<u64> {
    let digits: String = cell
        .chars()
        .filter(|c| !matches!(c, '.' | ',' | ' ' | '\u{a0}'))
        .collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

pub fn parse(text: &str) -> Result<Parsed> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let first = text.lines().next().unwrap_or("");
    let delim = b",;\t"
        .iter()
        .copied()
        .max_by_key(|d| first.bytes().filter(|b| b == d).count())
        .unwrap_or(b',');
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .flexible(true)
        .from_reader(text.as_bytes());
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| Error::Rule(format!("CSV ilegível: {e}")))?
        .iter()
        .map(str::to_string)
        .collect();

    // field -> column index, taking the first synonym that has a column.
    let mut cols: BTreeMap<&str, usize> = BTreeMap::new();
    for (field, names) in SYNONYMS {
        'names: for name in names {
            for (i, h) in headers.iter().enumerate() {
                if norm(h) == *name {
                    cols.insert(field, i);
                    break 'names;
                }
            }
        }
    }
    if !cols.contains_key("url") {
        return Err(Error::Rule(
            "não achei a coluna com o link do post (esperado: URL, Link ou Permalink)".into(),
        ));
    }
    if !FIELDS.iter().any(|f| cols.contains_key(f)) {
        return Err(Error::Rule(
            "não achei nenhuma coluna de resultados (impressões, reações, comentários, compartilhamentos, salvamentos ou cliques)".into(),
        ));
    }

    let mut rows = Vec::new();
    let mut skipped = Vec::new();
    for (n, rec) in rdr.records().enumerate() {
        let rec = rec.map_err(|e| Error::Rule(format!("CSV ilegível: {e}")))?;
        let line = n + 2;
        let url = rec
            .get(cols["url"])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let mut m = MetricsInput::default();
        for field in FIELDS {
            let Some(&i) = cols.get(field) else { continue };
            let cell = rec.get(i).unwrap_or("").trim();
            if cell.is_empty() {
                continue;
            }
            let Some(v) = number(cell) else {
                skipped.push((line, headers[i].clone(), cell.to_string()));
                continue;
            };
            match field {
                "impressions" => m.impressions = Some(v),
                "reactions" => m.reactions = Some(v),
                "comments" => m.comments = Some(v),
                "shares" => m.shares = Some(v),
                "saves" => m.saves = Some(v),
                _ => m.clicks = Some(v),
            }
        }
        if url.is_some() || has_any(&m) {
            rows.push(Row { line, url, metrics: m });
        }
    }
    let columns = SYNONYMS
        .iter()
        .filter_map(|(f, _)| {
            cols.get(f).map(|&i| ColumnMap {
                field: (*f).to_string(),
                header: headers[i].clone(),
            })
        })
        .collect();
    Ok(Parsed {
        columns,
        rows,
        skipped_cells: skipped,
    })
}

fn has_any(m: &MetricsInput) -> bool {
    m.impressions.is_some()
        || m.reactions.is_some()
        || m.comments.is_some()
        || m.shares.is_some()
        || m.saves.is_some()
        || m.clicks.is_some()
}

/// The file's numbers laid over what the card already has; empty cells keep the old number.
pub fn merged(before: Option<&Metrics>, new: &MetricsInput) -> MetricsInput {
    let b = before.cloned().unwrap_or_default();
    MetricsInput {
        impressions: new.impressions.or(b.impressions),
        reactions: new.reactions.or(b.reactions),
        comments: new.comments.or(b.comments),
        shares: new.shares.or(b.shares),
        saves: new.saves.or(b.saves),
        clicks: new.clicks.or(b.clicks),
    }
}

fn is_published_with(c: &Card, key: &str) -> bool {
    c.publication
        .as_ref()
        .is_some_and(|p| p.status == PubStatus::Published && p.url.as_deref().is_some_and(|u| normalize_url(u) == key))
}

/// Which published card each row belongs to. Nothing is written.
pub fn plan(index: &Index, parsed: &Parsed, today: jiff::civil::Date) -> Preview {
    let mut matched = Vec::new();
    let mut unmatched = Vec::new();
    for row in &parsed.rows {
        let Some(url) = &row.url else {
            continue;
        };
        if !has_any(&row.metrics) {
            continue;
        }
        let key = normalize_url(url);
        match index.cards.iter().find(|c| is_published_with(c, &key)) {
            Some(c) => {
                let m = merged(c.metrics.as_ref(), &row.metrics);
                matched.push(Match {
                    line: row.line,
                    card_id: c.id.clone(),
                    title: c.title.clone(),
                    network: c.network.clone(),
                    before: c.metrics.clone(),
                    after: Metrics {
                        impressions: m.impressions,
                        reactions: m.reactions,
                        comments: m.comments,
                        shares: m.shares,
                        saves: m.saves,
                        clicks: m.clicks,
                        recorded: Some(today),
                    },
                });
            }
            None => unmatched.push((row.line, url.clone())),
        }
    }
    Preview {
        columns: parsed.columns.clone(),
        matched,
        unmatched,
        skipped_cells: parsed.skipped_cells.clone(),
    }
}
