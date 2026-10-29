//! The data folder: reading it into an [`Index`] and every write operation.
//!
//! Both the app and the MCP server go through [`Workspace`], so each business
//! rule lives here exactly once (invariant 2).

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::{Mapping, Value};

use crate::clock::{Clock, SystemClock};
use crate::config::{Config, ConfigPatch, default_config_text, patch_blocks};
use crate::error::{Error, IoContext, Result, rule};
use crate::fields::{double_option, merge_map};
use crate::frontmatter::Doc;
use crate::markdown::{SectionMode, set_section};
use crate::model::*;
use crate::rank;
use crate::templates::{PALETTE, Seed, columns_for};
use crate::time::{When, format_instant};
use crate::yaml::{Y, YamlText};

pub const CONFIG_FILE: &str = "pauta.yaml";

/// Short content hash used for conflict detection (section 14.2).
pub fn hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex()[..16].to_string()
}

/// `"Entrega do capítulo 3!"` -> `"entrega-do-capitulo-3"`.
pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        let c = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let mut out = out.trim_end_matches('-').to_string();
    if out.len() > 60 {
        out.truncate(60);
        out = out.trim_end_matches('-').to_string();
    }
    if out.is_empty() { "item".into() } else { out }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).at(path)
}

/// Writes through a temporary file in the same folder, then renames it over the
/// target, so readers see either the old or the new file, never half of one.
pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir).at(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut tmp = tempfile::Builder::new()
        .prefix(&format!(".{name}."))
        .suffix(".tmp")
        .tempfile_in(dir)
        .at(dir)?;
    tmp.write_all(content.as_bytes()).at(path)?;
    tmp.as_file().sync_all().at(path)?;
    tmp.persist(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e.error,
    })?;
    Ok(())
}

fn md_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().is_some_and(|e| e == "md")
                && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))
        })
        .collect();
    out.sort();
    out
}

/// Last six characters of the id, lowercased. Counts characters, not bytes: ids
/// come from files and MCP clients, so they can hold multi-byte text.
fn id_suffix(id: &str) -> String {
    let chars: Vec<char> = id.to_lowercase().chars().collect();
    chars[chars.len().saturating_sub(6)..].iter().collect()
}

pub fn new_id() -> String {
    ulid::Ulid::new().to_string()
}

/// Everything in the data folder, read into memory (section 14.5).
#[derive(Debug, Clone, Serialize)]
pub struct Index {
    pub config: Config,
    pub boards: Vec<Board>,
    pub cards: Vec<Card>,
    pub events: Vec<Event>,
    pub invalid: Vec<InvalidFile>,
    /// Published posts, active and archived, for cadence and history. Not sent to the UI.
    #[serde(skip)]
    pub published: Vec<crate::rhythm::PublishedPost>,
    /// What stands between each pending post and "post it now", by card id. Cards that are ready are absent.
    pub readiness: std::collections::BTreeMap<String, Vec<crate::art::Problem>>,
}

impl Index {
    pub fn board(&self, id: &str) -> Option<&Board> {
        self.boards.iter().find(|b| b.id == id)
    }

