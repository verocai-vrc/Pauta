//! Shared app state: the open data folder and its in-memory index.

use std::path::Path;
use std::sync::{Mutex, RwLock};

use pauta_core::{Index, Workspace};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Default)]
pub struct AppState {
    pub ws: RwLock<Option<Workspace>>,
    pub index: RwLock<Option<Index>>,
    pub load_error: RwLock<Option<String>>,
    pub watcher: Mutex<Option<notify::RecommendedWatcher>>,
    /// Serializes access to `.pauta/reminders.json` between the scheduler and
    /// notification action threads.
    pub reminders_lock: Mutex<()>,
}

impl AppState {
    pub fn workspace(&self) -> Option<Workspace> {
        self.ws.read().unwrap().clone()
    }

    pub fn index(&self) -> Option<Index> {
        self.index.read().unwrap().clone()
    }
}

/// Re-reads the data folder and tells every window and the tray.
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    let Some(ws) = state.workspace() else { return };
    match ws.load() {
        Ok(index) => {
            *state.index.write().unwrap() = Some(index);
            *state.load_error.write().unwrap() = None;
        }
        Err(e) => *state.load_error.write().unwrap() = Some(e.to_string()),
    }
    crate::tray::update(app);
    let _ = app.emit("pauta://changed", ());
}

/// Switches to a data folder: opens it, watches it, and loads it.
pub fn activate(app: &AppHandle, root: &Path) -> pauta_core::Result<()> {
    let ws = Workspace::open(root)?;
    // Debug builds honor PAUTA_NOW so end-to-end tests can simulate the clock.
    let ws = if cfg!(debug_assertions) {
        ws.with_clock(pauta_core::locate::clock())
    } else {
        ws
    };
    let state = app.state::<AppState>();
    *state.ws.write().unwrap() = Some(ws.clone());
    *state.watcher.lock().unwrap() = crate::watcher::watch(app.clone(), ws.root());
    if let Err(e) = ws.auto_archive() {
        eprintln!("arquivamento automático falhou: {e}");
    }
    refresh(app);
    Ok(())
}
