//! System tray (section 8.3): next items at a glance, plus quick actions.
//! On Ubuntu this uses the AppIndicator support that ships enabled.

use std::sync::Mutex;

use pauta_core::schedule::{AgendaKind, all_items};
use pauta_core::time::fmt_hm;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::state::AppState;

pub struct TrayItems {
    next_post: MenuItem<Wry>,
    next_deadline: MenuItem<Wry>,
}

pub struct TrayState(pub Mutex<Option<TrayItems>>);

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let next_post = MenuItem::with_id(app, "next_post", "Próximo post: —", false, None::<&str>)?;
    let next_deadline = MenuItem::with_id(app, "next_deadline", "Próximo prazo: —", false, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Abrir o Pauta", true, None::<&str>)?;
    let today = MenuItem::with_id(app, "today", "Hoje", true, None::<&str>)?;
    let capture = MenuItem::with_id(app, "capture", "Capturar ideia…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &next_post,
            &next_deadline,
            &PredefinedMenuItem::separator(app)?,
            &open,
            &today,
            &capture,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id("pauta").menu(&menu).tooltip("Pauta");
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => crate::show_main(app),
            "today" => {
                crate::show_main(app);
                let _ = app.emit("pauta://navigate", "today");
            }
            "capture" => {
                let _ = crate::show_capture(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    app.manage(TrayState(Mutex::new(Some(TrayItems {
        next_post,
        next_deadline,
    }))));
    Ok(())
}

/// Refreshes the "next post / next deadline" lines from the current index.
pub fn update(app: &AppHandle) {
    let Some(tray) = app.try_state::<TrayState>() else {
        return;
    };
    let state = app.state::<AppState>();
    let (Some(index), Some(ws)) = (state.index(), state.workspace()) else {
        return;
    };
    let now = ws.now();
    let tz = index.config.tz.clone();
    let today = now.to_zoned(tz.clone()).date();
    let items = all_items(&index);
    let label = |i: &pauta_core::schedule::AgendaItem| {
        let day = if i.when.date == today {
            "hoje".to_string()
        } else {
            i.when.date.strftime("%d/%m").to_string()
        };
        let time = i.when.time.map(|t| format!(" {}", fmt_hm(t))).unwrap_or_default();
        let net = i.network.as_ref().map(|n| format!(" [{n}]")).unwrap_or_default();
        let mut title = i.title.clone();
        if title.chars().count() > 40 {
            title = title.chars().take(39).collect::<String>() + "…";
        }
        format!("{day}{time}{net} · {title}")
    };
    let next_post = items
        .iter()
        .find(|i| i.kind == AgendaKind::Publication && !i.done && i.when.at.is_some_and(|a| a >= now))
        .map(label)
        .unwrap_or_else(|| "nada agendado".into());
    let next_deadline = items
        .iter()
        .find(|i| i.kind != AgendaKind::Publication && !i.done && i.when.date >= today)
        .map(label)
        .unwrap_or_else(|| "nenhum".into());
    if let Some(t) = tray.0.lock().unwrap().as_ref() {
        let _ = t.next_post.set_text(format!("Próximo post: {next_post}"));
        let _ = t.next_deadline.set_text(format!("Próximo prazo: {next_deadline}"));
    }
}
