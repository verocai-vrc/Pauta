//! Format migrations (section 5.7).
//!
//! Each step upgrades the folder from version N to N+1. A full backup is taken
//! before the first step runs. Version 1 is the first format, so there are no
//! steps yet; the machinery is here so adding one later is a one-line change.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::SCHEMA_VERSION;
use crate::error::{Error, IoContext, Result, rule};
use crate::fields::get_i64;
use crate::store::{CONFIG_FILE, write_atomic};
use crate::yaml::{Y, YamlText};

type Step = fn(&Path) -> Result<()>;

/// `STEPS[i]` migrates from version `i + 1` to `i + 2`.
const STEPS: &[Step] = &[];

/// Returns the backup folder if a migration ran.
pub fn migrate(root: &Path) -> Result<Option<PathBuf>> {
    let cfg_path = root.join(CONFIG_FILE);
    let text = fs::read_to_string(&cfg_path).at(&cfg_path)?;
    let y = YamlText::new(text.as_str());
    let raw = y.parse().ok().and_then(|m| get_i64(&m, "schema_version")).unwrap_or(1);
    // Versions start at 1; `0`, negatives and values past u32 are hand-edit mistakes.
    let version = u32::try_from(raw).ok().filter(|v| *v >= 1).ok_or_else(|| {
        Error::Rule(format!(
            "schema_version inválido em {CONFIG_FILE}: {raw} (esperado um inteiro ≥ 1)"
        ))
    })?;
    if version > SCHEMA_VERSION {
        return rule(format!(
            "a pasta de dados usa o formato {version}, mais novo que o suportado ({SCHEMA_VERSION}); atualize o Pauta"
        ));
    }
    if version == SCHEMA_VERSION {
        return Ok(None);
    }
    let backup = root.join(".pauta").join(format!(
        "backup-v{version}-{}",
        jiff::Timestamp::now().strftime("%Y%m%dT%H%M%S")
    ));
    copy_tree(root, &backup)?;
    for v in version..SCHEMA_VERSION {
        let step = STEPS
            .get((v - 1) as usize)
            .ok_or_else(|| Error::Rule(format!("não há migração do formato {v} para {}", v + 1)))?;
        step(root)?;
    }
    let text = fs::read_to_string(&cfg_path).at(&cfg_path)?;
    let mut y = YamlText::new(text);
    y.set("schema_version", &Y::Int(SCHEMA_VERSION.into()));
    write_atomic(&cfg_path, y.as_str())?;
    Ok(Some(backup))
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to).at(to)?;
    for entry in fs::read_dir(from).at(from)?.flatten() {
        let name = entry.file_name();
        if name == ".pauta" || name == ".git" {
            continue;
        }
        let src = entry.path();
        let dst = to.join(&name);
        if src.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            fs::copy(&src, &dst).at(&src)?;
        }
    }
    Ok(())
}
