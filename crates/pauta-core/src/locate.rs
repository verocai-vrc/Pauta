//! Where the data folder is. Shared by the app and the MCP server so both
//! always look at the same place.
//!
//! Order: `PAUTA_ROOT` env var, then `data_root` in `~/.config/pauta/config.yaml`
//! (written by the app on first run), then `~/Documentos/Pauta`.

use std::path::PathBuf;

use crate::error::Result;
use crate::fields::get_str;
use crate::store::write_atomic;
use crate::yaml::{Y, YamlText};

fn home() -> PathBuf {
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from)
}

pub fn app_config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(".config"));
    base.join("pauta").join("config.yaml")
}

pub fn default_suggestion() -> PathBuf {
    home().join("Documentos").join("Pauta")
}

/// The data folder chosen in the app, if any.
pub fn saved_root() -> Option<PathBuf> {
    let text = std::fs::read_to_string(app_config_path()).ok()?;
    let m = YamlText::new(text).parse().ok()?;
    get_str(&m, "data_root").map(|s| expand(&s))
}

/// A fixed "now" for tests and fixtures, from `PAUTA_NOW` (RFC 3339, e.g.
/// `2026-10-28T21:00:00-03:00`). `None` when unset or malformed, meaning the real clock.
pub fn now_override() -> Option<jiff::Timestamp> {
    std::env::var("PAUTA_NOW").ok()?.parse().ok()
}

/// A workspace clock honoring [`now_override`].
pub fn clock() -> std::sync::Arc<dyn crate::clock::Clock> {
    match now_override() {
        Some(t) => std::sync::Arc::new(crate::clock::FixedClock::new(t)),
        None => std::sync::Arc::new(crate::clock::SystemClock),
    }
}

pub fn resolve_root() -> PathBuf {
    if let Some(p) = std::env::var_os("PAUTA_ROOT") {
        return expand(&p.to_string_lossy());
    }
    saved_root().unwrap_or_else(default_suggestion)
}

pub fn save_root(root: &std::path::Path) -> Result<()> {
    let path = app_config_path();
    let mut y = YamlText::new(std::fs::read_to_string(&path).unwrap_or_default());
    y.set("data_root", &Y::Quoted(root.to_string_lossy().into_owned()));
    write_atomic(&path, y.as_str())
}

pub fn expand(p: &str) -> PathBuf {
    match p.strip_prefix("~/") {
        Some(rest) => home().join(rest),
        None if p == "~" => home(),
        None => PathBuf::from(p),
    }
}