    pub fn card(&self, id: &str) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
    }

    pub fn event(&self, id: &str) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }

    pub fn column_cards(&self, board: &str, column: &str) -> Vec<&Card> {
        self.cards
            .iter()
            .filter(|c| c.board == board && c.column == column)
            .collect()
    }

    pub fn is_done(&self, card: &Card) -> bool {
        self.board(&card.board).is_some_and(|b| b.is_done_column(&card.column))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewCard {
    pub title: String,
    pub column: Option<String>,
    pub kind: Option<String>,
    pub network: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub body: Option<String>,
    pub due: Option<String>,
    pub publication_at: Option<String>,
    pub reminders: Option<Vec<String>>,
    pub links_board: Option<String>,
    pub links_article: Option<String>,
    /// Insert at the top of the column instead of the bottom.
    #[serde(default)]
    pub top: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PublicationPatch {
    #[serde(default, with = "double_option")]
    pub at: Option<Option<String>>,
    pub status: Option<String>,
    #[serde(default, with = "double_option")]
    pub url: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SectionEdit {
    pub title: String,
    pub text: String,
    pub mode: SectionMode,
}

/// A partial update. `None` = leave alone; for nullable fields, `Some(None)` clears.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CardPatch {
    pub title: Option<String>,
    pub kind: Option<String>,
    pub column: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(default, with = "double_option")]
    pub network: Option<Option<String>>,
    #[serde(default, with = "double_option")]
    pub due: Option<Option<String>>,
    #[serde(default, with = "double_option")]
    pub reminders: Option<Option<Vec<String>>>,
    #[serde(default, with = "double_option")]
    pub publication: Option<Option<PublicationPatch>>,
    #[serde(default, with = "double_option")]
    pub links_board: Option<Option<String>>,
    /// Numbers copied from the network's analytics; `Some(None)` removes them.
    #[serde(default, with = "double_option")]
    pub metrics: Option<Option<MetricsInput>>,
    pub body: Option<String>,
    pub section: Option<SectionEdit>,
}

/// Metrics as typed in: the whole block is replaced, empty fields are left out.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetricsInput {
    pub impressions: Option<u64>,
    pub reactions: Option<u64>,
    pub comments: Option<u64>,
    pub shares: Option<u64>,
    pub saves: Option<u64>,
    pub clicks: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewEvent {
    pub title: String,
    pub kind: Option<String>,
    /// `2026-12-05` (all day) or `2026-12-05T14:00` (timed).
    pub date: String,
    pub end: Option<String>,
    pub reminders: Option<Vec<String>>,
    pub board: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventPatch {
    pub title: Option<String>,
    pub kind: Option<String>,
    pub date: Option<String>,
    #[serde(default, with = "double_option")]
    pub end: Option<Option<String>>,
    #[serde(default, with = "double_option")]
    pub reminders: Option<Option<Vec<String>>>,
    #[serde(default, with = "double_option")]
    pub board: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BoardPatch {
    pub name: Option<String>,
    #[serde(default, with = "double_option")]
    pub color: Option<Option<String>>,
    pub columns: Option<Vec<Column>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrashEntry {
    /// Path inside `.trash/`, used to restore.
    pub path: PathBuf,
    /// Where it lived, relative to the data folder.
    pub original: PathBuf,
    pub title: String,
    pub deleted_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchivedItem {
    pub path: PathBuf,
    pub kind: String,
    pub board: Option<String>,
    pub title: String,
}

/// Where a card goes in its column, and which neighbours must be re-spaced first.
struct RankPlan {
    rank: String,
    /// `(path, hash when planned, new rank)`.
    respace: Vec<(PathBuf, String, String)>,
}

/// Handle to a data folder. Cheap to clone.
#[derive(Clone)]
pub struct Workspace {
    root: PathBuf,
    actor: String,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workspace")
            .field("root", &self.root)
            .field("actor", &self.actor)
            .finish()
    }
}

impl Workspace {
    /// Opens an existing data folder, migrating it if needed.
    pub fn open(root: impl Into<PathBuf>) -> Result<Workspace> {
        let ws = Workspace {
            root: root.into(),
            actor: "app".into(),
            clock: Arc::new(SystemClock),
        };
        let cfg = ws.root.join(CONFIG_FILE);
        if !cfg.exists() {
            return Err(Error::NotFound(format!(
                "pasta de dados do Pauta (sem {CONFIG_FILE}): {}",
                ws.root.display()
            )));
        }
        crate::migrate::migrate(&ws.root)?;
        Ok(ws)
    }

    /// Creates the folder structure and seed boards. Opens instead if it already exists.
    pub fn init(root: impl Into<PathBuf>, seed: &Seed) -> Result<Workspace> {
        let root = root.into();
        if root.join(CONFIG_FILE).exists() {
            return Workspace::open(root);
        }
        for d in ["boards", "calendar", "archive", ".trash", ".pauta"] {
            let p = root.join(d);
            fs::create_dir_all(&p).at(&p)?;
        }
        crate::time::zone(&seed.timezone)?;
        write_atomic(&root.join(CONFIG_FILE), &default_config_text(&seed.timezone, &[]))?;
        write_atomic(&root.join(".gitignore"), ".pauta/\n")?;
        let ws = Workspace::open(root)?;
        for (name, kind) in &seed.boards {
            ws.create_board(name, *kind, None)?;
        }
        Ok(ws)
    }

    /// Who is writing, recorded in `.pauta/log.jsonl` (e.g. `"app"`, `"mcp:pauta_move_card"`).
    pub fn with_actor(&self, actor: impl Into<String>) -> Workspace {
        Workspace {
            actor: actor.into(),
            ..self.clone()
        }
    }

    pub fn with_clock(&self, clock: Arc<dyn Clock>) -> Workspace {
        Workspace { clock, ..self.clone() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn now(&self) -> Timestamp {
        self.clock.now()
    }

    fn rel<'a>(&self, p: &'a Path) -> &'a Path {
        p.strip_prefix(&self.root).unwrap_or(p)
    }

    fn log(&self, op: &str, path: &Path) {
        let line = serde_json::json!({
            "at": self.now().to_string(),
            "actor": self.actor,
            "op": op,
            "path": self.rel(path),
        });
        let dir = self.root.join(".pauta");
        let _ = fs::create_dir_all(&dir);
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("log.jsonl"))
        {
            let _ = writeln!(f, "{line}");
        }
    }

    /// Writes `content` unless the file changed since it was read with `expected` hash.
    fn save(&self, path: &Path, expected: Option<&str>, content: &str, op: &str) -> Result<()> {
        if let Some(expected) = expected {
            let disk = read(path)?;
            let disk_hash = hash(&disk);
            if disk_hash != expected {
                return Err(Error::Conflict {
                    path: path.to_path_buf(),
                    disk_content: disk,
                    disk_hash,
                });
            }
        }
        write_atomic(path, content)?;
        self.log(op, path);
        Ok(())
    }

    /// Moves a file or folder without overwriting anything at the destination.
    fn relocate(&self, from: &Path, to: &Path, op: &str) -> Result<PathBuf> {
        let mut target = to.to_path_buf();
        let mut n = 2;
        while target.exists() {
            let stem = to
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let ext = to
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            target = to.with_file_name(format!("{stem}-{n}{ext}"));
            n += 1;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).at(parent)?;
        }
        fs::rename(from, &target).at(from)?;
        self.log(op, &target);
        Ok(target)
    }

    // ---------------------------------------------------------------- reading

    pub fn config(&self) -> Result<Config> {
        let path = self.root.join(CONFIG_FILE);
        let text = read(&path)?;
        let m = YamlText::new(text).parse().map_err(|message| Error::Invalid {
            path: path.clone(),
            message,
        })?;
        Config::from_mapping(&m).map_err(|message| Error::Invalid { path, message })
    }

    fn boards(&self, config: &Config, invalid: &mut Vec<InvalidFile>) -> Vec<Board> {
        let dir = self.root.join("boards");
        let mut boards = Vec::new();
        let Ok(entries) = fs::read_dir(&dir) else { return boards };
        for entry in entries.flatten() {
            let bdir = entry.path();
            let file = bdir.join("board.yaml");
            if !file.is_file() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().into_owned();
            match self.read_board_file(&file, &dir_name) {
                Ok(b) => boards.push(b),
                Err(e) => invalid.push(InvalidFile {
                    path: file,
                    error: e.to_string(),
                }),
            }
        }
        let pos = |id: &str| config.board_order.iter().position(|o| o == id).unwrap_or(usize::MAX);
        boards.sort_by(|a, b| pos(&a.id).cmp(&pos(&b.id)).then_with(|| a.name.cmp(&b.name)));
        boards
    }

    fn read_board_file(&self, file: &Path, dir_name: &str) -> Result<Board> {
        let text = read(file)?;
        let invalid = |message: String| Error::Invalid {
            path: file.to_path_buf(),
            message,
        };
        let m = YamlText::new(text.as_str()).parse().map_err(invalid)?;
        let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        Board::from_mapping(&m, dir_name, dir, hash(&text)).map_err(invalid)
    }

    fn board(&self, id: &str) -> Result<Board> {
        let file = self.root.join("boards").join(id).join("board.yaml");
        if !file.is_file() {
            return Err(Error::NotFound(format!("quadro '{id}'")));
        }
        self.read_board_file(&file, id)
    }

    fn read_card_file(&self, path: &Path, board: &Board, config: &Config) -> Result<(Card, Doc)> {
        let text = read(path)?;
        let doc = Doc::parse(&text);
        let invalid = |message: String| Error::Invalid {
            path: path.to_path_buf(),
            message,
        };
        let m = doc.fields().map_err(invalid)?;
        let card =
            Card::from_parts(&m, doc.body(), board, path.to_path_buf(), hash(&text), &config.tz).map_err(invalid)?;
        Ok((card, doc))
    }

    fn board_cards(&self, board: &Board, config: &Config, invalid: &mut Vec<InvalidFile>) -> Vec<Card> {
        let mut cards = Vec::new();
        for path in md_files(&board.dir.join("cards")) {
            match self.read_card_file(&path, board, config) {
                Ok((c, _)) => cards.push(c),
                Err(e) => invalid.push(InvalidFile {
                    path,
                    error: e.to_string(),
                }),
            }
        }
        cards.sort_by(|a, b| {
            board
                .column_index(&a.column)
                .cmp(&board.column_index(&b.column))
                .then_with(|| a.rank.cmp(&b.rank))
                .then_with(|| a.id.cmp(&b.id))
        });
        cards
    }

    fn read_event_file(&self, path: &Path, config: &Config) -> Result<(Event, Doc)> {
        let text = read(path)?;
        let doc = Doc::parse(&text);
        let invalid = |message: String| Error::Invalid {
            path: path.to_path_buf(),
            message,
        };
        let m = doc.fields().map_err(invalid)?;
        let ev = Event::from_parts(&m, doc.body(), path.to_path_buf(), hash(&text), &config.tz).map_err(invalid)?;
        Ok((ev, doc))
    }

    /// Reads the whole data folder. Broken files are listed, never fatal.
    pub fn load(&self) -> Result<Index> {
        let config = self.config()?;
        let mut invalid = Vec::new();
        let boards = self.boards(&config, &mut invalid);
        let mut cards = Vec::new();
        for b in &boards {
            cards.extend(self.board_cards(b, &config, &mut invalid));
        }
        let mut events = Vec::new();
        for path in md_files(&self.root.join("calendar")) {
            match self.read_event_file(&path, &config) {
                Ok((e, _)) => events.push(e),
                Err(e) => invalid.push(InvalidFile {
                    path,
                    error: e.to_string(),
                }),
            }
        }
        events.sort_by_key(|e| e.when.start(&config.tz));
        let mut published: Vec<_> = cards
            .iter()
            .filter_map(|c| crate::rhythm::PublishedPost::from_card(c, &config.tz, false))
            .collect();
        published.extend(self.archived_published(&config));
        let mut index = Index {
            config,
            boards,
            cards,
            events,
            invalid,
            published,
            readiness: Default::default(),
        };
        index.readiness = index
            .cards
            .iter()
            .filter(|c| {
                c.pending_publication_at().is_some() || c.publication.as_ref().is_some_and(|p| p.status.pending())
            })
            .filter_map(|c| {
                let problems = crate::art::readiness(&index, c);
                (!problems.is_empty()).then(|| (c.id.clone(), problems))
            })
            .collect();
        Ok(index)
    }

    /// Published posts whose cards were archived: archiving must not erase history.
    /// Files that cannot be read are skipped (they are not shown anywhere else either).
    fn archived_published(&self, config: &Config) -> Vec<crate::rhythm::PublishedPost> {
        let mut out = Vec::new();
        let Ok(boards) = fs::read_dir(self.root.join("archive").join("cards")) else {
            return out;
        };
        for b in boards.flatten() {
            let dir_name = b.file_name().to_string_lossy().into_owned();
            // The board may no longer exist, so read these cards against a stand-in.
            let stand_in = Board {
                id: dir_name.clone(),
                name: dir_name,
                kind: BoardKind::Generic,
                color: None,
                columns: columns_for(BoardKind::Generic),
                dir: b.path(),
                hash: String::new(),
            };
            for path in md_files(&b.path()) {
                if let Ok((card, _)) = self.read_card_file(&path, &stand_in, config)
                    && let Some(p) = crate::rhythm::PublishedPost::from_card(&card, &config.tz, true)
                {
                    out.push(p);
                }
            }
        }
        out
    }

    fn locate_card(&self, id: &str) -> Result<(PathBuf, Board)> {
        let config = self.config()?;
        let mut ignored = Vec::new();
        let boards = self.boards(&config, &mut ignored);
        let suffix = format!("--{}.md", id_suffix(id));
        // Fast path: the file name ends with the id's suffix.
        for b in &boards {
            for p in md_files(&b.dir.join("cards")) {
                if p.to_string_lossy().ends_with(&suffix)
                    && let Ok((c, _)) = self.read_card_file(&p, b, &config)
                    && c.id == id
                {
                    return Ok((p, b.clone()));
                }
            }
        }
        for b in &boards {
            for c in self.board_cards(b, &config, &mut ignored) {
                if c.id == id {
                    return Ok((c.path, b.clone()));
                }
            }
        }
        Err(Error::NotFound(format!("cartão '{id}'")))
    }

    pub fn card(&self, id: &str) -> Result<Card> {
        let (path, board) = self.locate_card(id)?;
        Ok(self.read_card_file(&path, &board, &self.config()?)?.0)
    }

    pub fn event(&self, id: &str) -> Result<Event> {
        Ok(self.locate_event(id)?.0)
    }

    fn locate_event(&self, id: &str) -> Result<(Event, Doc)> {
        let config = self.config()?;
        for p in md_files(&self.root.join("calendar")) {
            if let Ok((e, d)) = self.read_event_file(&p, &config)
                && e.id == id
            {
                return Ok((e, d));
            }
        }
        Err(Error::NotFound(format!("evento '{id}'")))
    }

    pub fn assets_dir(&self, card: &Card) -> PathBuf {
        self.root.join("boards").join(&card.board).join("assets").join(&card.id)
    }

    // ----------------------------------------------------------------- config

    pub fn update_config(&self, patch: &ConfigPatch) -> Result<Config> {
        let path = self.root.join(CONFIG_FILE);
        let text = read(&path)?;
        let mut y = YamlText::new(text.as_str());
        let current = y.parse().map_err(|message| Error::Invalid {
            path: path.clone(),
            message,
        })?;
        for (k, v) in patch_blocks(&current, patch)? {
            y.set(&k, &v);
        }
        self.save(&path, Some(&hash(&text)), y.as_str(), "update_config")?;
        self.config()
    }

    pub fn set_board_order(&self, order: &[String]) -> Result<()> {
        let path = self.root.join(CONFIG_FILE);
        let text = read(&path)?;
        let mut y = YamlText::new(text.as_str());
        y.set("board_order", &Y::str_list(order));
        self.save(&path, Some(&hash(&text)), y.as_str(), "set_board_order")
    }

    // ----------------------------------------------------------------- boards

    pub fn create_board(&self, name: &str, kind: BoardKind, color: Option<String>) -> Result<Board> {
        let name = name.trim();
        if name.is_empty() {
            return rule("o quadro precisa de um nome");
        }
        let base = slugify(name);
        let mut id = base.clone();
        let mut n = 2;
        while self.root.join("boards").join(&id).exists() || matches!(id.as_str(), "calendar" | "cards" | "boards") {
            id = format!("{base}-{n}");
            n += 1;
        }
        let config = self.config()?;
        let color = color.unwrap_or_else(|| PALETTE[config.board_order.len() % PALETTE.len()].into());
        let mut y = YamlText::new("");
        y.set("schema_version", &Y::Int(crate::config::SCHEMA_VERSION.into()));
        y.set("id", &Y::str(&id));
        y.set("name", &Y::Quoted(name.into()));
        y.set("kind", &Y::str(kind.as_str()));
        y.set("color", &Y::Quoted(color));
        y.set("columns", &columns_y(&columns_for(kind)));
        let dir = self.root.join("boards").join(&id);
        for sub in ["cards", "assets"] {
            fs::create_dir_all(dir.join(sub)).at(&dir)?;
        }
        let file = dir.join("board.yaml");
        self.save(&file, None, y.as_str(), "create_board")?;
        if !config.board_order.contains(&id) {
            let mut order = config.board_order.clone();
            order.push(id.clone());
            self.set_board_order(&order)?;
        }
        self.board(&id)
    }

    pub fn update_board(&self, id: &str, expected: Option<&str>, patch: &BoardPatch) -> Result<Board> {
        let board = self.board(id)?;
        let file = board.dir.join("board.yaml");
        let text = read(&file)?;
        let mut y = YamlText::new(text.as_str());
        if let Some(name) = &patch.name {
            if name.trim().is_empty() {
                return rule("o quadro precisa de um nome");
            }
            y.set("name", &Y::Quoted(name.trim().into()));
        }
        if let Some(color) = &patch.color {
            match color {
                Some(c) => y.set("color", &Y::Quoted(c.clone())),
                None => y.remove("color"),
            }
        }
        if let Some(columns) = &patch.columns {
            if columns.is_empty() {
                return rule("o quadro precisa de pelo menos uma coluna");
            }
            let mut seen = std::collections::HashSet::new();
            for c in columns {
                if c.id.trim().is_empty() || !seen.insert(c.id.as_str()) {
                    return rule(format!("identificador de coluna vazio ou repetido: '{}'", c.id));
                }
            }
            let config = self.config()?;
            let mut ignored = Vec::new();
            for card in self.board_cards(&board, &config, &mut ignored) {
                if board.column(&card.column).is_some() && !columns.iter().any(|c| c.id == card.column) {
                    return rule(format!(
                        "a coluna '{}' ainda tem cartões; mova-os antes de removê-la",
                        card.column
                    ));
                }
            }
            // Preserve unknown per-column keys by merging onto the existing entries.
            let current = y.parse().unwrap_or_default();
            let existing: Vec<Value> = match current.get("columns") {
                Some(Value::Sequence(s)) => s.clone(),
                _ => Vec::new(),
            };
            let items = columns
                .iter()
                .map(|c| {
                    let prev = existing.iter().find(|v| {
                        v.as_mapping().and_then(|m| crate::fields::get_str(m, "id")).as_deref() == Some(&c.id)
                    });
                    merge_map(prev, column_entries(c))
                })
                .collect();
            y.set("columns", &Y::Seq(items));
        }
        self.save(
            &file,
            Some(expected.unwrap_or(&hash(&text))),
            y.as_str(),
            "update_board",
        )?;
        self.board(id)
    }

    /// Moves the whole board folder to `archive/boards/`.
    pub fn archive_board(&self, id: &str) -> Result<()> {
        let board = self.board(id)?;
        let dest = self.root.join("archive").join("boards").join(&board.id);
        self.relocate(&board.dir, &dest, "archive_board")?;
        let order: Vec<String> = self.config()?.board_order.into_iter().filter(|b| b != id).collect();
        self.set_board_order(&order)
    }

    // ------------------------------------------------------------------ cards

    fn column_ranks(&self, board: &Board, column: &str, exclude: &str) -> Result<Vec<Card>> {
        let config = self.config()?;
        let mut ignored = Vec::new();
        Ok(self
            .board_cards(board, &config, &mut ignored)
            .into_iter()
            .filter(|c| c.column == column && c.id != exclude)
            .collect())
    }

    /// Plans a rank placing a card at `index` among `others` (sorted). Writes nothing.
    ///
    /// When hand-edited ranks leave no room, the plan also lists the other cards
    /// to re-space. Callers apply it with [`Self::apply_rank_plan`] only after the
    /// card's own write succeeded, so a rejected edit never touches other files.
    fn plan_rank(&self, others: &[Card], index: Option<usize>) -> RankPlan {
        let i = index.unwrap_or(others.len()).min(others.len());
        let before = i.checked_sub(1).map(|j| others[j].rank.as_str());
        let after = others.get(i).map(|c| c.rank.as_str());
        if let Some(rank) = rank::between(before, after) {
            return RankPlan {
                rank,
                respace: Vec::new(),
            };
        }
        let keys = rank::spread(others.len() + 1);
        let respace = others
            .iter()
            .enumerate()
            .filter_map(|(j, card)| {
                let k = if j < i { &keys[j] } else { &keys[j + 1] };
                (card.rank != *k).then(|| (card.path.clone(), card.hash.clone(), k.clone()))
            })
            .collect();
        RankPlan {
            rank: keys[i].clone(),
            respace,
        }
    }

    /// Best effort: a card edited since it was planned is skipped rather than
    /// overwritten, and a failure here leaves at worst two equal ranks, which sort
    /// by id. The edit that needed the plan has already succeeded.
    fn apply_rank_plan(&self, plan: &RankPlan) {
        for (path, hash, rank) in &plan.respace {
            let Ok(text) = read(path) else { continue };
            let mut doc = Doc::parse(&text);
            doc.set("rank", &Y::Quoted(rank.clone()));
            let _ = self.save(path, Some(hash), &doc.render(), "respace_column");
        }
    }

    pub fn create_card(&self, board_id: &str, new: &NewCard) -> Result<Card> {
        let board = self.board(board_id)?;
        let config = self.config()?;
        let tz = &config.tz;
        let title = new.title.trim();
        if title.is_empty() {
            return rule("o cartão precisa de um título");
        }
        let column = match &new.column {
            Some(c) => board
                .column(c)
                .ok_or_else(|| Error::Rule(format!("coluna '{c}' não existe no quadro '{board_id}'")))?
                .id
                .clone(),
            None => board.inbox().map(|c| c.id.clone()).unwrap_or_default(),
        };
        if let Some(n) = &new.network
            && config.network(n).is_none()
        {
            return rule(format!("rede '{n}' não configurada"));
        }
        let others = self.column_ranks(&board, &column, "")?;
        let plan = self.plan_rank(&others, if new.top { Some(0) } else { None });
        let id = new_id();
        let now = format_instant(self.now(), tz);

        let mut doc = Doc::new(
            new.body
                .as_deref()
                .map_or(String::from("\n"), |b| format!("\n{}\n", b.trim()))
                .as_str(),
        );
        doc.set("id", &Y::str(&id));
        doc.set("title", &Y::Quoted(title.into()));
        doc.set(
            "kind",
            &Y::str(
                new.kind
                    .clone()
                    .unwrap_or_else(|| board.kind.default_card_kind().into()),
            ),
        );
        doc.set("column", &Y::str(&column));
        doc.set("rank", &Y::Quoted(plan.rank.clone()));
        doc.set("tags", &Y::str_list(&new.tags));
        if let Some(n) = &new.network {
            doc.set("network", &Y::str(n));
            let at = match &new.publication_at {
                Some(s) => Y::Plain(When::parse(s, tz)?.to_file_string(tz)),
                None => Y::Null,
            };
            doc.set(
                "publication",
                &Y::Map(vec![
                    ("at".into(), at),
                    ("status".into(), Y::str("planned")),
                    ("url".into(), Y::Null),
                ]),
            );
        } else if new.publication_at.is_some() {
            return rule("defina a rede do cartão para agendar a publicação");
        }
        if let Some(d) = &new.due {
            doc.set("due", &Y::Plain(When::parse(d, tz)?.to_file_string(tz)));
        }
        if let Some(r) = &new.reminders {
            validate_reminders(r)?;
            doc.set("reminders", &Y::str_list(r));
        }
        let mut links = Vec::new();
        if let Some(b) = &new.links_board {
            links.push(("board".to_string(), Y::str(b)));
        }
        if let Some(a) = &new.links_article {
            links.push(("article".to_string(), Y::str(a)));
        }
        if !links.is_empty() {
            doc.set("links", &Y::Map(links));
        }
        doc.set("created", &Y::Plain(now.clone()));
        doc.set("updated", &Y::Plain(now.clone()));
        doc.set("column_since", &column_since_value(&column, &now));

        let cards_dir = board.dir.join("cards");
        let mut path = cards_dir.join(format!("{}--{}.md", slugify(title), id_suffix(&id)));
        if path.exists() {
            path = cards_dir.join(format!("{}--{}.md", slugify(title), id.to_lowercase()));
        }
        self.save(&path, None, &doc.render(), "create_card")?;
        self.apply_rank_plan(&plan);
        Ok(self.read_card_file(&path, &board, &config)?.0)
    }

    /// Read-modify-write of one card. The closure edits the document; `updated`
    /// is refreshed and the write is refused if the file changed meanwhile.
    fn edit_card(
        &self,
        id: &str,
        expected: Option<&str>,
        op: &str,
        f: impl FnOnce(&mut Doc, &Card, &Board, &Config) -> Result<()>,
    ) -> Result<Card> {
        let (path, board) = self.locate_card(id)?;
        let config = self.config()?;
        let (card, mut doc) = self.read_card_file(&path, &board, &config)?;
        let before = doc.render();
        f(&mut doc, &card, &board, &config)?;
        if doc.render() == before {
            if let Some(e) = expected
                && e != card.hash
            {
                return Err(Error::Conflict {
                    path,
                    disk_content: before,
                    disk_hash: card.hash,
                });
            }
            return Ok(card);
        }
        let now = format_instant(self.now(), &config.tz);
        // A column change (move, update, schedule, publish) restarts the "time in column" clock.
        let new_column = doc.fields().ok().and_then(|m| crate::fields::get_str(&m, "column"));
        if let Some(col) = new_column.filter(|c| *c != card.column) {
            doc.set("column_since", &column_since_value(&col, &now));
        }
        doc.set("updated", &Y::Plain(now));
        self.save(&path, Some(expected.unwrap_or(&card.hash)), &doc.render(), op)?;
        Ok(self.read_card_file(&path, &board, &config)?.0)
    }

    pub fn update_card(&self, id: &str, expected: Option<&str>, patch: &CardPatch) -> Result<Card> {
        // A column change through a patch lands at the bottom of the new column.
        let plan = match &patch.column {
            Some(col) => {
                let (_, board) = self.locate_card(id)?;
                if board.column(col).is_none() {
                    return rule(format!("coluna '{col}' não existe no quadro '{}'", board.id));
                }
                let current = self.card(id)?;
                if current.column == *col {
                    None
                } else {
                    let others = self.column_ranks(&board, col, id)?;
                    Some(self.plan_rank(&others, None))
                }
            }
            None => None,
        };
        let updated = self.edit_card(id, expected, "update_card", |doc, card, _board, config| {
            let tz = &config.tz;
            if let Some(t) = &patch.title {
                if t.trim().is_empty() {
                    return rule("o cartão precisa de um título");
                }
                doc.set("title", &Y::Quoted(t.trim().into()));
            }
            if let Some(k) = &patch.kind {
                doc.set("kind", &Y::str(k));
            }
            if let Some(c) = &patch.column {
                doc.set("column", &Y::str(c));
            }
            if let Some(plan) = &plan {
                doc.set("rank", &Y::Quoted(plan.rank.clone()));
            }
            if let Some(tags) = &patch.tags {
                let tags: Vec<String> = tags
                    .iter()
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
                doc.set("tags", &Y::str_list(&tags));
            }
            if let Some(n) = &patch.network {
                match n {
                    Some(n) if config.network(n).is_none() => return rule(format!("rede '{n}' não configurada")),
                    Some(n) => doc.set("network", &Y::str(n)),
                    None => doc.remove("network"),
                }
            }
            if let Some(d) = &patch.due {
                match d {
                    Some(d) => doc.set("due", &Y::Plain(When::parse(d, tz)?.to_file_string(tz))),
                    None => doc.remove("due"),
                }
            }
            if let Some(r) = &patch.reminders {
                match r {
                    Some(r) => {
                        validate_reminders(r)?;
                        doc.set("reminders", &Y::str_list(r));
                    }
                    None => doc.remove("reminders"),
                }
            }
            if let Some(p) = &patch.publication {
                match p {
                    Some(p) => {
                        let has_network = match &patch.network {
                            Some(n) => n.is_some(),
                            None => card.network.is_some(),
                        };
                        if !has_network {
                            return rule("defina a rede do cartão antes da publicação");
                        }
                        set_publication(doc, p, tz)?;
                    }
                    None => doc.remove("publication"),
                }
            }
            if let Some(b) = &patch.links_board {
                set_links(doc, vec![("board".into(), Y::opt_str(b.clone()))])?;
            }
            if let Some(m) = &patch.metrics {
                match m {
                    Some(m) => {
                        let published = patch
                            .publication
                            .as_ref()
                            .and_then(|p| p.as_ref())
                            .and_then(|p| p.status.as_deref())
                            .map_or(card.publication.as_ref().map(|p| p.status.as_str()), Some);
                        if card.network.is_none() || published != Some("published") {
                            return rule("só é possível registrar resultados de um post já publicado");
                        }
                        let today = self.now().to_zoned(tz.clone()).date();
                        set_metrics(doc, m, today);
                    }
                    None => doc.remove("metrics"),
                }
            }
            if let Some(body) = &patch.body {
                doc.set_body(&normalize_body(body));
            }
            if let Some(s) = &patch.section {
                let body = set_section(doc.body(), &s.title, &s.text, s.mode);
                doc.set_body(&normalize_body(&body));
            }
            Ok(())
        })?;
        if let Some(plan) = &plan {
            self.apply_rank_plan(plan);
        }
        Ok(updated)
    }

    /// Moves a card to `column` at position `index` (0 = top, `None` = bottom).
    /// Only this card's file changes (section 5.4, `rank`).
    pub fn move_card(&self, id: &str, column: &str, index: Option<usize>) -> Result<Card> {
        let (_, board) = self.locate_card(id)?;
        if board.column(column).is_none() {
            return rule(format!("coluna '{column}' não existe no quadro '{}'", board.id));
        }
        let others = self.column_ranks(&board, column, id)?;
        let plan = self.plan_rank(&others, index);
        let moved = self.edit_card(id, None, "move_card", |doc, _, _, _| {
            doc.set("column", &Y::str(column));
            doc.set("rank", &Y::Quoted(plan.rank.clone()));
            Ok(())
        })?;
        self.apply_rank_plan(&plan);
        Ok(moved)
    }

    /// Schedules the card's publication at `at`, or at the next free slot of its
    /// network, and moves it to the board's "scheduled" column.
    pub fn schedule_post(&self, id: &str, at: Option<&str>) -> Result<Card> {
        let index = self.load()?;
        let card = index
            .card(id)
            .ok_or_else(|| Error::NotFound(format!("cartão '{id}'")))?;
        let Some(network) = card.network.clone() else {
            return rule("o cartão não tem rede definida (network)");
        };
        let when = match at {
            Some(s) => When::parse(s, &index.config.tz)?,
            None => {
                let slot = crate::schedule::find_slots(&index, &network, self.now(), 1, Some(id))?
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error::Rule(format!("nenhuma faixa de postagem configurada para '{network}'")))?;
                When::instant(slot, &index.config.tz)
            }
        };
        if when.all_day() {
            return rule("a publicação precisa de data e horário");
        }
        let board = index.board(&card.board).cloned();
        let target_col = board
            .as_ref()
            .and_then(|b| b.column_with_role(ColumnRole::Scheduled))
            .filter(|c| c.id != card.column)
            .map(|c| c.id.clone());
        let tz = index.config.tz.clone();
        let patch = PublicationPatch {
            at: Some(Some(when.to_file_string(&tz))),
            status: Some("scheduled".into()),
            url: None,
        };
        let card = self.edit_card(id, None, "schedule_post", |doc, _, _, _| {
            set_publication(doc, &patch, &tz)
        })?;
        match target_col {
            Some(col) if !index.is_done(&card) => self.move_card(id, &col, None),
            _ => Ok(card),
        }
    }

    /// Records the post as published (with its link) and moves the card to the
    /// board's done column.
    pub fn mark_published(&self, id: &str, url: Option<&str>, at: Option<&str>) -> Result<Card> {
        let config = self.config()?;
        let tz = config.tz.clone();
        let published_at = match at {
            Some(s) => crate::time::parse_instant(s, &tz)?,
            None => self.now(),
        };
        let card = self.edit_card(id, None, "mark_published", |doc, card, _, _| {
            if card.network.is_none() {
                return rule("o cartão não tem rede definida (network)");
            }
            let m = doc.fields().map_err(Error::Rule)?;
            let mut updates = vec![
                ("status".to_string(), Y::str("published")),
                ("published_at".to_string(), Y::Plain(format_instant(published_at, &tz))),
            ];
            // `merge_map` treats `Y::Null` as "remove the key", so only touch `url`
            // when a link was given; otherwise a link already on the card survives.
            if let Some(u) = url.map(str::trim).filter(|u| !u.is_empty()) {
                updates.push(("url".to_string(), Y::str(u)));
            }
            if card.publication.as_ref().and_then(|p| p.at.as_ref()).is_none() {
                updates.push(("at".into(), Y::Plain(format_instant(published_at, &tz))));
            }
            // Keep `url: null` visible, as in the documented format.
            let mut merged = merge_map(m.get("publication"), updates.clone());
            if let Y::Map(entries) = &mut merged
                && !entries.iter().any(|(k, _)| k == "url")
            {
                entries.push(("url".into(), Y::Null));
            }
            doc.set("publication", &merged);
            Ok(())
        })?;
        let (_, board) = self.locate_card(id)?;
        match board.column_with_role(ColumnRole::Done) {
            Some(done) if done.id != card.column => self.move_card(id, &done.id.clone(), Some(0)),
            _ => Ok(card),
        }
    }

    /// Copies a card for another network and links the versions as siblings.
    pub fn duplicate_for_network(&self, id: &str, network: &str) -> Result<Card> {
        let src = self.card(id)?;
        let new = NewCard {
            title: src.title.clone(),
            column: Some(src.column.clone()),
            kind: Some(src.kind.clone()),
            network: Some(network.into()),
            tags: src.tags.clone(),
            body: Some(src.body.clone()),
            links_board: src.links.board.clone(),
            links_article: src.links.article.clone(),
            ..Default::default()
        };
        let copy = self.create_card(&src.board, &new)?;
        let mut family: Vec<String> = vec![src.id.clone()];
        family.extend(src.links.siblings.iter().cloned());
        family.push(copy.id.clone());
        family.dedup();
        for member in &family {
            let others: Vec<String> = family.iter().filter(|f| *f != member).cloned().collect();
            match self.edit_card(member, None, "link_siblings", |doc, _, _, _| {
                set_links(doc, vec![("siblings".into(), Y::str_list(&others))])
            }) {
                Ok(_) | Err(Error::NotFound(_)) => {}
                Err(e) => return Err(e),
            }
        }
        self.card(&copy.id)
    }

    pub fn archive_card(&self, id: &str) -> Result<PathBuf> {
        let (path, board) = self.locate_card(id)?;
        self.archive_card_file(&path, &board.id)
    }

    fn archive_card_file(&self, path: &Path, board_id: &str) -> Result<PathBuf> {
        let name = path.file_name().unwrap_or_default();
        let dest = self.root.join("archive").join("cards").join(board_id).join(name);
        self.relocate(path, &dest, "archive_card")
    }

    pub fn trash_card(&self, id: &str) -> Result<PathBuf> {
        let (path, _) = self.locate_card(id)?;
        self.to_trash(&path)
    }

    /// Archives cards that sat in a done column longer than `auto_archive_days`.
    pub fn auto_archive(&self) -> Result<usize> {
        let index = self.load()?;
        let Some(days) = index.config.auto_archive_days else {
            return Ok(0);
        };
        let cutoff = self.now() - jiff::SignedDuration::from_hours(24 * days);
        let mut n = 0;
        for c in &index.cards {
            if index.is_done(c) && c.updated.is_some_and(|u| u < cutoff) {
                // The index already knows where the file is; no need to search for it.
                self.archive_card_file(&c.path, &c.board)?;
                n += 1;
            }
        }
        Ok(n)
    }

    // ----------------------------------------------------------------- events

    pub fn create_event(&self, new: &NewEvent) -> Result<Event> {
        let config = self.config()?;
        let tz = &config.tz;
        let title = new.title.trim();
        if title.is_empty() {
            return rule("o evento precisa de um título");
        }
        let when = When::parse(&new.date, tz)?;
        let id = new_id();
        let mut doc = Doc::new(
            new.body
                .as_deref()
                .map_or(String::from("\n"), |b| format!("\n{}\n", b.trim()))
                .as_str(),
        );
        doc.set("id", &Y::str(&id));
        doc.set("title", &Y::Quoted(title.into()));
        doc.set("kind", &Y::str(new.kind.clone().unwrap_or_else(|| "event".into())));
        doc.set(
            if when.all_day() { "date" } else { "start" },
            &Y::Plain(when.to_file_string(tz)),
        );
        if let Some(end) = &new.end {
            doc.set("end", &Y::Plain(When::parse(end, tz)?.to_file_string(tz)));
        }
        if let Some(r) = &new.reminders {
            validate_reminders(r)?;
            doc.set("reminders", &Y::str_list(r));
        }
        if let Some(b) = &new.board {
            doc.set("links", &Y::Map(vec![("board".into(), Y::str(b))]));
        }
        doc.set("tags", &Y::str_list(&new.tags));
        let path = self
            .root
            .join("calendar")
            .join(format!("{}-{}--{}.md", when.date, slugify(title), id_suffix(&id)));
        self.save(&path, None, &doc.render(), "create_event")?;
        Ok(self.read_event_file(&path, &config)?.0)
    }

    pub fn update_event(&self, id: &str, expected: Option<&str>, patch: &EventPatch) -> Result<Event> {
        let config = self.config()?;
        let tz = &config.tz;
        let (ev, mut doc) = self.locate_event(id)?;
        let before = doc.render();
        if let Some(t) = &patch.title {
            if t.trim().is_empty() {
                return rule("o evento precisa de um título");
            }
            doc.set("title", &Y::Quoted(t.trim().into()));
        }
        if let Some(k) = &patch.kind {
            doc.set("kind", &Y::str(k));
        }
        if let Some(d) = &patch.date {
            let when = When::parse(d, tz)?;
            let (set, clear) = if when.all_day() {
                ("date", "start")
            } else {
                ("start", "date")
            };
            doc.set(set, &Y::Plain(when.to_file_string(tz)));
            doc.remove(clear);
        }
        if let Some(e) = &patch.end {
            match e {
                Some(e) => doc.set("end", &Y::Plain(When::parse(e, tz)?.to_file_string(tz))),
                None => doc.remove("end"),
            }
        }
        if let Some(r) = &patch.reminders {
            match r {
                Some(r) => {
                    validate_reminders(r)?;
                    doc.set("reminders", &Y::str_list(r));
                }
                None => doc.remove("reminders"),
            }
        }
        if let Some(b) = &patch.board {
            set_links(&mut doc, vec![("board".into(), Y::opt_str(b.clone()))])?;
        }
        if let Some(tags) = &patch.tags {
            doc.set("tags", &Y::str_list(tags));
        }
        if let Some(body) = &patch.body {
            doc.set_body(&normalize_body(body));
        }
        if doc.render() != before {
            self.save(
                &ev.path,
                Some(expected.unwrap_or(&ev.hash)),
                &doc.render(),
                "update_event",
            )?;
        }
        Ok(self.read_event_file(&ev.path, &config)?.0)
    }

    pub fn archive_event(&self, id: &str) -> Result<PathBuf> {
        let (ev, _) = self.locate_event(id)?;
        let dest = self
            .root
            .join("archive")
            .join("calendar")
            .join(ev.path.file_name().unwrap_or_default());
        self.relocate(&ev.path, &dest, "archive_event")
    }

    pub fn trash_event(&self, id: &str) -> Result<PathBuf> {
        let (ev, _) = self.locate_event(id)?;
        self.to_trash(&ev.path)
    }

    // ------------------------------------------------------- trash & archive

    fn to_trash(&self, path: &Path) -> Result<PathBuf> {
        let stamp = self.now().strftime("%Y%m%dT%H%M%S").to_string();
        let dest = self.root.join(".trash").join(stamp).join(self.rel(path));
        self.relocate(path, &dest, "trash")
    }

    pub fn list_trash(&self) -> Result<Vec<TrashEntry>> {
        let trash = self.root.join(".trash");
        let mut out = Vec::new();
        let Ok(stamps) = fs::read_dir(&trash) else {
            return Ok(out);
        };
        for stamp in stamps.flatten() {
            let stamp_dir = stamp.path();
            let mut stack = vec![stamp_dir.clone()];
            while let Some(dir) = stack.pop() {
                let Ok(entries) = fs::read_dir(&dir) else { continue };
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else if p.extension().is_some_and(|x| x == "md") {
                        out.push(TrashEntry {
                            original: p.strip_prefix(&stamp_dir).unwrap_or(&p).to_path_buf(),
                            title: title_of(&p),
                            deleted_at: stamp.file_name().to_string_lossy().into_owned(),
                            path: p,
                        });
                    }
                }
            }
        }
        out.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
        Ok(out)
    }

    pub fn restore_trash(&self, trash_path: &Path) -> Result<PathBuf> {
        let entry = self
            .list_trash()?
            .into_iter()
            .find(|e| e.path == trash_path)
            .ok_or_else(|| Error::NotFound(format!("item da lixeira {}", trash_path.display())))?;
        let dest = self.root.join(&entry.original);
        self.relocate(&entry.path, &dest, "restore_trash")
    }

    pub fn list_archive(&self) -> Result<Vec<ArchivedItem>> {
        let mut out = Vec::new();
        let cards = self.root.join("archive").join("cards");
        if let Ok(boards) = fs::read_dir(&cards) {
            for b in boards.flatten() {
                for p in md_files(&b.path()) {
                    out.push(ArchivedItem {
                        title: title_of(&p),
                        kind: "card".into(),
                        board: Some(b.file_name().to_string_lossy().into_owned()),
                        path: p,
                    });
                }
            }
        }
        for p in md_files(&self.root.join("archive").join("calendar")) {
            out.push(ArchivedItem {
                title: title_of(&p),
                kind: "event".into(),
                board: None,
                path: p,
            });
        }
        let boards = self.root.join("archive").join("boards");
        if let Ok(entries) = fs::read_dir(&boards) {
            for b in entries.flatten() {
                out.push(ArchivedItem {
                    title: self
                        .read_board_file(&b.path().join("board.yaml"), &b.file_name().to_string_lossy())
                        .map(|bd| bd.name)
                        .unwrap_or_else(|_| b.file_name().to_string_lossy().into_owned()),
                    kind: "board".into(),
                    board: Some(b.file_name().to_string_lossy().into_owned()),
                    path: b.path(),
                });
            }
        }
        Ok(out)
    }

    pub fn restore_archived(&self, path: &Path) -> Result<PathBuf> {
        let item = self
            .list_archive()?
            .into_iter()
            .find(|i| i.path == path)
            .ok_or_else(|| Error::NotFound(format!("item arquivado {}", path.display())))?;
        let name = path.file_name().unwrap_or_default();
        match item.kind.as_str() {
            "card" => {
                let board = item.board.unwrap_or_default();
                let bdir = self.root.join("boards").join(&board);
                if !bdir.exists() {
                    return rule(format!("o quadro '{board}' não existe mais; restaure-o primeiro"));
                }
                self.relocate(path, &bdir.join("cards").join(name), "restore_archived")
            }
            "event" => self.relocate(path, &self.root.join("calendar").join(name), "restore_archived"),
            _ => {
                let id = item.board.unwrap_or_default();
                let dest = self.relocate(path, &self.root.join("boards").join(&id), "restore_archived")?;
                let mut order = self.config()?.board_order;
                if !order.contains(&id) {
                    order.push(id);
                    self.set_board_order(&order)?;
                }
                Ok(dest)
            }
        }
    }
}

