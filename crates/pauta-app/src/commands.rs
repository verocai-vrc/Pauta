//! Commands the UI calls through `invoke`. Each one is a thin wrapper over
//! `pauta-core`, followed by a refresh so every window sees the change.

use std::path::PathBuf;

use pauta_core::config::{Config, ConfigPatch};
use pauta_core::model::{Board, BoardKind, Card, Event};
use pauta_core::schedule::{self, AgendaItem, Today};
use pauta_core::store::{ArchivedItem, BoardPatch, CardPatch, EventPatch, NewCard, NewEvent, TrashEntry};
use pauta_core::time::{When, fmt_hm};
use pauta_core::{Error, Index, Workspace, locate};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_opener::OpenerExt;

use crate::state::{AppState, activate, refresh};

/// Errors as the UI sees them. Conflicts carry the version on disk so the
/// editor can show both and let the author choose (section 14.2).
#[derive(Debug, Serialize)]
pub struct CmdError {
    kind: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    disk_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disk_hash: Option<String>,
}

impl From<Error> for CmdError {
    fn from(e: Error) -> Self {
        let message = e.to_string();
        match e {
            Error::Conflict {
                disk_content,
                disk_hash,
                ..
            } => CmdError {
                kind: "conflict",
                message,
                disk_content: Some(disk_content),
                disk_hash: Some(disk_hash),
            },
            Error::NotFound(_) => CmdError::plain("not_found", message),
            Error::Invalid { .. } => CmdError::plain("invalid", message),
            Error::Rule(_) => CmdError::plain("rule", message),
            Error::Io { .. } => CmdError::plain("io", message),
        }
    }
}

impl CmdError {
    fn plain(kind: &'static str, message: impl Into<String>) -> Self {
        CmdError {
            kind,
            message: message.into(),
            disk_content: None,
            disk_hash: None,
        }
    }

    fn other(e: impl std::fmt::Display) -> Self {
        CmdError::plain("other", e.to_string())
    }
}

type R<T> = Result<T, CmdError>;

fn ws(state: &State<AppState>) -> R<Workspace> {
    state
        .workspace()
        .ok_or_else(|| CmdError::plain("no_workspace", "nenhuma pasta de dados aberta"))
}

fn index(state: &State<AppState>) -> R<Index> {
    if let Some(e) = state.load_error.read().unwrap().clone() {
        return Err(CmdError::plain("invalid", e));
    }
    state
        .index()
        .ok_or_else(|| CmdError::plain("no_workspace", "nenhuma pasta de dados aberta"))
}

/// Runs a write and refreshes the index afterwards.
fn write<T>(app: &AppHandle, state: &State<AppState>, f: impl FnOnce(&Workspace) -> pauta_core::Result<T>) -> R<T> {
    let w = ws(state)?;
    let out = f(&w)?;
    refresh(app);
    Ok(out)
}

#[derive(Serialize)]
pub struct Status {
    root: Option<PathBuf>,
    suggested: PathBuf,
    error: Option<String>,
}

#[tauri::command]
pub fn app_status(state: State<AppState>) -> Status {
    Status {
        root: state.workspace().map(|w| w.root().to_path_buf()),
        suggested: locate::default_suggestion(),
        error: state.load_error.read().unwrap().clone(),
    }
}

/// First run: creates (or opens) the data folder and remembers it.
#[tauri::command]
pub fn setup_workspace(app: AppHandle, state: State<AppState>, root: String) -> R<Status> {
    let root = locate::expand(&root);
    Workspace::init(&root, &pauta_core::templates::default_seed())?;
    locate::save_root(&root)?;
    activate(&app, &root)?;
    Ok(app_status(state))
}

#[tauri::command]
pub fn get_index(state: State<AppState>) -> R<Index> {
    index(&state)
}

#[tauri::command]
pub fn get_today(state: State<AppState>) -> R<Today> {
    let w = ws(&state)?;
    Ok(schedule::today(&index(&state)?, w.now()))
}

#[tauri::command]
pub fn get_rhythm(state: State<AppState>) -> R<pauta_core::rhythm::Rhythm> {
    let w = ws(&state)?;
    Ok(pauta_core::rhythm::rhythm(&index(&state)?, w.now()))
}

/// An analytics export is a few KB; anything much larger is the wrong file.
const MAX_CSV: usize = 5 * 1024 * 1024;

