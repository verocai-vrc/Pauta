//! Desktop notifications (sections 9.3 to 9.5).
//!
//! A background thread wakes every 20 seconds and asks `pauta-core` which
//! reminders are due. A fixed tick (rather than sleeping until the next
//! reminder) also covers suspend/resume and clock changes for free: on the first
//! tick after waking, anything that passed while asleep comes back as "missed".
//! See docs/adr/0005-notificacoes.md.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use jiff::SignedDuration;
use notify_rust::{Notification, Timeout};
use pauta_core::reminders::ReminderState;
use pauta_core::schedule::{AgendaKind, Reminder};
use pauta_core::time::fmt_hm;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::state::AppState;

const TICK: Duration = Duration::from_secs(20);

/// Action buttons need a thread parked in `wait_for_action` until the notification
/// is closed. Notifications nobody clicks would otherwise pile up threads for as
/// long as the app runs, so the number of parked threads is capped; past the cap a
/// notification is still shown, just without buttons and without a waiter.
const MAX_WAITERS: usize = 16;
static WAITERS: AtomicUsize = AtomicUsize::new(0);

/// Servers that honor the expiry close the notification, which also ends the wait.
/// Reminders missed this way still show up in the Today screen.
const EXPIRE: Timeout = Timeout::Milliseconds(30 * 60 * 1000);

/// Holds one of the `MAX_WAITERS` slots; released when the waiter thread ends.
struct WaiterSlot;

impl Drop for WaiterSlot {
    fn drop(&mut self) {
        WAITERS.fetch_sub(1, Ordering::SeqCst);
    }
}

fn reserve_waiter() -> Option<WaiterSlot> {
    WAITERS
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
            (n < MAX_WAITERS).then_some(n + 1)
        })
        .ok()
        .map(|_| WaiterSlot)
}

/// Shows `n` and runs `on_action` with the name of the clicked action.
fn deliver(mut n: Notification, on_action: impl FnOnce(&str) + Send + 'static) {
    n.timeout(EXPIRE);
    let Some(slot) = reserve_waiter() else {
        n.actions.clear();
        if let Err(e) = n.show() {
            eprintln!("notificação falhou: {e}");
        }
        return;
    };
    std::thread::spawn(move || {
        let _slot = slot;
        match n.show() {
            Ok(handle) => handle.wait_for_action(on_action),
            Err(e) => eprintln!("notificação falhou: {e}"),
        }
    });
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            tick(&app);
            crate::tray::update(&app);
            std::thread::sleep(TICK);
        }
    });
}

fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (Some(ws), Some(index)) = (state.workspace(), state.index()) else {
        return;
    };
    let _guard = state.reminders_lock.lock().unwrap();
    let mut rs = ReminderState::load(ws.root());
    let now = ws.now();
    let due = rs.due(&index, now);
    if due.fresh.is_empty() && due.missed_keys.is_empty() {
        return;
    }
    for r in &due.fresh {
        show(app, r);
    }
    if !due.missed.is_empty() {
        show_missed(app, &due.missed);
    }
    rs.mark_delivered(due.all_keys().collect::<Vec<_>>(), now);
    if let Err(e) = rs.save(ws.root(), now) {
        eprintln!("não foi possível salvar o estado dos lembretes: {e}");
    }
}

fn network_label(app: &AppHandle, id: &str) -> String {
    app.state::<AppState>()
        .index()
        .and_then(|i| i.config.network(id).map(|n| n.label.clone()))
        .unwrap_or_else(|| id.to_string())
}

fn texts(app: &AppHandle, r: &Reminder) -> (String, String) {
    let item = &r.item;
    let at = item.when.time.map(|t| format!(" às {}", fmt_hm(t))).unwrap_or_default();
    match item.kind {
        AgendaKind::Publication => {
            let net = item
                .network
                .as_deref()
                .map(|n| network_label(app, n))
                .unwrap_or_default();
            let summary = if r.spec == "0" {
                format!("Hora de postar no {net}")
            } else {
                format!("Post no {net}{at}")
            };
            (summary, item.title.clone())
        }
        AgendaKind::Due => (
            format!("Prazo: {}", item.when.date.strftime("%d/%m")),
            format!("{}{at}", item.title),
        ),
        AgendaKind::Event => {
            let kind = match item.status.as_deref() {
                Some("deadline") => "Prazo",
                Some("milestone") => "Marco",
                Some("launch") => "Lançamento",
                _ => "Evento",
            };
            (
                format!("{kind}: {}", item.when.date.strftime("%d/%m")),
                format!("{}{at}", item.title),
            )
        }
    }
}

fn show(app: &AppHandle, r: &Reminder) {
    let (summary, body) = texts(app, r);
    let mut n = Notification::new();
    n.appname("Pauta")
        .summary(&summary)
        .body(&body)
        .icon("appointment-soon")
        .action("default", "Abrir")
        .action("open", "Abrir");
    let is_post = r.item.kind == AgendaKind::Publication;
    if is_post {
        n.action("copy", "Copiar texto");
    }
    n.action("snooze15", "Adiar 15 min").action("snooze60", "Adiar 1 h");
    let app = app.clone();
    let r = r.clone();
    deliver(n, move |action| match action {
        "default" | "open" => open_item(&app, &r),
        "copy" => copy_post(&app, &r.item.id),
        "snooze15" => snooze(&app, &r.key, 15),
        "snooze60" => snooze(&app, &r.key, 60),
        _ => {}
    });
}

fn show_missed(app: &AppHandle, missed: &[Reminder]) {
    let posts = missed.iter().filter(|r| r.item.kind == AgendaKind::Publication).count();
    let summary = match (missed.len(), posts) {
        (1, 1) => "1 post atrasado".to_string(),
        (n, p) if n == p => format!("{n} posts atrasados"),
        (1, _) => "1 lembrete perdido".to_string(),
        (n, _) => format!("{n} lembretes perdidos"),
    };
    let body = missed
        .iter()
        .take(5)
        .map(|r| format!("• {}", r.item.title))
        .collect::<Vec<_>>()
        .join("\n");
    let mut n = Notification::new();
    n.appname("Pauta")
        .summary(&summary)
        .body(&body)
        .icon("appointment-missed")
        .action("default", "Ver em Hoje")
        .action("today", "Ver em Hoje");
    let app = app.clone();
    deliver(n, move |action| {
        if matches!(action, "default" | "today") {
            crate::show_main(&app);
            let _ = app.emit("pauta://navigate", "today");
        }
    });
}

fn open_item(app: &AppHandle, r: &Reminder) {
    crate::show_main(app);
    let kind = if r.item.kind == AgendaKind::Event {
        "event"
    } else {
        "card"
    };
    let _ = app.emit("pauta://open", json!({ "kind": kind, "id": r.item.id }));
}

fn copy_post(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    let Some(index) = state.index() else { return };
    if let Some(card) = index.card(id) {
        let text = crate::commands::post_text_of(&index, card);
        let _ = app.clipboard().write_text(text);
    }
}

fn snooze(app: &AppHandle, key: &str, minutes: i64) {
    let state = app.state::<AppState>();
    let Some(ws) = state.workspace() else { return };
    let _guard = state.reminders_lock.lock().unwrap();
    let mut rs = ReminderState::load(ws.root());
    let now = ws.now();
    rs.snooze(key, now + SignedDuration::from_mins(minutes));
    let _ = rs.save(ws.root(), now);
}
