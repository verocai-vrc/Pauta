//! `pauta-core`: everything the Pauta app and MCP server share.
//!
//! - [`store::Workspace`] reads and writes the data folder (Markdown + YAML files).
//! - [`schedule`] answers "what is due, when is the next free slot, what is late".
//! - [`reminders`] tracks which reminders were delivered or snoozed.
//!
//! No dependency on Tauri, notifications or the real clock (see [`clock::Clock`]).

pub mod art;
pub mod clock;
pub mod config;
pub mod error;
pub mod fields;
pub mod frontmatter;
pub mod ics;
pub mod import;
pub mod locate;
pub mod markdown;
pub mod migrate;
pub mod model;
pub mod rank;
pub mod reminders;
pub mod rhythm;
pub mod schedule;
pub mod store;
pub mod templates;
pub mod time;
pub mod yaml;

pub use error::{Error, Result};
pub use store::{Index, Workspace};
