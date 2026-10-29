//! Board templates (section 6.2) and the first-run seed.

use crate::model::{BoardKind, Column, ColumnRole};

fn col(id: &str, name: &str, role: Option<ColumnRole>) -> Column {
    Column {
        id: id.into(),
        name: name.into(),
        role,
        limit: None,
    }
}

pub fn columns_for(kind: BoardKind) -> Vec<Column> {
    use ColumnRole::*;
    match kind {
        BoardKind::Content => vec![
            col("idea", "Ideia", Some(Inbox)),
            col("draft", "Rascunho", None),
            col("producing", "Produção", None),
            col("scheduled", "Agendado", Some(Scheduled)),
            col("published", "Publicado", Some(Done)),
        ],
        BoardKind::Project => vec![
            col("backlog", "Backlog", Some(Inbox)),
            col("todo", "A fazer", None),
            col("doing", "Fazendo", None),
            col("review", "Revisão", None),
            col("done", "Concluído", Some(Done)),
        ],
        BoardKind::Generic => vec![
            col("todo", "A fazer", Some(Inbox)),
            col("doing", "Fazendo", None),
            col("done", "Feito", Some(Done)),
        ],
    }
}

pub const PALETTE: [&str; 6] = ["#7a2e3b", "#2e5a7a", "#3b7a2e", "#7a5a2e", "#5a2e7a", "#2e7a6e"];

pub struct Seed {
    pub timezone: String,
    pub boards: Vec<(String, BoardKind)>,
}

/// The boards agreed in the planning interview: one content board plus the
/// author's current projects.
pub fn default_seed() -> Seed {
    Seed {
        timezone: "America/Recife".into(),
        boards: vec![
            ("Conteúdo".into(), BoardKind::Content),
            ("TCC2".into(), BoardKind::Project),
            ("Dungeon Meister".into(), BoardKind::Project),
            ("Abstract-Tex".into(), BoardKind::Project),
            ("Pauta".into(), BoardKind::Project),
        ],
    }
}
