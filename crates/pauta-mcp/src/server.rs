//! The MCP tools (section 11.3). Each tool is a thin adapter over `pauta-core`:
//! parse arguments, call one `Workspace` method, return a compact JSON summary.
//! Safety (section 11.4): nothing is deleted permanently and nothing is posted.

use std::path::PathBuf;

use pauta_core::markdown::SectionMode;
use pauta_core::model::Card;
use pauta_core::schedule::{self, AgendaItem};
use pauta_core::store::{CardPatch, NewCard, NewEvent, PublicationPatch, SectionEdit};
use pauta_core::time::{When, fmt_hm};
use pauta_core::{Index, Workspace};
use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, ListResourceTemplatesResult, ListResourcesResult,
    PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResponse, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, prompt_handler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

const INSTRUCTIONS: &str = "Pauta é o organizador pessoal do Arthur: quadros (estilo Trello) de conteúdo e de projetos, \
calendário de prazos e publicações agendadas no LinkedIn e no Instagram. Cada cartão de conteúdo é de UMA rede \
(campo network); versões da mesma ideia para outras redes são cartões irmãos (pauta_duplicate_for_network). \
Fluxo de conteúdo: idea → draft → producing → scheduled → published. Arte: pauta_request_art registra o pedido e escreve a direção visual; o Pauta não tem API do Claude Design, então o autor cola o pedido lá e exporta os arquivos para a pasta de anexos (pauta_list_art_requests mostra onde), e pauta_mark_art_ready conclui. Datas aceitam AAAA-MM-DD (dia inteiro) ou \
AAAA-MM-DDTHH:MM no fuso configurado. Nada é publicado nas redes por estas ferramentas e nada é excluído \
definitivamente; comece por pauta_today ou pauta_list_boards para conhecer os ids.";

#[derive(Clone)]
pub struct Pauta {
    root: PathBuf,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

type ToolResult = Result<CallToolResult, ErrorData>;

fn ok(v: Value) -> ToolResult {
    Ok(CallToolResult::success(vec![ContentBlock::text(v.to_string())]))
}

/// Domain errors go back to Claude as tool errors it can read and act on.
fn fail(e: impl std::fmt::Display) -> ToolResult {
    Ok(CallToolResult::error(vec![ContentBlock::text(e.to_string())]))
}

macro_rules! tryt {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return fail(e),
        }
    };
}

fn when(w: &When) -> String {
    match w.time {
        Some(t) => format!("{} {}", w.date, fmt_hm(t)),
        None => w.date.to_string(),
    }
}

pub(crate) fn card_summary(c: &Card) -> Value {
    let mut v = json!({ "id": c.id, "board": c.board, "column": c.column, "title": c.title, "kind": c.kind });
    let m = v.as_object_mut().unwrap();
    if !c.tags.is_empty() {
        m.insert("tags".into(), json!(c.tags));
    }
    if let Some(n) = &c.network {
        m.insert("network".into(), json!(n));
    }
    if let Some(p) = &c.publication {
        m.insert(
            "publication".into(),
            json!({ "at": p.at.as_ref().map(when), "status": p.status.as_str(), "url": p.url }),
        );
    }
    if let Some(d) = &c.due {
        m.insert("due".into(), json!(when(d)));
    }
    if let Some(a) = &c.art {
        m.insert("art".into(), json!({ "status": a.status.as_str(), "format": a.format }));
    }
    if c.checklist.total > 0 {
        m.insert(
            "checklist".into(),
            json!(format!("{}/{}", c.checklist.done, c.checklist.total)),
        );
    }
    v
}

pub(crate) fn card_full(c: &Card) -> Value {
    let mut v = card_summary(c);
    let m = v.as_object_mut().unwrap();
    m.insert("rank".into(), json!(c.rank));
    m.insert(
        "links".into(),
        json!({ "board": c.links.board, "siblings": c.links.siblings, "article": c.links.article }),
    );
    m.insert("reminders".into(), json!(c.reminders));
    m.insert("updated".into(), json!(c.updated.map(|t| t.to_string())));
    if let Some(metrics) = &c.metrics {
        m.insert("metrics".into(), serde_json::to_value(metrics).unwrap_or(Value::Null));
    }
    m.insert("body".into(), json!(c.body));
    m.insert("assets".into(), json!(c.assets));
    m.insert("file".into(), json!(c.path));
    v
}