fn title_of(path: &Path) -> String {
    fs::read_to_string(path)
        .ok()
        .and_then(|t| Doc::parse(&t).fields().ok())
        .and_then(|m| crate::fields::get_str(&m, "title"))
        .unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
}

fn validate_reminders(list: &[String]) -> Result<()> {
    for r in list {
        crate::time::ReminderSpec::parse(r)?;
    }
    Ok(())
}

/// The body always starts with a blank line after the header and ends with a newline.
fn normalize_body(body: &str) -> String {
    let trimmed = body.trim_matches('\n');
    if trimmed.trim().is_empty() {
        "\n".into()
    } else {
        format!("\n{trimmed}\n")
    }
}

fn set_metrics(doc: &mut Doc, m: &MetricsInput, recorded: jiff::civil::Date) {
    let fields = [
        ("impressions", m.impressions),
        ("reactions", m.reactions),
        ("comments", m.comments),
        ("shares", m.shares),
        ("saves", m.saves),
        ("clicks", m.clicks),
    ];
    let mut entries: Vec<(String, Y)> = fields
        .iter()
        .filter_map(|(k, v)| v.map(|v| ((*k).to_string(), Y::Int(i64::try_from(v).unwrap_or(i64::MAX)))))
        .collect();
    if entries.is_empty() {
        doc.remove("metrics");
        return;
    }
    entries.push(("recorded".into(), Y::Plain(recorded.to_string())));
    doc.set("metrics", &Y::Map(entries));
}