/// Which published card each row of an analytics CSV belongs to. Writes nothing.
#[tauri::command]
pub fn preview_metrics_import(state: State<AppState>, text: String) -> R<pauta_core::import::Preview> {
    if text.len() > MAX_CSV {
        return Err(CmdError::plain(
            "rule",
            "o arquivo é grande demais para ser uma exportação de estatísticas",
        ));
    }
    Ok(ws(&state)?.preview_metrics_import(&text)?)
}

#[tauri::command]
pub fn import_metrics(state: State<AppState>, text: String) -> R<pauta_core::store::ImportResult> {
    if text.len() > MAX_CSV {
        return Err(CmdError::plain(
            "rule",
            "o arquivo é grande demais para ser uma exportação de estatísticas",
        ));
    }
    Ok(ws(&state)?.import_metrics(&text)?)
}

/// Writes the calendar as `.ics` to a path the user picked. Returns the number of events.
#[tauri::command]
pub fn export_ics(state: State<AppState>, path: PathBuf, include_done: bool) -> R<usize> {
    if path.extension().is_none_or(|e| !e.eq_ignore_ascii_case("ics")) {
        return Err(CmdError::plain("rule", "o arquivo precisa terminar em .ics"));
    }
    let w = ws(&state)?;
    let text = pauta_core::ics::export(&index(&state)?, w.now(), pauta_core::ics::IcsOptions { include_done });
    pauta_core::store::write_atomic(&path, &text)?;
    Ok(text.matches("BEGIN:VEVENT").count())
}

#[tauri::command]
pub fn get_agenda(state: State<AppState>, from: String, to: String) -> R<Vec<AgendaItem>> {
    let parse = |s: &str| s.parse::<jiff::civil::Date>().map_err(CmdError::other);
    Ok(schedule::agenda(&index(&state)?, parse(&from)?, parse(&to)?))
}

#[derive(Serialize)]
pub struct Slot {
    /// `YYYY-MM-DDTHH:MM` in the configured zone, ready for an input field.
    local: String,
    date: String,
    time: String,
}

#[tauri::command]
pub fn find_slots(
    state: State<AppState>,
    network: String,
    count: Option<usize>,
    exclude: Option<String>,
) -> R<Vec<Slot>> {
    let w = ws(&state)?;
    let idx = index(&state)?;
    let tz = idx.config.tz.clone();
    let slots = schedule::find_slots(&idx, &network, w.now(), count.unwrap_or(3), exclude.as_deref())?;
    Ok(slots
        .into_iter()
        .map(|t| {
            let w = When::instant(t, &tz);
            let time = w.time.map(fmt_hm).unwrap_or_default();
            Slot {
                local: format!("{}T{}", w.date, time),
                date: w.date.to_string(),
                time,
            }
        })
        .collect())
}

#[tauri::command]
pub fn create_card(app: AppHandle, state: State<AppState>, board: String, card: NewCard) -> R<Card> {
    write(&app, &state, |w| w.create_card(&board, &card))
}

#[tauri::command]
pub fn update_card(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    expected: Option<String>,
    patch: CardPatch,
) -> R<Card> {
    write(&app, &state, |w| w.update_card(&id, expected.as_deref(), &patch))
}

#[tauri::command]
pub fn move_card(app: AppHandle, state: State<AppState>, id: String, column: String, index: Option<usize>) -> R<Card> {
    write(&app, &state, |w| w.move_card(&id, &column, index))
}

#[tauri::command]
pub fn schedule_post(app: AppHandle, state: State<AppState>, id: String, at: Option<String>) -> R<Card> {
    write(&app, &state, |w| w.schedule_post(&id, at.as_deref()))
}

#[tauri::command]
pub fn mark_published(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    url: Option<String>,
    at: Option<String>,
) -> R<Card> {
    write(&app, &state, |w| w.mark_published(&id, url.as_deref(), at.as_deref()))
}

#[tauri::command]
pub fn duplicate_card(app: AppHandle, state: State<AppState>, id: String, network: String) -> R<Card> {
    write(&app, &state, |w| w.duplicate_for_network(&id, &network))
}

#[tauri::command]
pub fn archive_card(app: AppHandle, state: State<AppState>, id: String) -> R<()> {
    write(&app, &state, |w| w.archive_card(&id).map(|_| ()))
}

#[tauri::command]
pub fn trash_card(app: AppHandle, state: State<AppState>, id: String) -> R<()> {
    write(&app, &state, |w| w.trash_card(&id).map(|_| ()))
}

#[tauri::command]
pub fn create_event(app: AppHandle, state: State<AppState>, event: NewEvent) -> R<Event> {
    write(&app, &state, |w| w.create_event(&event))
}

