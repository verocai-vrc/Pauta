//! Per-machine app settings in `~/.config/pauta/config.yaml` (next to `data_root`).
//! Data-folder settings live in `pauta.yaml` instead, so they travel with the data.

use pauta_core::fields::get_str;
use pauta_core::locate::app_config_path;
use pauta_core::store::write_atomic;
use pauta_core::yaml::{Y, YamlText};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppSettings {
    pub close_to_tray: bool,
    pub theme: String,
}

fn read() -> YamlText {
    YamlText::new(std::fs::read_to_string(app_config_path()).unwrap_or_default())
}

pub fn load() -> AppSettings {
    let m = read().parse().unwrap_or_default();
    AppSettings {
        close_to_tray: get_str(&m, "close_to_tray").is_none_or(|v| v != "false"),
        theme: get_str(&m, "theme").unwrap_or_else(|| "system".into()),
    }
}

pub fn set(key: &str, value: Y) -> pauta_core::Result<()> {
    let mut y = read();
    y.set(key, &value);
    write_atomic(&app_config_path(), y.as_str())
}

pub fn flag(key: &str) -> bool {
    let m = read().parse().unwrap_or_default();
    get_str(&m, key).is_some_and(|v| v == "true")
}