fn column_since_value(column: &str, at: &str) -> Y {
    Y::Map(vec![
        ("column".into(), Y::str(column)),
        ("at".into(), Y::Plain(at.to_string())),
    ])
}

fn column_entries(c: &Column) -> Vec<(String, Y)> {
    vec![
        ("id".into(), Y::str(&c.id)),
        ("name".into(), Y::Quoted(c.name.clone())),
        (
            "role".into(),
            c.role.map_or(Y::Null, |r| {
                Y::str(match r {
                    ColumnRole::Inbox => "inbox",
                    ColumnRole::Scheduled => "scheduled",
                    ColumnRole::Done => "done",
                })
            }),
        ),
        ("limit".into(), c.limit.map_or(Y::Null, |l| Y::Int(l.into()))),
    ]
}

fn columns_y(cols: &[Column]) -> Y {
    Y::Seq(cols.iter().map(|c| merge_map(None, column_entries(c))).collect())
}

fn set_links(doc: &mut Doc, updates: Vec<(String, Y)>) -> Result<()> {
    let m: Mapping = doc.fields().map_err(Error::Rule)?;
    let merged = merge_map(m.get("links"), updates);
    match &merged {
        Y::Map(e) if e.is_empty() => doc.remove("links"),
        _ => doc.set("links", &merged),
    }
    Ok(())
}

