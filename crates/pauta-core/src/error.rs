use std::path::{Path, PathBuf};

/// Every fallible operation in `pauta-core` returns this error type.
///
/// We use `thiserror` in the library (typed errors the app and the MCP server can
/// match on) and leave `anyhow` to the binaries, where errors are only displayed.
/// See docs/adr/0006-erros-e-posse-em-rust.md.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("erro de E/S em {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("arquivo inválido {path}: {message}")]
    Invalid { path: PathBuf, message: String },

    /// The file on disk is not the version the caller started editing from.
    /// Nothing was written; the caller decides what to keep.
    #[error("o arquivo foi alterado por fora desde que foi lido: {path}")]
    Conflict {
        path: PathBuf,
        disk_content: String,
        disk_hash: String,
    },

    #[error("não encontrado: {0}")]
    NotFound(String),

    /// Bad input or a violated business rule (unknown column, bad date, ...).
    #[error("{0}")]
    Rule(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Attaches the offending path to `std::io::Error`s, so messages say *which* file failed.
pub(crate) trait IoContext<T> {
    fn at(self, path: &Path) -> Result<T>;
}

impl<T> IoContext<T> for std::io::Result<T> {
    fn at(self, path: &Path) -> Result<T> {
        self.map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

pub(crate) fn rule<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Rule(msg.into()))
}
