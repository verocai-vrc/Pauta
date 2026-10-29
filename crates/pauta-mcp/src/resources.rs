//! MCP resources (section 11.6): read-only views Claude can pull in without a tool call.
//!
//! | URI                    | What                                              |
//! |------------------------|---------------------------------------------------|
//! | `pauta://today`        | the day's summary                                 |
//! | `pauta://boards`       | boards, columns and card counts                   |
//! | `pauta://rhythm`       | cadence progress and publication history          |
//! | `pauta://agenda`       | everything dated in the next 14 days              |
//! | `pauta://card/{id}`    | one card as its Markdown file                     |
//! | `pauta://board/{id}`   | the cards of one board                            |
//!
//! The same builders back the matching tools, so a resource and its tool can't disagree.

use pauta_core::Workspace;
use rmcp::ErrorData;
use rmcp::model::{ReadResourceResult, Resource, ResourceContents, ResourceTemplate};
use serde_json::Value;

use crate::server::{Pauta, agenda_json, boards_json, card_summary, rhythm_json, today_json};

const JSON: &str = "application/json";

fn json_resource(name: &str, uri: &str, title: &str, description: &str) -> Resource {
    let mut r = Resource::new(uri, name).with_title(title).with_description(description);
    r.mime_type = Some(JSON.into());
    r
}

fn template(uri: &str, name: &str, title: &str, description: &str, mime: &str) -> ResourceTemplate {
    let mut t = ResourceTemplate::new(uri, name);
    t.title = Some(title.into());
    t.description = Some(description.into());
    t.mime_type = Some(mime.into());
    t
}

fn contents(uri: &str, mime: &str, text: String) -> ReadResourceResult {
    ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
        uri: uri.into(),
        mime_type: Some(mime.into()),
        text,
        meta: None,
    }])
}

fn as_json(uri: &str, v: &Value) -> ReadResourceResult {
    contents(uri, JSON, serde_json::to_string_pretty(v).unwrap_or_default())
}

fn internal(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

impl Pauta {
    pub(crate) fn resources() -> Vec<Resource> {
        vec![
            json_resource(
                "today",
                "pauta://today",
                "Hoje",
                "Resumo do dia: posts de hoje, prazos, atrasados, fila vazia, cartões parados e metas de cadência.",
            ),
            json_resource(
                "boards",
                "pauta://boards",
                "Quadros",
                "Quadros com colunas (id, nome, papel) e a contagem de cartões, mais as redes configuradas.",
            ),
            json_resource(
                "rhythm",
                "pauta://rhythm",
                "Ritmo",
                "Progresso das metas de cadência e histórico de posts por semana e por mês.",
            ),
            json_resource(
                "agenda",
                "pauta://agenda",
                "Agenda dos próximos 14 dias",
                "Publicações, prazos de cartões e eventos de calendário.",
            ),
        ]
    }

    pub(crate) fn resource_templates() -> Vec<ResourceTemplate> {
        vec![
            template(
                "pauta://card/{id}",
                "card",
                "Cartão",
                "Um cartão completo, como o arquivo Markdown (cabeçalho YAML e corpo).",
                "text/markdown",
            ),
            template(
                "pauta://board/{id}",
                "board",
                "Cartões de um quadro",
                "Resumo dos cartões de um quadro, por coluna.",
                JSON,
            ),
        ]
    }

    pub(crate) fn read(&self, uri: &str) -> Result<ReadResourceResult, ErrorData> {
        let not_found = || ErrorData::resource_not_found(format!("recurso desconhecido: {uri}"), None);
        let (ws, index) = self.index().map_err(internal)?;
        match uri {
            "pauta://today" => Ok(as_json(uri, &today_json(&ws, &index))),
            "pauta://boards" => Ok(as_json(uri, &boards_json(&index))),
            "pauta://rhythm" => Ok(as_json(uri, &rhythm_json(&ws, &index))),
            "pauta://agenda" => {
                let today = ws.now().to_zoned(index.config.tz.clone()).date();
                let to = today.saturating_add(jiff::ToSpan::days(14)).to_string();
                Ok(as_json(
                    uri,
                    &agenda_json(&ws, &index, None, Some(&to)).map_err(internal)?,
                ))
            }
            _ => {
                if let Some(id) = uri.strip_prefix("pauta://card/") {
                    let card = index.card(id).ok_or_else(not_found)?;
                    read_card_file(&ws, &card.path, uri)
                } else if let Some(id) = uri.strip_prefix("pauta://board/") {
                    let board = index.board(id).ok_or_else(not_found)?;
                    let columns: Vec<Value> = board
                        .columns
                        .iter()
                        .map(|c| {
                            serde_json::json!({
                                "id": c.id, "name": c.name,
                                "cards": index.column_cards(&board.id, &c.id).into_iter().map(card_summary).collect::<Vec<_>>(),
                            })
                        })
                        .collect();
                    Ok(as_json(
                        uri,
                        &serde_json::json!({ "id": board.id, "name": board.name, "columns": columns }),
                    ))
                } else {
                    Err(not_found())
                }
            }
        }
    }
}

/// Reads a card file, only from inside the data folder (section 11.4).
fn read_card_file(ws: &Workspace, path: &std::path::Path, uri: &str) -> Result<ReadResourceResult, ErrorData> {
    if !path.starts_with(ws.root()) {
        return Err(ErrorData::invalid_params("o cartão está fora da pasta de dados", None));
    }
    let text = std::fs::read_to_string(path).map_err(internal)?;
    Ok(contents(uri, "text/markdown", text))
}