fn set_publication(doc: &mut Doc, p: &PublicationPatch, tz: &jiff::tz::TimeZone) -> Result<()> {
    let m: Mapping = doc.fields().map_err(Error::Rule)?;
    let existing = m.get("publication").filter(|v| v.is_mapping());
    let mut updates: Vec<(String, Y)> = Vec::new();
    if let Some(at) = &p.at {
        let v = match at {
            Some(s) => {
                let w = When::parse(s, tz)?;
                if w.all_day() {
                    return rule("a publicação precisa de data e horário");
                }
                Y::Plain(w.to_file_string(tz))
            }
            None => Y::Null,
        };
        updates.push(("at".into(), v));
    }
    if let Some(s) = &p.status {
        if PubStatus::parse(s).is_none() {
            return rule(format!(
                "estado inválido '{s}': use planned, scheduled, published ou skipped"
            ));
        }
        updates.push(("status".into(), Y::str(s)));
    } else if existing.is_none() {
        updates.push(("status".into(), Y::str("planned")));
    }
    if let Some(u) = &p.url {
        updates.push(("url".into(), Y::opt_str(u.clone())));
    }
    // merge_map drops null keys; the documented format keeps `at: null` / `url: null`.
    let mut merged = merge_map(existing, updates);
    if let Y::Map(entries) = &mut merged {
        for k in ["at", "url"] {
            if !entries.iter().any(|(ek, _)| ek == k) {
                let pos = if k == "at" { 0 } else { entries.len() };
                entries.insert(pos, (k.into(), Y::Null));
            }
        }
    }
    doc.set("publication", &merged);
    Ok(())
}

