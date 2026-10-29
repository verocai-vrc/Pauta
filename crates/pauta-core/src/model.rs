//! Boards, cards and calendar events as read from disk (sections 5.3 to 5.5).

use std::path::PathBuf;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::{Mapping, Value};

use crate::fields::*;
use crate::markdown::{Checklist, checklist};
use crate::time::{When, parse_instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BoardKind {
    Content,
    Project,
    Generic,
}

impl BoardKind {
    pub fn parse(s: &str) -> BoardKind {
        match s {
            "content" => BoardKind::Content,
            "project" => BoardKind::Project,
            _ => BoardKind::Generic,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BoardKind::Content => "content",
            BoardKind::Project => "project",
            BoardKind::Generic => "generic",
        }
    }

    /// Default `kind` of new cards on this board.
    pub fn default_card_kind(self) -> &'static str {
        match self {
            BoardKind::Content => "post",
            _ => "task",
        }
    }
}

/// What a column means to the scheduling rules. Columns are free-form, so the
/// rules look at roles, never at column ids or names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnRole {
    /// Where quick captures land.
    Inbox,
    /// Where a card goes when a publication is scheduled.
    Scheduled,
    /// Finished: published / done. Items here never count as overdue.
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub role: Option<ColumnRole>,
    /// Optional work-in-progress limit (section 6.3).
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub kind: BoardKind,
    pub color: Option<String>,
    pub columns: Vec<Column>,
    pub dir: PathBuf,
    pub hash: String,
}

