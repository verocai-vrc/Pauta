//! Reloads when files change outside the app (an editor, git, the MCP server).

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use tauri::AppHandle;

/// Events are grouped for this long before reloading (section 14.3).
const DEBOUNCE: Duration = Duration::from_millis(300);

fn relevant(event: &notify::Event) -> bool {
    use notify::EventKind::*;
    matches!(event.kind, Create(_) | Modify(_) | Remove(_))
        && event.paths.iter().any(|p| {
            let s = p.to_string_lossy();
            // Our own state files and editors' temp files don't change the data.
            !s.contains("/.pauta/")
                && !s.contains("/.git/")
                && !p.file_name().is_some_and(|n| {
                    let n = n.to_string_lossy();
                    n.ends_with(".tmp") || n.ends_with('~') || n.ends_with(".swp")
                })
        })
}

pub fn watch(app: AppHandle, root: &Path) -> Option<notify::RecommendedWatcher> {
    let (tx, rx) = mpsc::channel::<()>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(ev) = res
            && relevant(&ev)
        {
            let _ = tx.send(());
        }
    })
    .map_err(|e| eprintln!("observador de arquivos indisponível: {e}"))
    .ok()?;
    if let Err(e) = watcher.watch(root, RecursiveMode::Recursive) {
        eprintln!("não foi possível observar {}: {e}", root.display());
        return None;
    }
    std::thread::spawn(move || {
        // Ends when the watcher (and with it `tx`) is dropped.
        while rx.recv().is_ok() {
            while rx.recv_timeout(DEBOUNCE).is_ok() {}
            crate::state::refresh(&app);
        }
    });
    Some(watcher)
}