// ------------------------------------------------------------------- artwork

/// Largest file `attach_asset` accepts.
pub const MAX_ASSET_BYTES: u64 = 200 * 1024 * 1024;

/// A file name that is safe inside the assets folder: no path parts, no leading dot.
fn safe_file_name(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_start_matches('.').trim().to_string();
    (!cleaned.is_empty()).then_some(cleaned)
}

fn art_value(status: ArtStatus, format: Option<&str>, requested_at: &str) -> Y {
    let mut entries = vec![("status".to_string(), Y::str(status.as_str()))];
    if let Some(f) = format {
        entries.push(("format".to_string(), Y::str(f)));
    }
    entries.push(("requested_at".to_string(), Y::Plain(requested_at.to_string())));
    Y::Map(entries)
}

impl Workspace {
    /// Orders artwork for a post: records the request on the card, makes sure the body has
    /// an `## Arte` section for the brief (writing `brief` there if given), and moves the card
    /// to the production column if it is still earlier in the flow.
    pub fn request_art(&self, id: &str, format: Option<&str>, brief: Option<&str>) -> Result<Card> {
        let index = self.load()?;
        let card = index
            .card(id)
            .ok_or_else(|| Error::NotFound(format!("cartão '{id}'")))?;
        let Some(net) = card.network.as_deref().and_then(|n| index.config.network(n)) else {
            return rule("o cartão precisa de uma rede para pedir a arte");
        };
        let format_id = match format {
            Some(f) => {
                if !net.media.formats.iter().any(|x| x.id == f) {
                    let valid: Vec<&str> = net.media.formats.iter().map(|x| x.id.as_str()).collect();
                    return rule(format!(
                        "formato '{f}' não existe em {}; use: {}",
                        net.label,
                        valid.join(", ")
                    ));
                }
                Some(f.to_string())
            }
            None => crate::art::format_for(net, card).map(|f| f.id.clone()),
        };
        let board = index.board(&card.board).cloned();
        let production = board.as_ref().and_then(|b| {
            let target = b.column("producing")?;
            let (here, there) = (b.column_index(&card.column), b.column_index(&target.id));
            (here < there && !b.is_done_column(&card.column)).then(|| target.id.clone())
        });
        let now = format_instant(self.now(), &index.config.tz);
        let has_section = crate::markdown::section_text(&card.body, "Arte").is_some();
        let updated = self.edit_card(id, None, "request_art", |doc, _, _, _| {
            doc.set("art", &art_value(ArtStatus::Requested, format_id.as_deref(), &now));
            match brief.map(str::trim).filter(|b| !b.is_empty()) {
                Some(b) => {
                    let body = set_section(doc.body(), "Arte", b, SectionMode::Replace);
                    doc.set_body(&normalize_body(&body));
                }
                None if !has_section => {
                    let body = set_section(
                        doc.body(),
                        "Arte",
                        "_Descreva a direção visual: o que mostrar, tom, cores, referências._",
                        SectionMode::Replace,
                    );
                    doc.set_body(&normalize_body(&body));
                }
                None => {}
            }
            Ok(())
        })?;
        match production {
            Some(col) => self.move_card(id, &col, None),
            None => Ok(updated),
        }
    }