impl Board {
    pub fn column(&self, id: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.id == id)
    }

    pub fn column_with_role(&self, role: ColumnRole) -> Option<&Column> {
        self.columns.iter().find(|c| c.role == Some(role))
    }

    /// The inbox column, falling back to the first column.
    pub fn inbox(&self) -> Option<&Column> {
        self.column_with_role(ColumnRole::Inbox).or(self.columns.first())
    }

    /// The done column, falling back to the last column.
    pub fn done(&self) -> Option<&Column> {
        self.column_with_role(ColumnRole::Done).or(self.columns.last())
    }

    pub fn is_done_column(&self, id: &str) -> bool {
        self.done().is_some_and(|c| c.id == id)
    }

    pub fn column_index(&self, id: &str) -> usize {
        self.columns.iter().position(|c| c.id == id).unwrap_or(0)
    }

    pub(crate) fn from_mapping(m: &Mapping, dir_name: &str, dir: PathBuf, hash: String) -> Result<Board, String> {
        let columns = match m.get("columns") {
            Some(Value::Sequence(items)) => items
                .iter()
                .filter_map(Value::as_mapping)
                .filter_map(|c| {
                    let id = get_str(c, "id")?;
                    Some(Column {
                        name: get_str(c, "name").unwrap_or_else(|| id.clone()),
                        role: get_str(c, "role").and_then(|r| match r.as_str() {
                            "inbox" => Some(ColumnRole::Inbox),
                            "scheduled" => Some(ColumnRole::Scheduled),
                            "done" => Some(ColumnRole::Done),
                            _ => None,
                        }),
                        limit: get_i64(c, "limit").map(|l| l as u32),
                        id,
                    })
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        if columns.is_empty() {
            return Err("o quadro não tem colunas".into());
        }
        Ok(Board {
            // The folder name is the identity: every path is built from it, so an
            // `id:` that disagrees (folder renamed by hand) must not win.
            id: dir_name.to_string(),
            name: get_str(m, "name").unwrap_or_else(|| dir_name.to_string()),
            kind: BoardKind::parse(&get_str(m, "kind").unwrap_or_default()),
            color: get_str(m, "color"),
            columns,
            dir,
            hash,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PubStatus {
    Planned,
    Scheduled,
    Published,
    Skipped,
}

impl PubStatus {
    pub fn parse(s: &str) -> Option<PubStatus> {
        Some(match s {
            "planned" => PubStatus::Planned,
            "scheduled" => PubStatus::Scheduled,
            "published" => PubStatus::Published,
            "skipped" => PubStatus::Skipped,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PubStatus::Planned => "planned",
            PubStatus::Scheduled => "scheduled",
            PubStatus::Published => "published",
            PubStatus::Skipped => "skipped",
        }
    }

    /// Still waiting to be posted.
    pub fn pending(self) -> bool {
        matches!(self, PubStatus::Planned | PubStatus::Scheduled)
    }
}

/// The single publication of a card (one card per network, see ADR 0004).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Publication {
    pub at: Option<When>,
    pub status: PubStatus,
    pub url: Option<String>,
    pub published_at: Option<Timestamp>,
}

/// Numbers the author copies from the network's own analytics after a post has run
/// (no platform API is involved). Every field is optional: record what you have.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metrics {
    pub impressions: Option<u64>,
    pub reactions: Option<u64>,
    pub comments: Option<u64>,
    pub shares: Option<u64>,
    pub saves: Option<u64>,
    pub clicks: Option<u64>,
    /// When the numbers were copied, so old snapshots are recognizable.
    #[serde(default)]
    pub recorded: Option<jiff::civil::Date>,
}

impl Metrics {
    pub(crate) fn from_value(v: Option<&Value>) -> Option<Metrics> {
        let m = v?.as_mapping()?;
        let n = |k: &str| get_i64(m, k).and_then(|v| u64::try_from(v).ok());
        let metrics = Metrics {
            impressions: n("impressions"),
            reactions: n("reactions"),
            comments: n("comments"),
            shares: n("shares"),
            saves: n("saves"),
            clicks: n("clicks"),
            recorded: get_str(m, "recorded").and_then(|s| s.parse().ok()),
        };
        metrics.has_numbers().then_some(metrics)
    }

    pub fn has_numbers(&self) -> bool {
        [
            self.impressions,
            self.reactions,
            self.comments,
            self.shares,
            self.saves,
            self.clicks,
        ]
        .iter()
        .any(Option::is_some)
    }

    /// Reactions, comments, shares and saves: what people did with the post.
    pub fn engagement(&self) -> u64 {
        [self.reactions, self.comments, self.shares, self.saves]
            .iter()
            .flatten()
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtStatus {
    /// Ordered from the design tool, not back yet.
    Requested,
    /// The files are in the card's attachments folder.
    Ready,
}

impl ArtStatus {
    pub fn parse(s: &str) -> Option<ArtStatus> {
        match s {
            "requested" => Some(ArtStatus::Requested),
            "ready" => Some(ArtStatus::Ready),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ArtStatus::Requested => "requested",
            ArtStatus::Ready => "ready",
        }
    }
}

/// Where the artwork for a post stands. The brief itself is the `## Arte` section of
/// the body; the files are in `assets/<card id>/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Art {
    pub status: ArtStatus,
    /// A format id of the card's network (`image`, `carousel`, ...).
    pub format: Option<String>,
    pub requested_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Links {
    /// Related project board.
    pub board: Option<String>,
    /// Ids of the same idea's cards for other networks.
    pub siblings: Vec<String>,
    /// For promo posts: the article card being promoted.
    pub article: Option<String>,
}

impl Links {
    fn from_value(v: Option<&Value>) -> Links {
        let Some(m) = v.and_then(Value::as_mapping) else {
            return Links::default();
        };
        Links {
            board: get_str(m, "board"),
            siblings: get_list(m, "siblings"),
            article: get_str(m, "article"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Card {
    pub id: String,
    pub board: String,
    pub title: String,
    pub kind: String,
    pub column: String,
    pub rank: String,
    pub tags: Vec<String>,
    pub network: Option<String>,
    pub publication: Option<Publication>,
    pub due: Option<When>,
    pub reminders: Option<Vec<String>>,
    pub links: Links,
    pub created: Option<Timestamp>,
    pub updated: Option<Timestamp>,
    pub metrics: Option<Metrics>,
    pub art: Option<Art>,
    /// File names in the card's attachments folder, sorted.
    pub assets: Vec<String>,
    /// When the card entered its current column. `None` for cards written before
    /// this field existed or moved by hand (the field names the column it refers to).
    pub column_since: Option<Timestamp>,
    pub body: String,
    pub checklist: Checklist,
    pub path: PathBuf,
    pub hash: String,
}

/// Visible files in `<board>/assets/<card id>/`.
pub(crate) fn list_assets(board_dir: &std::path::Path, card_id: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(board_dir.join("assets").join(card_id))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect();
    names.sort();
    names
}

fn opt_instant(m: &Mapping, k: &str, tz: &TimeZone) -> Option<Timestamp> {
    get_str(m, k).and_then(|s| parse_instant(&s, tz).ok())
}

fn opt_when(m: &Mapping, k: &str, tz: &TimeZone) -> Result<Option<When>, String> {
    match get_str(m, k) {
        Some(s) => When::parse(&s, tz).map(Some).map_err(|e| format!("{k}: {e}")),
        None => Ok(None),
    }
}

fn required(m: &Mapping, k: &str) -> Result<String, String> {
    get_str(m, k)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("campo obrigatório ausente: {k}"))
}

impl Card {
    pub(crate) fn from_parts(
        m: &Mapping,
        body: &str,
        board: &Board,
        path: PathBuf,
        hash: String,
        tz: &TimeZone,
    ) -> Result<Card, String> {
        let publication = match m.get("publication") {
            Some(Value::Mapping(p)) => Some(Publication {
                at: opt_when(p, "at", tz)?,
                status: get_str(p, "status")
                    .and_then(|s| PubStatus::parse(&s))
                    .unwrap_or(PubStatus::Planned),
                url: get_str(p, "url"),
                published_at: opt_instant(p, "published_at", tz),
            }),
            _ => None,
        };
        let column = get_str(m, "column")
            .or_else(|| board.inbox().map(|c| c.id.clone()))
            .unwrap_or_default();
        let column_since = m
            .get("column_since")
            .and_then(Value::as_mapping)
            .filter(|s| get_str(s, "column").as_deref() == Some(column.as_str()))
            .and_then(|s| opt_instant(s, "at", tz));
        let id = required(m, "id")?;
        Ok(Card {
            id: id.clone(),
            title: required(m, "title")?,
            board: board.id.clone(),
            kind: get_str(m, "kind").unwrap_or_else(|| board.kind.default_card_kind().into()),
            column,
            column_since,
            rank: get_str(m, "rank").unwrap_or_default(),
            tags: get_list(m, "tags"),
            network: get_str(m, "network"),
            publication,
            due: opt_when(m, "due", tz)?,
            reminders: get_opt_list(m, "reminders"),
            links: Links::from_value(m.get("links")),
            created: opt_instant(m, "created", tz),
            updated: opt_instant(m, "updated", tz),
            metrics: Metrics::from_value(m.get("metrics")),
            art: m.get("art").and_then(Value::as_mapping).and_then(|a| {
                Some(Art {
                    status: ArtStatus::parse(&get_str(a, "status")?)?,
                    format: get_str(a, "format"),
                    requested_at: opt_instant(a, "requested_at", tz),
                })
            }),
            assets: list_assets(&board.dir, &id),
            checklist: checklist(body),
            body: body.to_string(),
            path,
            hash,
        })
    }

    pub fn pending_publication_at(&self) -> Option<&When> {
        let p = self.publication.as_ref()?;
        p.status.pending().then_some(p.at.as_ref()).flatten()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    /// `deadline | event | milestone | launch`.
    pub kind: String,
    pub when: When,
    pub end: Option<When>,
    pub reminders: Option<Vec<String>>,
    pub links: Links,
    pub tags: Vec<String>,
    pub body: String,
    pub path: PathBuf,
    pub hash: String,
}

impl Event {
    pub(crate) fn from_parts(
        m: &Mapping,
        body: &str,
        path: PathBuf,
        hash: String,
        tz: &TimeZone,
    ) -> Result<Event, String> {
        let when = match (opt_when(m, "date", tz)?, opt_when(m, "start", tz)?) {
            (Some(w), _) | (None, Some(w)) => w,
            (None, None) => return Err("o evento precisa de 'date' ou 'start'".into()),
        };
        Ok(Event {
            id: required(m, "id")?,
            title: required(m, "title")?,
            kind: get_str(m, "kind").unwrap_or_else(|| "event".into()),
            when,
            end: opt_when(m, "end", tz)?,
            reminders: get_opt_list(m, "reminders"),
            links: Links::from_value(m.get("links")),
            tags: get_list(m, "tags"),
            body: body.to_string(),
            path,
            hash,
        })
    }
}

/// A file that could not be read. Shown in the UI, never rewritten automatically
/// (section 14.6).
#[derive(Debug, Clone, Serialize)]
pub struct InvalidFile {
    pub path: PathBuf,
    pub error: String,
}