#[tauri::command]
pub fn update_event(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    expected: Option<String>,
    patch: EventPatch,
) -> R<Event> {
    write(&app, &state, |w| w.update_event(&id, expected.as_deref(), &patch))
}

#[tauri::command]
pub fn archive_event(app: AppHandle, state: State<AppState>, id: String) -> R<()> {
    write(&app, &state, |w| w.archive_event(&id).map(|_| ()))
}

#[tauri::command]
pub fn trash_event(app: AppHandle, state: State<AppState>, id: String) -> R<()> {
    write(&app, &state, |w| w.trash_event(&id).map(|_| ()))
}

#[tauri::command]
pub fn create_board(app: AppHandle, state: State<AppState>, name: String, kind: BoardKind) -> R<Board> {
    write(&app, &state, |w| w.create_board(&name, kind, None))
}

#[tauri::command]
pub fn update_board(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    expected: Option<String>,
    patch: BoardPatch,
) -> R<Board> {
    write(&app, &state, |w| w.update_board(&id, expected.as_deref(), &patch))
}

#[tauri::command]
pub fn set_board_order(app: AppHandle, state: State<AppState>, order: Vec<String>) -> R<()> {
    write(&app, &state, |w| w.set_board_order(&order))
}

#[tauri::command]
pub fn archive_board(app: AppHandle, state: State<AppState>, id: String) -> R<()> {
    write(&app, &state, |w| w.archive_board(&id))
}

#[tauri::command]
pub fn list_trash(state: State<AppState>) -> R<Vec<TrashEntry>> {
    Ok(ws(&state)?.list_trash()?)
}

#[tauri::command]
pub fn restore_trash(app: AppHandle, state: State<AppState>, path: PathBuf) -> R<()> {
    write(&app, &state, |w| w.restore_trash(&path).map(|_| ()))
}

#[tauri::command]
pub fn list_archive(state: State<AppState>) -> R<Vec<ArchivedItem>> {
    Ok(ws(&state)?.list_archive()?)
}

#[tauri::command]
pub fn restore_archived(app: AppHandle, state: State<AppState>, path: PathBuf) -> R<()> {
    write(&app, &state, |w| w.restore_archived(&path).map(|_| ()))
}

#[tauri::command]
pub fn update_config(app: AppHandle, state: State<AppState>, patch: ConfigPatch) -> R<Config> {
    write(&app, &state, |w| w.update_config(&patch))
}

/// The text to paste into the network: the network's section, or the body minus notes and the art brief.
pub fn post_text_of(idx: &Index, card: &Card) -> String {
    pauta_core::art::text_for_posting(idx, card)
}

#[tauri::command]
pub fn copy_post_text(app: AppHandle, state: State<AppState>, id: String) -> R<usize> {
    let idx = index(&state)?;
    let card = idx
        .card(&id)
        .ok_or_else(|| CmdError::plain("not_found", format!("cartão '{id}'")))?;
    let text = post_text_of(&idx, card);
    app.clipboard().write_text(text.clone()).map_err(CmdError::other)?;
    Ok(text.chars().count())
}

/// Orders artwork for a post (section "Arte" of the card, status, format). See ADR 0007.
#[tauri::command]
pub fn request_art(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    format: Option<String>,
    brief: Option<String>,
) -> R<Card> {
    write(&app, &state, |w| {
        w.request_art(&id, format.as_deref(), brief.as_deref())
    })
}

#[tauri::command]
pub fn set_art_status(app: AppHandle, state: State<AppState>, id: String, status: String) -> R<Card> {
    write(&app, &state, |w| w.set_art_status(&id, &status))
}

/// Copies the files the user picked into the card's attachments folder. Returns the names they got.
#[tauri::command]
pub fn attach_assets(app: AppHandle, state: State<AppState>, id: String, paths: Vec<PathBuf>) -> R<Vec<String>> {
    write(&app, &state, |w| {
        paths
            .iter()
            .map(|p| {
                w.attach_asset(&id, p)
                    .map(|d| d.file_name().unwrap_or_default().to_string_lossy().into_owned())
            })
            .collect()
    })
}

/// Puts the brief for the design tool on the clipboard.
#[tauri::command]
pub fn copy_art_request(app: AppHandle, state: State<AppState>, id: String) -> R<()> {
    let idx = index(&state)?;
    let card = idx
        .card(&id)
        .ok_or_else(|| CmdError::plain("not_found", format!("cartão '{id}'")))?;
    let text = pauta_core::art::request_text(&idx, card)?;
    app.clipboard().write_text(text).map_err(CmdError::other)
}