    /// Marks the artwork `requested` (back to waiting), `ready`, or clears the record (`none`).
    /// `ready` needs at least one file in the card's attachments folder.
    pub fn set_art_status(&self, id: &str, status: &str) -> Result<Card> {
        let now = format_instant(self.now(), &self.config()?.tz);
        let card = self.card(id)?;
        let wanted =
            match status {
                "none" => None,
                s => Some(ArtStatus::parse(s).ok_or_else(|| {
                    Error::Rule(format!("estado de arte inválido '{s}': use requested, ready ou none"))
                })?),
            };
        if wanted == Some(ArtStatus::Ready) && card.assets.is_empty() {
            return rule(format!(
                "não há arquivos na pasta de anexos ({}); anexe a arte antes de marcá-la como pronta",
                self.assets_dir(&card).display()
            ));
        }
        self.edit_card(id, None, "set_art_status", |doc, card, _, _| {
            match wanted {
                None => doc.remove("art"),
                Some(s) => {
                    let format = card.art.as_ref().and_then(|a| a.format.clone());
                    let at = card.art.as_ref().and_then(|a| a.requested_at).map_or(now.clone(), |t| {
                        format_instant(t, &self.config().map(|c| c.tz).unwrap_or(jiff::tz::TimeZone::UTC))
                    });
                    doc.set("art", &art_value(s, format.as_deref(), &at));
                }
            }
            Ok(())
        })
    }

