//! MCP prompts (section 11.6): ready-made starting points, with the relevant Pauta
//! data already inlined so Claude doesn't need a round of tool calls first.
//! They only *ask* Claude to act; every write still goes through a tool and Claude
//! Code / Desktop's own approval.

use pauta_core::model::BoardKind;
use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{GetPromptResult, PromptMessage, Role};
use rmcp::{prompt, prompt_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::server::{Pauta, rhythm_json, today_json};

#[derive(Deserialize, JsonSchema)]
pub struct DraftFromMilestone {
    /// Id of the project card (or milestone) to write about, from pauta_list_cards.
    pub card_id: Option<String>,
    /// Anything else worth saying: context, audience, an angle.
    pub note: Option<String>,
    /// `linkedin` or `instagram`. Default: linkedin.
    pub network: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ArtBrief {
    /// Id of the content card that needs artwork.
    pub card_id: String,
    /// Format id (e.g. `carousel`). Default: the card's current format or the network's first.
    pub format: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct PlanWeek {
    /// Only this network. Default: all of them.
    pub network: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SuggestTopics {
    /// Subject or angle to look for (e.g. "Rust", "divulgação científica"). Default: what the recent posts and tags are about.
    pub focus: Option<String>,
    /// Only this network. Default: all of them.
    pub network: Option<String>,
}

fn user(text: String) -> GetPromptResult {
    GetPromptResult::new(vec![PromptMessage::new_text(Role::User, text)])
}

fn pretty(v: &serde_json::Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default()
}

fn internal(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

#[prompt_router(vis = "pub(crate)")]
impl Pauta {
    #[prompt(
        name = "draft_post_from_milestone",
        description = "Rascunhar um post (LinkedIn ou Instagram) a partir de um marco ou cartão de projeto, com o texto na seção da rede e as notas separadas."
    )]
    async fn draft_post_from_milestone(
        &self,
        Parameters(a): Parameters<DraftFromMilestone>,
    ) -> Result<GetPromptResult, ErrorData> {
        let (_, index) = self.index().map_err(internal)?;
        let network = a.network.unwrap_or_else(|| "linkedin".into());
        let net = index
            .config
            .network(&network)
            .ok_or_else(|| ErrorData::invalid_params(format!("rede '{network}' não configurada"), None))?;
        let content_board = index.boards.iter().find(|b| b.kind == BoardKind::Content);
        let draft_col = content_board
            .and_then(|b| b.columns.iter().find(|c| c.id == "draft").or(b.columns.get(1)))
            .map_or("draft", |c| c.id.as_str());

        let source = match &a.card_id {
            Some(id) => {
                let c = index
                    .card(id)
                    .ok_or_else(|| ErrorData::invalid_params(format!("cartão '{id}' não encontrado"), None))?;
                let board = index.board(&c.board).map_or(c.board.as_str(), |b| b.name.as_str());
                format!(
                    "Cartão do projeto \"{board}\" (id {id}):\n- título: {}\n- coluna: {}\n- etiquetas: {}\n- prazo: {}\n- lista de verificação: {}/{}\n\nCorpo do cartão:\n{}",
                    c.title,
                    c.column,
                    c.tags.join(", "),
                    c.due.as_ref().map_or("—".to_string(), |d| d.date.to_string()),
                    c.checklist.done,
                    c.checklist.total,
                    c.body.trim(),
                )
            }
            None => "Nenhum cartão foi indicado: parta do que eu disser abaixo.".to_string(),
        };
        let limit = net
            .char_limit
            .map_or(String::new(), |l| format!(" Limite de caracteres: {l}."));
        let text = format!(
            "Quero um rascunho de post para o {label} sobre um marco do meu trabalho.\n\n{source}\n\n{note}\
Como escrever: sou desenvolvedor e pesquisador em TI e gosto de divulgação científica. Escreva em primeira pessoa, em português do Brasil, com clareza e sem exagero de entusiasmo. Comece pelo que o leitor ganha em saber isso, explique o essencial sem jargão desnecessário e feche com um convite natural a conversar.{limit}\n\n\
Como proceder:\n\
1. Mostre o rascunho aqui e espere meu retorno; ajuste até eu aprovar.\n\
2. Aprovado, crie o cartão com pauta_create_card: board \"{board}\", coluna \"{draft_col}\", network \"{network}\", corpo com a seção `## {label}` (só o texto do post) e a seção `## Notas` (fontes, ideias descartadas){related}.\n\
3. Não agende nem marque como publicado sem eu pedir. Se eu pedir um horário, use pauta_find_slots e pauta_schedule_post.",
            label = net.label,
            note = a.note.map_or(String::new(), |n| format!("Observações minhas: {n}\n\n")),
            board = content_board.map_or("conteudo", |b| b.id.as_str()),
            related = a.card_id.as_ref().map_or(String::new(), |_| {
                ", e related_board com o quadro do projeto de origem".to_string()
            }),
        );
        Ok(user(text))
    }

    #[prompt(
        name = "art_brief",
        description = "Escrever a direção visual da arte de um post e registrar o pedido (pauta_request_art) para o Claude Design."
    )]
    async fn art_brief(&self, Parameters(a): Parameters<ArtBrief>) -> Result<GetPromptResult, ErrorData> {
        let (ws, index) = self.index().map_err(internal)?;
        let card = index
            .card(&a.card_id)
            .ok_or_else(|| ErrorData::invalid_params(format!("cartão '{}' não encontrado", a.card_id), None))?;
        let net = card
            .network
            .as_deref()
            .and_then(|n| index.config.network(n))
            .ok_or_else(|| ErrorData::invalid_params("o cartão não tem rede definida", None))?;
        let formats: Vec<String> = net
            .media
            .formats
            .iter()
            .map(|f| {
                format!(
                    "- {}: {} ({} px){}",
                    f.id,
                    f.label,
                    f.size,
                    f.note.as_ref().map_or(String::new(), |n| format!(", {n}"))
                )
            })
            .collect();
        let chosen = a
            .format
            .clone()
            .or_else(|| pauta_core::art::format_for(net, card).map(|f| f.id.clone()));
        let text = format!(
            "Vamos preparar a arte de um post do {label}.\n\nCartão (id {id}): {title}\n\nTexto do post:\n{post}\n\nDireção visual já escrita no cartão:\n{brief}\n\nFormatos do {label}:\n{formats}\nFormato sugerido: {chosen}\n\nIdentidade visual do autor:\n{brand}\n\n\
Como proceder:\n\
1. Proponha uma direção visual curta (conceito, composição, paleta e tipografia coerentes com a identidade, quanto texto cabe na imagem, o que evitar). Seja concreto e legível em tela de celular, com contraste alto. Mostre aqui e espere meu retorno.\n\
2. Aprovada, chame pauta_request_art com id \"{id}\", o formato e a direção no campo brief.\n\
3. Lembre-me do próximo passo, que é manual: abrir o cartão no Pauta, clicar em \"Copiar pedido\", colar no Claude Design (não há API para isso), exportar os arquivos e anexá-los no cartão ou salvá-los em {dir}. Quando eu avisar que estão lá, use pauta_mark_art_ready.\n\
Não gere imagens nem diga que a arte foi criada: o Pauta só registra o pedido e acompanha o resultado.",
            label = net.label,
            id = card.id,
            title = card.title,
            post = pauta_core::art::text_for_posting(&index, card).trim(),
            brief = pauta_core::markdown::section_text(&card.body, "Arte").unwrap_or_else(|| "(nenhuma ainda)".into()),
            formats = formats.join("\n"),
            chosen = chosen.as_deref().unwrap_or("a definir"),
            brand = index
                .config
                .art
                .brand
                .as_deref()
                .unwrap_or("(sem notas de identidade cadastradas)"),
            dir = ws.assets_dir(card).display(),
        );
        Ok(user(text))
    }

    #[prompt(
        name = "weekly_review",
        description = "Revisão da semana: o que foi publicado, o que falta para as metas, o que está parado, e 2 ideias de post."
    )]
    async fn weekly_review(&self) -> Result<GetPromptResult, ErrorData> {
        let (ws, index) = self.index().map_err(internal)?;
        let text = format!(
            "Faça comigo uma revisão curta da semana no Pauta, sem cobrança: o objetivo é enxergar o ritmo e decidir o próximo passo.\n\n\
Hoje (pauta_today):\n{}\n\nRitmo (pauta_rhythm):\n{}\n\n\
Responda em até 10 linhas: (1) o que saiu e o que está agendado, (2) o que falta para as metas de cadência, (3) até três cartões parados que valem uma decisão (avançar, adiar ou arquivar), (4) duas ideias de post que aproveitem o que já tenho em andamento. Depois pergunte o que quero registrar; não altere nada antes disso.",
            pretty(&today_json(&ws, &index)),
            pretty(&rhythm_json(&ws, &index)),
        );
        Ok(user(text))
    }

    #[prompt(
        name = "plan_posting_week",
        description = "Propor em quais horários livres agendar os rascunhos prontos, respeitando as faixas e as metas de cadência."
    )]
    async fn plan_posting_week(&self, Parameters(a): Parameters<PlanWeek>) -> Result<GetPromptResult, ErrorData> {
        let (ws, index) = self.index().map_err(internal)?;
        let nets: Vec<_> = index
            .config
            .networks
            .iter()
            .filter(|n| a.network.as_ref().is_none_or(|w| *w == n.id))
            .collect();
        if nets.is_empty() {
            return Err(ErrorData::invalid_params("rede não configurada", None));
        }
        let mut sections = Vec::new();
        for n in nets {
            let slots = pauta_core::schedule::find_slots(&index, &n.id, ws.now(), 5, None).map_err(internal)?;
            let slots: Vec<String> = slots
                .into_iter()
                .map(|t| pauta_core::time::format_instant(t, &index.config.tz))
                .collect();
            // Cards that could go out: past the idea stage, of this network (or not yet assigned), without a date.
            let ready: Vec<_> = index
                .cards
                .iter()
                .filter(|c| c.network.as_deref().is_none_or(|x| x == n.id))
                .filter(|c| {
                    index.board(&c.board).is_some_and(|b| {
                        b.kind == BoardKind::Content
                            && !b.is_done_column(&c.column)
                            && b.column(&c.column)
                                .is_some_and(|col| col.role != Some(pauta_core::model::ColumnRole::Inbox))
                    })
                })
                .filter(|c| c.pending_publication_at().is_none())
                .map(|c| json!({"id": c.id, "title": c.title, "column": c.column, "network": c.network}))
                .collect();
            let goal = pauta_core::rhythm::cadence(&index, ws.now())
                .into_iter()
                .find(|c| c.network == n.id)
                .map_or("sem meta definida".to_string(), |c| {
                    format!(
                        "meta {} no período; publicados {}, agendados {}",
                        c.goal, c.done, c.scheduled
                    )
                });
            sections.push(format!(
                "## {}\n- cadência: {goal}\n- próximos horários livres: {}\n- candidatos: {}",
                n.label,
                slots.join(", "),
                serde_json::to_string(&ready).unwrap_or_default()
            ));
        }
        let text = format!(
            "Proponha um plano de publicação para os próximos dias com o que já está pronto no Pauta.\n\n{}\n\n\
Casando candidatos e horários livres, monte uma tabela (cartão, rede, horário sugerido, motivo). Priorize o que está mais adiantado e cubra primeiro as metas de cadência que faltam. Cartões sem rede precisam de uma rede antes de agendar. Mostre o plano e peça minha confirmação; só então use pauta_update_card (se faltar rede) e pauta_schedule_post com o horário escolhido. Não marque nada como publicado.",
            sections.join("\n\n"),
        );
        Ok(user(text))
    }

    #[prompt(
        name = "suggest_topics",
        description = "Pesquisar temas em alta no assunto de que costumo falar e propor ideias de post, sem repetir o que já publiquei."
    )]
    async fn suggest_topics(&self, Parameters(a): Parameters<SuggestTopics>) -> Result<GetPromptResult, ErrorData> {
        let (_, index) = self.index().map_err(internal)?;
        // Newest first; archived posts still count, so a topic isn't suggested twice.
        let mut recent: Vec<_> = index
            .published
            .iter()
            .filter(|p| a.network.as_ref().is_none_or(|n| *n == p.network))
            .collect();
        recent.sort_by_key(|x| std::cmp::Reverse(x.date));
        let recent: Vec<String> = recent
            .iter()
            .take(15)
            .map(|p| format!("- {} ({}, {})", p.title, p.network, p.date))
            .collect();
        // Tags on content cards say what the author writes about.
        let mut tags: Vec<(&str, usize)> = Vec::new();
        for c in &index.cards {
            for t in &c.tags {
                match tags.iter_mut().find(|(k, _)| *k == t.as_str()) {
                    Some((_, n)) => *n += 1,
                    None => tags.push((t.as_str(), 1)),
                }
            }
        }
        tags.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(y.0)));
        let tags: Vec<&str> = tags.iter().take(12).map(|(t, _)| *t).collect();
        let open: Vec<String> = index
            .cards
            .iter()
            .filter(|c| {
                index
                    .board(&c.board)
                    .is_some_and(|b| b.kind == BoardKind::Content && !b.is_done_column(&c.column))
            })
            .take(25)
            .map(|c| format!("- {}", c.title))
            .collect();
        let focus = a
            .focus
            .as_deref()
            .unwrap_or("o que meus posts e etiquetas abaixo indicam");
        let text = format!(
            "Ajude-me a achar o que postar sobre {focus}. Use a busca na web, se você tiver essa ferramenta; se não tiver, diga isso e proponha só a partir do que está abaixo, sem fingir que viu tendências.\n\n\
Já publiquei (não repita o tema):\n{recent}\n\n\
Já tenho como ideia ou em andamento:\n{open}\n\n\
Etiquetas mais usadas: {tags}\n\n\
Traga até cinco ideias. Para cada uma: título curto, por que agora (com a fonte e a data de onde veio), o ângulo que eu, desenvolvedor e pesquisador em TI com gosto por divulgação científica, posso dar, e em qual rede combina mais. Separe o que é fato encontrado do que é palpite seu. Mostre a lista e pergunte quais eu quero; só então use pauta_add_idea (uma chamada por ideia escolhida, com a fonte em notes). Não publique nem agende nada.",
            recent = if recent.is_empty() {
                "(nada ainda)".into()
            } else {
                recent.join("\n")
            },
            open = if open.is_empty() {
                "(nada)".into()
            } else {
                open.join("\n")
            },
            tags = if tags.is_empty() {
                "(nenhuma)".into()
            } else {
                tags.join(", ")
            },
        );
        Ok(user(text))
    }
}
