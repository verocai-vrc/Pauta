//! Pauta desktop app. Command-line flags:
//! `--hidden` start in the tray (used by autostart), `--capture` open quick capture.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod notifier;
mod settings;
mod state;
mod tray;
mod watcher;

use pauta_core::locate;
use pauta_core::yaml::Y;
use tauri::{AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::state::AppState;

/// Global shortcut for quick capture. Global shortcuts only work on X11; on
/// Wayland, bind a GNOME custom shortcut to `pauta --capture` instead.
const CAPTURE_SHORTCUT: &str = "ctrl+alt+p";

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn show_capture(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("capture") {
        w.show()?;
        return w.set_focus();
    }
    WebviewWindowBuilder::new(app, "capture", WebviewUrl::App("index.html?capture=1".into()))
        .title("Capturar ideia — Pauta")
        .inner_size(520.0, 280.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .build()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let hidden = args.iter().any(|a| a == "--hidden");
    let capture = args.iter().any(|a| a == "--capture");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|a| a == "--capture") {
                let _ = show_capture(app);
            } else {
                show_main(app);
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::setup_workspace,
            commands::get_index,
            commands::get_today,
            commands::get_agenda,
            commands::get_rhythm,
            commands::export_ics,
            commands::preview_metrics_import,
            commands::import_metrics,
            commands::find_slots,
            commands::create_card,
            commands::update_card,
            commands::move_card,
            commands::schedule_post,
            commands::mark_published,
            commands::duplicate_card,
            commands::archive_card,
            commands::trash_card,
            commands::create_event,
            commands::update_event,
            commands::archive_event,
            commands::trash_event,
            commands::create_board,
            commands::update_board,
            commands::set_board_order,
            commands::archive_board,
            commands::list_trash,
            commands::restore_trash,
            commands::list_archive,
            commands::restore_archived,
            commands::update_config,
            commands::copy_post_text,
            commands::request_art,
            commands::set_art_status,
            commands::attach_assets,
            commands::copy_art_request,
            commands::open_art_tool,
            commands::copy_text,
            commands::list_assets,
            commands::open_assets,
            commands::open_path,
            commands::open_url,
            commands::get_preferences,
            commands::set_preference,
            commands::mcp_snippet,
            commands::open_capture,
            commands::close_capture,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let root = locate::resolve_root();
            if root.join(pauta_core::store::CONFIG_FILE).exists()
                && let Err(e) = state::activate(&handle, &root)
            {
                *handle.state::<AppState>().load_error.write().unwrap() = Some(e.to_string());
            }
            if let Err(e) = tray::build(&handle) {
                eprintln!("bandeja do sistema indisponível: {e}");
            }
            tray::update(&handle);
            // Autostart is on by default (planning decision); the author can turn it off.
            // Only release builds register themselves, so dev runs never end up in
            // ~/.config/autostart pointing at target/debug.
            if cfg!(not(debug_assertions)) && !settings::flag("autostart_initialized") {
                let _ = handle.autolaunch().enable();
                let _ = settings::set("autostart_initialized", Y::Bool(true));
            }
            if let Err(e) = handle.global_shortcut().on_shortcut(CAPTURE_SHORTCUT, |app, _, event| {
                if event.state == ShortcutState::Pressed {
                    let _ = show_capture(app);
                }
            }) {
                eprintln!("atalho global {CAPTURE_SHORTCUT} indisponível: {e}");
            }
            notifier::start(handle.clone());
            if !hidden || handle.state::<AppState>().workspace().is_none() {
                show_main(&handle);
            }
            if capture {
                show_capture(&handle)?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window keeps Pauta in the tray so reminders still arrive.
            if let WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "main"
                && settings::load().close_to_tray
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("erro ao iniciar o Pauta");

    app.run(|app, event| {
        // Closing the quick-capture window while the main one is hidden must not quit.
        if let RunEvent::ExitRequested { api, code: None, .. } = event
            && settings::load().close_to_tray
            && app.get_webview_window("main").is_some()
        {
            api.prevent_exit();
        }
    });
}