    /// Copies a file the author picked into the card's attachments folder, never
    /// overwriting an existing file. Returns where it went.
    pub fn attach_asset(&self, id: &str, source: &Path) -> Result<PathBuf> {
        let card = self.card(id)?;
        let meta = fs::metadata(source).at(source)?;
        if !meta.is_file() {
            return rule(format!("{} não é um arquivo", source.display()));
        }
        if meta.len() > MAX_ASSET_BYTES {
            return rule(format!(
                "arquivo grande demais ({} MB; o limite é {} MB)",
                meta.len() / 1_048_576,
                MAX_ASSET_BYTES / 1_048_576
            ));
        }
        let name = source
            .file_name()
            .and_then(|n| safe_file_name(&n.to_string_lossy()))
            .ok_or_else(|| Error::Rule("nome de arquivo inválido".into()))?;
        let dir = self.assets_dir(&card);
        fs::create_dir_all(&dir).at(&dir)?;
        let mut dest = dir.join(&name);
        let (stem, ext) = match dest.extension() {
            Some(e) => (
                dest.file_stem().unwrap_or_default().to_string_lossy().into_owned(),
                format!(".{}", e.to_string_lossy()),
            ),
            None => (name.clone(), String::new()),
        };
        let mut n = 2;
        while dest.exists() {
            dest = dir.join(format!("{stem}-{n}{ext}"));
            n += 1;
        }
        // Copy to a temporary file in the same folder, then rename: a half-copied
        // file never shows up under its final name.
        let mut tmp = tempfile::Builder::new().prefix(".copy-").tempfile_in(&dir).at(&dir)?;
        let mut input = fs::File::open(source).at(source)?;
        std::io::copy(&mut input, &mut tmp).at(&dest)?;
        tmp.as_file().sync_all().at(&dest)?;
        tmp.persist(&dest).map_err(|e| Error::Io {
            path: dest.clone(),
            source: e.error,
        })?;
        self.log("attach_asset", &dest);
        Ok(dest)
    }
}

/// What `import_metrics` did: cards updated, and the ones that failed with the reason.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub applied: Vec<String>,
    pub failed: Vec<(String, String)>,
}

impl Workspace {
    /// Reads an analytics CSV and says which published card each row would update. Writes nothing.
    pub fn preview_metrics_import(&self, csv_text: &str) -> Result<crate::import::Preview> {
        let parsed = crate::import::parse(csv_text)?;
        let index = self.load()?;
        let today = self.now().to_zoned(index.config.tz.clone()).date();
        Ok(crate::import::plan(&index, &parsed, today))
    }

    /// Applies the same plan the preview showed. A failure on one card does not stop the others.
    pub fn import_metrics(&self, csv_text: &str) -> Result<ImportResult> {
        let preview = self.preview_metrics_import(csv_text)?;
        let mut result = ImportResult {
            applied: Vec::new(),
            failed: Vec::new(),
        };
        for m in preview.matched {
            let input = MetricsInput {
                impressions: m.after.impressions,
                reactions: m.after.reactions,
                comments: m.after.comments,
                shares: m.after.shares,
                saves: m.after.saves,
                clicks: m.after.clicks,
            };
            let patch = CardPatch {
                metrics: Some(Some(input)),
                ..Default::default()
            };
            match self.update_card(&m.card_id, None, &patch) {
                Ok(_) => result.applied.push(m.card_id),
                Err(e) => result.failed.push((m.card_id, e.to_string())),
            }
        }
        Ok(result)
    }
}