/// Opens the design tool's page (configurable; there is no API to call).
#[tauri::command]
pub fn open_art_tool(app: AppHandle, state: State<AppState>) -> R<()> {
    let url = index(&state)?.config.art.tool_url;
    app.opener().open_url(url, None::<&str>).map_err(CmdError::other)
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> R<()> {
    app.clipboard().write_text(text).map_err(CmdError::other)
}

#[derive(Serialize)]
pub struct Asset {
    name: String,
    path: PathBuf,
}

#[tauri::command]
pub fn list_assets(state: State<AppState>, id: String) -> R<Vec<Asset>> {
    let w = ws(&state)?;
    let idx = index(&state)?;
    let Some(card) = idx.card(&id) else {
        return Ok(Vec::new());
    };
    let dir = w.assets_dir(card);
    let mut out: Vec<Asset> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| Asset {
            name: e.file_name().to_string_lossy().into_owned(),
            path: e.path(),
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[tauri::command]
pub fn open_assets(app: AppHandle, state: State<AppState>, id: String) -> R<()> {
    let w = ws(&state)?;
    let idx = index(&state)?;
    let card = idx
        .card(&id)
        .ok_or_else(|| CmdError::plain("not_found", format!("cartão '{id}'")))?;
    let dir = w.assets_dir(card);
    std::fs::create_dir_all(&dir).map_err(CmdError::other)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(CmdError::other)
}

#[tauri::command]
pub fn open_path(app: AppHandle, path: PathBuf) -> R<()> {
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(CmdError::other)
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> R<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(CmdError::plain("rule", "só links http(s) podem ser abertos"));
    }
    app.opener().open_url(url, None::<&str>).map_err(CmdError::other)
}

#[derive(Serialize)]
pub struct Preferences {
    close_to_tray: bool,
    autostart: bool,
    theme: String,
}

#[tauri::command]
pub fn get_preferences(app: AppHandle) -> Preferences {
    let s = crate::settings::load();
    Preferences {
        close_to_tray: s.close_to_tray,
        autostart: app.autolaunch().is_enabled().unwrap_or(false),
        theme: s.theme,
    }
}

#[tauri::command]
pub fn set_preference(app: AppHandle, key: String, value: serde_json::Value) -> R<Preferences> {
    use pauta_core::yaml::Y;
    match (key.as_str(), &value) {
        ("autostart", serde_json::Value::Bool(on)) => {
            let al = app.autolaunch();
            if *on { al.enable() } else { al.disable() }.map_err(CmdError::other)?;
        }
        ("close_to_tray", serde_json::Value::Bool(on)) => crate::settings::set("close_to_tray", Y::Bool(*on))?,
        ("theme", serde_json::Value::String(t)) if ["system", "light", "dark"].contains(&t.as_str()) => {
            crate::settings::set("theme", Y::str(t))?
        }
        _ => return Err(CmdError::plain("rule", format!("preferência inválida: {key}"))),
    }
    Ok(get_preferences(app))
}

#[derive(Serialize)]
pub struct McpSnippet {
    binary: PathBuf,
    binary_exists: bool,
    claude_code: String,
    claude_desktop: String,
}

/// Ready-to-copy client configuration (section 11.5).
#[tauri::command]
pub fn mcp_snippet(state: State<AppState>) -> R<McpSnippet> {
    let root = ws(&state)?.root().to_path_buf();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    let binary = [exe_dir.join("pauta-mcp"), PathBuf::from("/usr/bin/pauta-mcp")]
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| exe_dir.join("pauta-mcp"));
    let claude_code = format!(
        "claude mcp add --scope user pauta -- '{}' --root '{}'",
        binary.display(),
        root.display()
    );
    let desktop = serde_json::json!({
        "mcpServers": { "pauta": { "command": binary, "args": ["--root", root] } }
    });
    Ok(McpSnippet {
        binary_exists: binary.exists(),
        binary,
        claude_code,
        claude_desktop: serde_json::to_string_pretty(&desktop).unwrap_or_default(),
    })
}

#[tauri::command]
pub fn open_capture(app: AppHandle) -> R<()> {
    crate::show_capture(&app).map_err(CmdError::other)
}

#[tauri::command]
pub fn close_capture(app: AppHandle) {
    if let Some(w) = app.get_webview_window("capture") {
        let _ = w.close();
    }
}