pub(crate) fn today_json(ws: &Workspace, index: &Index) -> Value {
    let t = schedule::today(index, ws.now());
    json!({
        "date": t.date.to_string(),
        "post_today": t.posts.iter().map(agenda_item).collect::<Vec<_>>(),
        "upcoming": t.upcoming.iter().map(agenda_item).collect::<Vec<_>>(),
        "overdue": t.overdue.iter().map(agenda_item).collect::<Vec<_>>(),
        "empty_queues": t.empty_queues.iter().map(|q| json!({
            "network": q.network, "days": q.days,
            "candidates": q.candidates.iter().map(|c| json!({"id": c.id, "title": c.title, "column": c.column})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "stale": t.stale.iter().map(|c| json!({"id": c.id, "board": c.board, "title": c.title, "column": c.column})).collect::<Vec<_>>(),
        "cadence": t.cadence.iter().map(cadence_json).collect::<Vec<_>>(),
        "invalid_files": t.invalid_files,
    })
}

pub(crate) fn rhythm_json(ws: &Workspace, index: &Index) -> Value {
    let r = pauta_core::rhythm::rhythm(index, ws.now());
    let bucket =
        |b: &pauta_core::rhythm::Bucket| json!({ "from": b.start.to_string(), "counts": b.counts, "total": b.total });
    json!({
        "cadence": r.cadence.iter().map(cadence_json).collect::<Vec<_>>(),
        "weeks": r.history.weeks.iter().map(bucket).collect::<Vec<_>>(),
        "months": r.history.months.iter().map(bucket).collect::<Vec<_>>(),
        "total_published": r.history.total_posts,
        "insights": serde_json::to_value(&r.insights).unwrap_or(Value::Null),
        "recent": r.recent.iter().take(10).map(|p| json!({
            "id": p.id, "title": p.title, "network": p.network, "date": p.date.to_string(), "url": p.url,
        })).collect::<Vec<_>>(),
    })
}

pub(crate) fn boards_json(index: &Index) -> Value {
    let boards: Vec<Value> = index
        .boards
        .iter()
        .map(|b| {
            json!({
                "id": b.id, "name": b.name, "kind": b.kind,
                "columns": b.columns.iter().map(|c| json!({
                    "id": c.id, "name": c.name, "role": c.role,
                    "cards": index.column_cards(&b.id, &c.id).len(),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let networks: Vec<&str> = index.config.networks.iter().map(|n| n.id.as_str()).collect();
    json!({ "boards": boards, "networks": networks, "timezone": index.config.timezone })
}

/// Everything dated from `from` to `to` (defaults: today, and a week later).
pub(crate) fn agenda_json(
    ws: &Workspace,
    index: &Index,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Value, String> {
    let today = ws.now().to_zoned(index.config.tz.clone()).date();
    let from = from
        .map_or(Ok(today), str::parse::<jiff::civil::Date>)
        .map_err(|e| e.to_string())?;
    let to = to
        .map_or_else(
            || Ok(from.saturating_add(jiff::ToSpan::days(7))),
            str::parse::<jiff::civil::Date>,
        )
        .map_err(|e| e.to_string())?;
    let items: Vec<Value> = schedule::agenda(index, from, to).iter().map(agenda_item).collect();
    Ok(json!({ "from": from.to_string(), "to": to.to_string(), "items": items }))
}

pub(crate) fn problem_json(p: &pauta_core::art::Problem) -> Value {
    let text = match p.code {
        "no_text" => "o post não tem texto",
        "over_limit" => "o texto passa do limite de caracteres da rede",
        "media_missing" => "a rede exige imagem e não há arquivos nos anexos",
        "art_pending" => "a arte foi pedida e ainda não está marcada como pronta",
        other => other,
    };
    json!({ "code": p.code, "detail": p.detail, "meaning": text })
}

/// An art request as Claude needs it: what to make, how, and where the files go.
pub(crate) fn art_json(ws: &Workspace, index: &Index, c: &Card) -> Value {
    let net = c.network.as_deref().and_then(|n| index.config.network(n));
    let format = net.and_then(|n| pauta_core::art::format_for(n, c));
    json!({
        "id": c.id,
        "title": c.title,
        "network": c.network,
        "status": c.art.as_ref().map(|a| a.status.as_str()),
        "format": format.map(|f| json!({ "id": f.id, "label": f.label, "size": f.size, "note": f.note })),
        "brief": pauta_core::markdown::section_text(&c.body, "Arte"),
        "post_text": pauta_core::art::text_for_posting(index, c),
        "assets_dir": ws.assets_dir(c),
        "files": c.assets,
        "request_text": pauta_core::art::request_text(index, c).ok(),
    })
}

pub(crate) fn cadence_json(c: &pauta_core::rhythm::CadenceProgress) -> Value {
    json!({
        "network": c.network,
        "goal": format!("{} por {}", c.goal, match c.per { pauta_core::config::Period::Week => "semana", pauta_core::config::Period::Month => "mês" }),
        "published": c.done,
        "scheduled": c.scheduled,
        "missing": c.goal.saturating_sub(c.done + c.scheduled),
        "period": format!("{} a {}", c.period_start, c.period_end),
    })
}

pub(crate) fn agenda_item(i: &AgendaItem) -> Value {
    let mut v = json!({ "type": i.kind, "id": i.id, "title": i.title, "when": when(&i.when) });
    let m = v.as_object_mut().unwrap();
    for (k, val) in [
        ("board", &i.board),
        ("network", &i.network),
        ("status", &i.status),
        ("column", &i.column),
    ] {
        if let Some(val) = val {
            m.insert(k.into(), json!(val));
        }
    }
    if let Some(e) = &i.end {
        m.insert("end".into(), json!(when(e)));
    }
    if i.done {
        m.insert("done".into(), json!(true));
    }
    v
}

/// `""` clears a field, anything else sets it.
fn clearable(v: &Option<String>) -> Option<Option<String>> {
    v.as_ref().map(|s| (!s.trim().is_empty()).then(|| s.clone()))
}

#[derive(Deserialize, JsonSchema)]
pub struct Empty {}

#[derive(Deserialize, JsonSchema)]
pub struct IdArg {
    /// Card id (from pauta_list_cards or pauta_today).
    pub id: String,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct ListCards {
    /// Board id. Omit for all boards.
    pub board: Option<String>,
    /// Column id, e.g. `draft`.
    pub column: Option<String>,
    pub tag: Option<String>,
    /// `linkedin` or `instagram`.
    pub network: Option<String>,
    /// Case-insensitive search in title and body.
    pub text: Option<String>,
    /// Only cards with a publication or due date on/after this date (AAAA-MM-DD).
    pub from: Option<String>,
    /// Only cards with a publication or due date on/before this date (AAAA-MM-DD).
    pub to: Option<String>,
    /// Maximum number of cards (default 50).
    pub limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AddIdea {
    pub title: String,
    /// Free notes, stored in a `## Notas` section.
    pub notes: Option<String>,
    /// Content board id. Defaults to the first content board.
    pub board: Option<String>,
    pub network: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateCard {
    pub board: String,
    pub title: String,
    /// Column id. Defaults to the board's first (inbox) column.
    pub column: Option<String>,
    /// `post`, `article`, `task` or `note`. Defaults by board kind.
    pub kind: Option<String>,
    /// Network of a content card (`linkedin` / `instagram`).
    pub network: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Markdown body. For posts, the text to publish; put private notes under `## Notas`.
    pub body: Option<String>,
    /// Deadline: AAAA-MM-DD or AAAA-MM-DDTHH:MM.
    pub due: Option<String>,
    /// Planned publication time (AAAA-MM-DDTHH:MM). Requires `network`.
    pub publication_at: Option<String>,
    /// Reminder offsets like `-1h`, `0`, `-7d`, `09:00`. Defaults come from pauta.yaml.
    pub reminders: Option<Vec<String>>,
    /// Related project board id.
    pub related_board: Option<String>,
    /// For a promo post: id of the article card it promotes.
    pub article: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Section {
    /// Section heading without `##`, e.g. `Notas`.
    pub title: String,
    pub text: String,
    pub mode: Mode,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Replace,
    Append,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateCard {
    pub id: String,
    pub title: Option<String>,
    /// Moves to the bottom of this column. Use pauta_move_card to choose the position.
    pub column: Option<String>,
    pub kind: Option<String>,
    /// Replaces all tags.
    pub tags: Option<Vec<String>>,
    /// `""` removes the network.
    pub network: Option<String>,
    /// AAAA-MM-DD or AAAA-MM-DDTHH:MM; `""` removes the deadline.
    pub due: Option<String>,
    pub reminders: Option<Vec<String>>,
    /// Replaces the whole Markdown body.
    pub body: Option<String>,
    /// Replaces or appends to one `## Section` of the body, creating it if missing.
    pub section: Option<Section>,
    /// AAAA-MM-DDTHH:MM; `""` clears the planned time.
    pub publication_at: Option<String>,
    /// `planned`, `scheduled`, `published` or `skipped`.
    pub publication_status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MoveCard {
    pub id: String,
    /// Target column id.
    pub column: String,
    /// 0 = top of the column. Omit for the bottom.
    pub position: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SchedulePost {
    pub id: String,
    /// AAAA-MM-DDTHH:MM. Omit to use the network's next free posting slot.
    pub at: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FindSlots {
    pub network: String,
    /// How many slots (default 5).
    pub count: Option<usize>,
    /// Search after this time (AAAA-MM-DDTHH:MM). Default: now.
    pub after: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Range {
    /// AAAA-MM-DD. Default: today.
    pub from: Option<String>,
    /// AAAA-MM-DD, inclusive. Default: 7 days after `from`.
    pub to: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateEvent {
    pub title: String,
    /// AAAA-MM-DD for all-day items (deadlines), or AAAA-MM-DDTHH:MM.
    pub date: String,
    /// `deadline`, `event`, `milestone` or `launch`.
    pub kind: Option<String>,
    pub end: Option<String>,
    /// e.g. `["-7d", "-1d", "09:00"]`. Defaults to deadline_default in pauta.yaml.
    pub reminders: Option<Vec<String>>,
    /// Related board id, e.g. `tcc2`.
    pub board: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarkPublished {
    pub id: String,
    /// Link to the published post.
    pub url: Option<String>,
    /// When it was posted (AAAA-MM-DDTHH:MM). Default: now.
    pub at: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RecordMetrics {
    pub id: String,
    /// Numbers as shown in the network's own analytics. Send only what you have; the whole block is replaced.
    pub impressions: Option<u64>,
    pub reactions: Option<u64>,
    pub comments: Option<u64>,
    pub shares: Option<u64>,
    pub saves: Option<u64>,
    pub clicks: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RequestArt {
    pub id: String,
    /// Format id of the card's network, e.g. `image`, `carousel`, `story`. Default: the card's current one, or the network's first.
    pub format: Option<String>,
    /// The visual direction (what to show, tone, colors, references). It is written to the card's `## Arte` section.
    pub brief: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Duplicate {
    pub id: String,
    /// The other network, e.g. `instagram`.
    pub network: String,
}

impl Pauta {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    /// Opened per call so edits made in the app or by hand are always seen.
    pub(crate) fn ws(&self, tool: &str) -> pauta_core::Result<Workspace> {
        Ok(Workspace::open(&self.root)?.with_actor(format!("mcp:{tool}")))
    }

    pub(crate) fn index(&self) -> pauta_core::Result<(Workspace, Index)> {
        let ws = self.ws("read")?;
        let index = ws.load()?;
        Ok((ws, index))
    }
}

#[tool_router]
impl Pauta {
    #[tool(
        description = "Resumo do dia: posts de hoje, prazos dos próximos 7 dias, itens atrasados, redes sem nada agendado (fila vazia) e cartões parados. Bom ponto de partida.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_today(&self, _: Parameters<Empty>) -> ToolResult {
        let (ws, index) = tryt!(self.index());
        ok(today_json(&ws, &index))
    }

    #[tool(
        description = "Ritmo de publicação: progresso da meta de cadência de cada rede no período atual (ex.: 1 de 2 posts esta semana, mais os agendados) e o histórico de posts publicados por semana (12) e por mês (6). Use para sugerir o que falta publicar, sem cobrar.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_rhythm(&self, _: Parameters<Empty>) -> ToolResult {
        let (ws, index) = tryt!(self.index());
        ok(rhythm_json(&ws, &index))
    }

    #[tool(
        description = "Lista os quadros com suas colunas (id, nome, papel) e a contagem de cartões por coluna, mais as redes configuradas.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_list_boards(&self, _: Parameters<Empty>) -> ToolResult {
        let (_, index) = tryt!(self.index());
        ok(boards_json(&index))
    }

    #[tool(
        description = "Lista cartões com filtros opcionais (quadro, coluna, etiqueta, rede, texto, intervalo de datas). Retorna resumos; use pauta_get_card para o texto completo.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_list_cards(&self, Parameters(f): Parameters<ListCards>) -> ToolResult {
        let (_, index) = tryt!(self.index());
        let from = tryt!(f.from.as_deref().map(str::parse::<jiff::civil::Date>).transpose());
        let to = tryt!(f.to.as_deref().map(str::parse::<jiff::civil::Date>).transpose());
        let text = f.text.as_ref().map(|t| t.to_lowercase());
        let cards: Vec<Value> = index
            .cards
            .iter()
            .filter(|c| f.board.as_ref().is_none_or(|b| *b == c.board))
            .filter(|c| f.column.as_ref().is_none_or(|col| *col == c.column))
            .filter(|c| f.tag.as_ref().is_none_or(|t| c.tags.contains(t)))
            .filter(|c| f.network.as_ref().is_none_or(|n| c.network.as_ref() == Some(n)))
            .filter(|c| {
                text.as_ref()
                    .is_none_or(|t| c.title.to_lowercase().contains(t) || c.body.to_lowercase().contains(t))
            })
            .filter(|c| {
                if from.is_none() && to.is_none() {
                    return true;
                }
                let dates = c
                    .publication
                    .as_ref()
                    .and_then(|p| p.at.as_ref())
                    .into_iter()
                    .chain(c.due.as_ref());
                dates
                    .map(|w| w.date)
                    .any(|d| from.is_none_or(|f| d >= f) && to.is_none_or(|t| d <= t))
            })
            .take(f.limit.unwrap_or(50))
            .map(card_summary)
            .collect();
        ok(json!({ "count": cards.len(), "cards": cards }))
    }

    #[tool(
        description = "Lê um cartão completo: campos, links, lembretes e o corpo em Markdown.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_get_card(&self, Parameters(a): Parameters<IdArg>) -> ToolResult {
        let ws = tryt!(self.ws("read"));
        ok(card_full(&tryt!(ws.card(&a.id))))
    }

    #[tool(description = "Captura rápida de uma ideia na coluna inicial (Ideia) de um quadro de conteúdo.")]
    async fn pauta_add_idea(&self, Parameters(a): Parameters<AddIdea>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_add_idea"));
        let board = match a.board {
            Some(b) => b,
            None => {
                let index = tryt!(ws.load());
                let content = index
                    .boards
                    .iter()
                    .find(|b| b.kind == pauta_core::model::BoardKind::Content);
                tryt!(
                    content
                        .map(|b| b.id.clone())
                        .ok_or("nenhum quadro de conteúdo encontrado")
                )
            }
        };
        let body = a.notes.map(|n| format!("## Notas\n\n{}", n.trim()));
        let card = tryt!(ws.create_card(
            &board,
            &NewCard {
                title: a.title,
                network: a.network,
                tags: a.tags,
                body,
                ..Default::default()
            }
        ));
        ok(card_summary(&card))
    }

    #[tool(
        description = "Cria um cartão em qualquer quadro, com coluna, rede, corpo, prazo e horário de publicação opcionais."
    )]
    async fn pauta_create_card(&self, Parameters(a): Parameters<CreateCard>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_create_card"));
        let card = tryt!(ws.create_card(
            &a.board,
            &NewCard {
                title: a.title,
                column: a.column,
                kind: a.kind,
                network: a.network,
                tags: a.tags,
                body: a.body,
                due: a.due,
                publication_at: a.publication_at,
                reminders: a.reminders,
                links_board: a.related_board,
                links_article: a.article,
                top: false,
            }
        ));
        ok(card_summary(&card))
    }

    #[tool(
        description = "Altera campos de um cartão e/ou o corpo: substitui o corpo inteiro (body) ou só uma seção ## (section, com mode replace ou append). Campos omitidos não mudam; \"\" limpa due, network e publication_at."
    )]
    async fn pauta_update_card(&self, Parameters(a): Parameters<UpdateCard>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_update_card"));
        let publication = if a.publication_at.is_some() || a.publication_status.is_some() {
            Some(Some(PublicationPatch {
                at: clearable(&a.publication_at),
                status: a.publication_status,
                url: None,
            }))
        } else {
            None
        };
        let patch = CardPatch {
            title: a.title,
            kind: a.kind,
            column: a.column,
            tags: a.tags,
            network: clearable(&a.network),
            due: clearable(&a.due),
            reminders: a.reminders.map(Some),
            publication,
            links_board: None,
            metrics: None,
            body: a.body,
            section: a.section.map(|s| SectionEdit {
                title: s.title,
                text: s.text,
                mode: match s.mode {
                    Mode::Replace => SectionMode::Replace,
                    Mode::Append => SectionMode::Append,
                },
            }),
        };
        ok(card_summary(&tryt!(ws.update_card(&a.id, None, &patch))))
    }

    #[tool(description = "Move um cartão para uma coluna (e posição) do mesmo quadro.")]
    async fn pauta_move_card(&self, Parameters(a): Parameters<MoveCard>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_move_card"));
        ok(card_summary(&tryt!(ws.move_card(&a.id, &a.column, a.position))))
    }

    #[tool(
        description = "Agenda a publicação de um cartão na rede dele, num horário dado ou no próximo horário livre das faixas de postagem, e move o cartão para Agendado. Não publica nada: o Pauta só lembra o autor."
    )]
    async fn pauta_schedule_post(&self, Parameters(a): Parameters<SchedulePost>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_schedule_post"));
        let card = tryt!(ws.schedule_post(&a.id, a.at.as_deref()));
        let mut v = card_summary(&card);
        // Scheduling is allowed with gaps (the author decides), but Claude should say what is missing.
        let index = tryt!(ws.load());
        let problems = index.readiness.get(&card.id).cloned().unwrap_or_default();
        v["ready_to_post"] = json!(problems.is_empty());
        if !problems.is_empty() {
            v["problems"] = json!(problems.iter().map(problem_json).collect::<Vec<_>>());
        }
        ok(v)
    }

    #[tool(
        description = "Pede a arte de um post ao Claude Design: grava no cartão o pedido (estado, formato) e escreve a direção visual na seção ## Arte. NÃO gera nem busca imagem: o Pauta não tem API do Claude Design. Depois, o autor usa 'Copiar pedido' no app, cola no Claude Design e exporta os arquivos para a pasta de anexos do cartão (veja pauta_list_art_requests)."
    )]
    async fn pauta_request_art(&self, Parameters(a): Parameters<RequestArt>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_request_art"));
        let card = tryt!(ws.request_art(&a.id, a.format.as_deref(), a.brief.as_deref()));
        let index = tryt!(ws.load());
        ok(art_json(&ws, &index, index.card(&card.id).unwrap_or(&card)))
    }

    #[tool(
        description = "Lista os cartões com arte pedida e ainda não pronta: formato e tamanho, direção visual, texto do post, a pasta de anexos onde os arquivos devem ficar e os arquivos já presentes. Use para ver o que está esperando arte.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_list_art_requests(&self, _: Parameters<Empty>) -> ToolResult {
        let (ws, index) = tryt!(self.index());
        let requests: Vec<Value> = index
            .cards
            .iter()
            .filter(|c| {
                c.art
                    .as_ref()
                    .is_some_and(|a| a.status == pauta_core::model::ArtStatus::Requested)
            })
            .map(|c| art_json(&ws, &index, c))
            .collect();
        ok(json!({ "count": requests.len(), "requests": requests }))
    }

    #[tool(
        description = "Marca a arte de um cartão como pronta. Exige pelo menos um arquivo na pasta de anexos do cartão; não move nem cria arquivos."
    )]
    async fn pauta_mark_art_ready(&self, Parameters(a): Parameters<IdArg>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_mark_art_ready"));
        let card = tryt!(ws.set_art_status(&a.id, "ready"));
        let index = tryt!(ws.load());
        ok(art_json(&ws, &index, index.card(&card.id).unwrap_or(&card)))
    }

    #[tool(
        description = "Lista os próximos horários livres de postagem de uma rede, segundo as faixas configuradas e o que já está agendado.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_find_slots(&self, Parameters(a): Parameters<FindSlots>) -> ToolResult {
        let (ws, index) = tryt!(self.index());
        let tz = index.config.tz.clone();
        let after = match &a.after {
            Some(s) => tryt!(pauta_core::time::parse_instant(s, &tz)),
            None => ws.now(),
        };
        let slots = tryt!(schedule::find_slots(
            &index,
            &a.network,
            after,
            a.count.unwrap_or(5),
            None
        ));
        let slots: Vec<String> = slots.into_iter().map(|t| when(&When::instant(t, &tz))).collect();
        ok(json!({ "network": a.network, "slots": slots }))
    }

    #[tool(
        description = "Lista tudo o que tem data num intervalo: publicações, prazos de cartões e eventos do calendário.",
        annotations(read_only_hint = true)
    )]
    async fn pauta_list_agenda(&self, Parameters(a): Parameters<Range>) -> ToolResult {
        let (ws, index) = tryt!(self.index());
        ok(tryt!(agenda_json(&ws, &index, a.from.as_deref(), a.to.as_deref())))
    }

    #[tool(
        description = "Cria um evento de calendário avulso: prazo (deadline), evento, marco (milestone) ou lançamento (launch), com lembretes."
    )]
    async fn pauta_create_event(&self, Parameters(a): Parameters<CreateEvent>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_create_event"));
        let ev = tryt!(ws.create_event(&NewEvent {
            title: a.title,
            kind: a.kind.or(Some("deadline".into())),
            date: a.date,
            end: a.end,
            reminders: a.reminders,
            board: a.board,
            tags: a.tags,
            body: a.notes,
        }));
        ok(json!({ "id": ev.id, "title": ev.title, "kind": ev.kind, "when": when(&ev.when), "file": ev.path }))
    }

    #[tool(description = "Registra que o autor publicou o post (com o link), e move o cartão para Publicado.")]
    async fn pauta_mark_published(&self, Parameters(a): Parameters<MarkPublished>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_mark_published"));
        ok(card_summary(&tryt!(ws.mark_published(
            &a.id,
            a.url.as_deref(),
            a.at.as_deref()
        ))))
    }

    #[tool(
        description = "Registra os resultados de um post já publicado (impressões, reações, comentários, compartilhamentos, salvamentos, cliques), copiados das estatísticas da própria rede. Substitui o bloco inteiro; não envia nada à rede."
    )]
    async fn pauta_record_metrics(&self, Parameters(a): Parameters<RecordMetrics>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_record_metrics"));
        let patch = CardPatch {
            metrics: Some(Some(pauta_core::store::MetricsInput {
                impressions: a.impressions,
                reactions: a.reactions,
                comments: a.comments,
                shares: a.shares,
                saves: a.saves,
                clicks: a.clicks,
            })),
            ..Default::default()
        };
        let card = tryt!(ws.update_card(&a.id, None, &patch));
        ok(json!({ "id": card.id, "title": card.title, "metrics": card.metrics }))
    }

    #[tool(
        description = "Arquiva um cartão (move para archive/, recuperável pelo app). Não há exclusão definitiva.",
        annotations(destructive_hint = false)
    )]
    async fn pauta_archive_card(&self, Parameters(a): Parameters<IdArg>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_archive_card"));
        let path = tryt!(ws.archive_card(&a.id));
        ok(json!({ "archived": a.id, "file": path }))
    }

    #[tool(
        description = "Cria a versão de um cartão para outra rede (ex.: a legenda do Instagram de um post do LinkedIn), copiando título, etiquetas e corpo, e liga os dois como irmãos."
    )]
    async fn pauta_duplicate_for_network(&self, Parameters(a): Parameters<Duplicate>) -> ToolResult {
        let ws = tryt!(self.ws("pauta_duplicate_for_network"));
        ok(card_summary(&tryt!(ws.duplicate_for_network(&a.id, &a.network))))
    }
}

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for Pauta {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new("pauta", env!("CARGO_PKG_VERSION")))
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(Self::resources()))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(Self::resource_templates()))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        Ok(self.read(&request.uri)?.into())
    }
}
