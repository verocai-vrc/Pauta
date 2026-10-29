//! Starts the real `pauta-mcp` binary on a temporary data folder, calls every
//! tool over the MCP protocol and checks the files it leaves behind (section 15.2).

use std::path::Path;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use serde_json::{Value, json};
use tokio::process::Command;

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let dst = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), dst).unwrap();
        }
    }
}

fn text(r: &CallToolResult) -> String {
    r.content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("")
}

fn files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            if e.path().is_dir() {
                stack.push(e.path());
            } else {
                out.push(e.path());
            }
        }
    }
    out
}

#[tokio::test]
async fn every_tool_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample-data"),
        dir.path(),
    );
    let root = dir.path().to_path_buf();
    let transport = TokioChildProcess::new(Command::new(env!("CARGO_BIN_EXE_pauta-mcp")).configure(|cmd| {
        cmd.arg("--root").arg(&root);
    }))
    .unwrap();
    let client = ().serve(transport).await.expect("client connects");

    let tools = client.list_all_tools().await.unwrap();
    let mut names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "pauta_add_idea",
            "pauta_archive_card",
            "pauta_create_card",
            "pauta_create_event",
            "pauta_duplicate_for_network",
            "pauta_find_slots",
            "pauta_get_card",
            "pauta_list_agenda",
            "pauta_list_art_requests",
            "pauta_list_boards",
            "pauta_list_cards",
            "pauta_mark_art_ready",
            "pauta_mark_published",
            "pauta_move_card",
            "pauta_record_metrics",
            "pauta_request_art",
            "pauta_rhythm",
            "pauta_schedule_post",
            "pauta_today",
            "pauta_update_card",
        ]
    );
    // No tool deletes or posts (section 11.4).
    assert!(names.iter().all(|n| !n.contains("delete") && !n.contains("publish_to")));

    let call = |name: &'static str, args: Value| {
        let client = &client;
        async move {
            let params = CallToolRequestParams::new(name).with_arguments(args.as_object().unwrap().clone());
            let r = client.call_tool(params).await.unwrap();
            let t = text(&r);
            (r.is_error.unwrap_or(false), t)
        }
    };
    let json_of = |t: &str| serde_json::from_str::<Value>(t).unwrap();

    let (err, t) = call("pauta_list_boards", json!({})).await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["boards"][0]["id"], "conteudo");

    let (err, t) = call(
        "pauta_add_idea",
        json!({"title": "Ideia via Claude", "notes": "contexto da conversa"}),
    )
    .await;
    assert!(!err, "{t}");
    let idea = json_of(&t);
    assert_eq!(idea["column"], "idea");
    let id = idea["id"].as_str().unwrap().to_string();

    let (err, t) = call(
        "pauta_update_card",
        json!({
            "id": id, "network": "linkedin", "column": "draft",
            "section": {"title": "LinkedIn", "text": "Rascunho escrito pelo Claude", "mode": "replace"}
        }),
    )
    .await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["column"], "draft");

    let (_, t) = call("pauta_get_card", json!({"id": id})).await;
    let card = json_of(&t);
    assert!(
        card["body"]
            .as_str()
            .unwrap()
            .contains("## LinkedIn\n\nRascunho escrito pelo Claude")
    );
    assert!(
        card["body"]
            .as_str()
            .unwrap()
            .contains("## Notas\n\ncontexto da conversa")
    );

    let (err, t) = call(
        "pauta_find_slots",
        json!({"network": "linkedin", "count": 2, "after": "2026-10-28T21:00"}),
    )
    .await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["slots"], json!(["2026-10-29 09:00", "2026-11-05 09:00"]));

    let (err, t) = call("pauta_schedule_post", json!({"id": id, "at": "2026-11-05T09:00"})).await;
    assert!(!err, "{t}");
    let s = json_of(&t);
    assert_eq!(s["column"], "scheduled");
    assert_eq!(s["publication"]["status"], "scheduled");

    let (err, t) = call("pauta_duplicate_for_network", json!({"id": id, "network": "instagram"})).await;
    assert!(!err, "{t}");
    let ig = json_of(&t)["id"].as_str().unwrap().to_string();

    let (err, t) = call(
        "pauta_list_cards",
        json!({"network": "instagram", "text": "via claude"}),
    )
    .await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["count"], 1);

    // ---- artwork: request, wait, files arrive, ready
    let (err, t) = call(
        "pauta_request_art",
        json!({"id": ig, "format": "carousel", "brief": "Cinco slides, um conceito por slide"}),
    )
    .await;
    assert!(!err, "{t}");
    let art = json_of(&t);
    assert_eq!(
        (art["status"].as_str(), art["format"]["size"].as_str()),
        (Some("requested"), Some("1080x1350"))
    );
    assert!(art["brief"].as_str().unwrap().contains("Cinco slides"));
    assert!(
        art["request_text"]
            .as_str()
            .unwrap()
            .contains("Formato: Carrossel (4:5)")
    );
    let assets_dir = std::path::PathBuf::from(art["assets_dir"].as_str().unwrap());
    assert!(assets_dir.starts_with(dir.path()), "files go inside the data folder");
    let (err, t) = call("pauta_request_art", json!({"id": ig, "format": "hologram"})).await;
    assert!(err && t.contains("story"), "{t}");
    let (_, t) = call("pauta_list_art_requests", json!({})).await;
    assert_eq!(json_of(&t)["count"], 1);
    let (err, t) = call("pauta_mark_art_ready", json!({"id": ig})).await;
    assert!(err, "no files yet: {t}");
    std::fs::create_dir_all(&assets_dir).unwrap();
    std::fs::write(assets_dir.join("01.png"), b"png").unwrap();
    let (err, t) = call("pauta_mark_art_ready", json!({"id": ig})).await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["status"], "ready");
    assert_eq!(json_of(&t)["files"], json!(["01.png"]));
    let (_, t) = call("pauta_list_art_requests", json!({})).await;
    assert_eq!(json_of(&t)["count"], 0, "ready requests are no longer waiting");
    let (err, t) = call("pauta_schedule_post", json!({"id": ig, "at": "2026-11-07T19:00"})).await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["ready_to_post"], true, "{t}");

    let (err, t) = call(
        "pauta_create_card",
        json!({"board": "dungeon-meister", "title": "Escrever README", "due": "2026-11-10"}),
    )
    .await;
    assert!(!err, "{t}");
    let task = json_of(&t)["id"].as_str().unwrap().to_string();
    let (err, t) = call("pauta_move_card", json!({"id": task, "column": "done", "position": 0})).await;
    assert!(!err, "{t}");

    let (err, t) = call("pauta_create_event", json!({
        "title": "Entrega do capítulo 4 do TCC2", "date": "2026-12-12", "board": "tcc2", "reminders": ["-7d", "09:00"]
    }))
    .await;
    assert!(!err, "{t}");

    let (err, t) = call("pauta_list_agenda", json!({"from": "2026-12-01", "to": "2026-12-31"})).await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["items"].as_array().unwrap().len(), 2);

    let (err, t) = call(
        "pauta_mark_published",
        json!({"id": id, "url": "https://www.linkedin.com/posts/x"}),
    )
    .await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["column"], "published");

    let (err, t) = call(
        "pauta_record_metrics",
        json!({"id": id, "impressions": 900, "reactions": 25, "comments": 3}),
    )
    .await;
    assert!(!err, "{t}");
    assert_eq!(json_of(&t)["metrics"]["reactions"], 25);
    let (err, t) = call("pauta_record_metrics", json!({"id": task, "reactions": 1})).await;
    assert!(err, "a project task has no published post: {t}");

    let (err, t) = call("pauta_today", json!({})).await;
    assert!(!err, "{t}");
    assert!(json_of(&t)["date"].is_string());
    assert_eq!(json_of(&t)["cadence"][0]["network"], "linkedin");

    let (err, t) = call("pauta_rhythm", json!({})).await;
    assert!(!err, "{t}");
    let rhythm = json_of(&t);
    assert_eq!(rhythm["weeks"].as_array().unwrap().len(), 12);
    assert_eq!(rhythm["cadence"][0]["goal"], "2 por semana");

    let (err, t) = call("pauta_archive_card", json!({"id": ig})).await;
    assert!(!err, "{t}");

    // Domain errors come back as readable tool errors, not protocol failures.
    let (err, t) = call("pauta_move_card", json!({"id": task, "column": "nao-existe"})).await;
    assert!(err);
    assert!(t.contains("nao-existe"), "{t}");

    client.cancel().await.unwrap();

    // Archived, not deleted; every write was logged with the tool name.
    assert_eq!(files_under(&dir.path().join("archive/cards/conteudo")).len(), 1);
    let log = std::fs::read_to_string(dir.path().join(".pauta/log.jsonl")).unwrap();
    for tool in [
        "pauta_add_idea",
        "pauta_update_card",
        "pauta_schedule_post",
        "pauta_create_event",
        "pauta_archive_card",
    ] {
        assert!(
            log.contains(&format!("\"actor\":\"mcp:{tool}\"")),
            "{tool} missing in log:\n{log}"
        );
    }
    assert!(!dir.path().join(".trash").exists() || files_under(&dir.path().join(".trash")).is_empty());
}

#[tokio::test]
async fn resources_and_prompts_over_stdio() {
    use rmcp::model::{GetPromptRequestParams, ReadResourceRequestParams, ResourceContents};

    let dir = tempfile::tempdir().unwrap();
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample-data"),
        dir.path(),
    );
    let root = dir.path().to_path_buf();
    let transport = TokioChildProcess::new(Command::new(env!("CARGO_BIN_EXE_pauta-mcp")).configure(|cmd| {
        cmd.arg("--root").arg(&root);
    }))
    .unwrap();
    let client = ().serve(transport).await.expect("client connects");

    // ---- resources
    let uris: Vec<String> = client
        .list_all_resources()
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.uri)
        .collect();
    assert_eq!(
        uris,
        ["pauta://today", "pauta://boards", "pauta://rhythm", "pauta://agenda"]
    );
    let templates: Vec<String> = client
        .list_all_resource_templates()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.uri_template)
        .collect();
    assert_eq!(templates, ["pauta://card/{id}", "pauta://board/{id}"]);

    let read = |uri: &'static str| {
        let client = &client;
        async move {
            let r = client.read_resource(ReadResourceRequestParams::new(uri)).await;
            r.map(|r| match r.contents.into_iter().next().unwrap() {
                ResourceContents::TextResourceContents { text, mime_type, .. } => (text, mime_type.unwrap_or_default()),
                other => panic!("expected text, got {other:?}"),
            })
        }
    };
    let (text, mime) = read("pauta://boards").await.unwrap();
    assert_eq!(mime, "application/json");
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap()["boards"][0]["id"],
        "conteudo"
    );
    let (text, _) = read("pauta://today").await.unwrap();
    assert!(serde_json::from_str::<Value>(&text).unwrap()["cadence"].is_array());
    let (text, _) = read("pauta://rhythm").await.unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap()["weeks"]
            .as_array()
            .unwrap()
            .len(),
        12
    );
    let (text, _) = read("pauta://agenda").await.unwrap();
    assert!(serde_json::from_str::<Value>(&text).unwrap()["items"].is_array());

    // A card resource is the file itself, byte for byte.
    let (text, mime) = read("pauta://card/01JBX3K9Q2AB").await.unwrap();
    assert_eq!(mime, "text/markdown");
    let on_disk = std::fs::read_to_string(
        dir.path()
            .join("boards/conteudo/cards/por-que-rust-nao-tem-coletor-de-lixo--k9q2ab.md"),
    )
    .unwrap();
    assert_eq!(text, on_disk);
    let (text, _) = read("pauta://board/dungeon-meister").await.unwrap();
    let board: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        board["columns"][2]["cards"][0]["title"],
        "Resolução em etapas com janelas de reação"
    );
    // Unknown things are errors, not empty results.
    assert!(read("pauta://card/NAO-EXISTE").await.is_err());
    assert!(read("pauta://nada").await.is_err());
    assert!(read("file:///etc/passwd").await.is_err());

    // ---- prompts
    let mut names: Vec<String> = client
        .list_all_prompts()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "art_brief",
            "draft_post_from_milestone",
            "plan_posting_week",
            "suggest_topics",
            "weekly_review"
        ]
    );

    let prompt = |name: &'static str, args: Value| {
        let client = &client;
        async move {
            let mut p = GetPromptRequestParams::new(name);
            if let Some(m) = args.as_object() {
                p = p.with_arguments(m.clone());
            }
            client.get_prompt(p).await.map(|r| {
                r.messages
                    .into_iter()
                    .filter_map(|m| m.content.as_text().map(|t| t.text.clone()))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
    };
    let text = prompt(
        "draft_post_from_milestone",
        json!({"card_id": "01JBZ8M4T1XY", "network": "linkedin", "note": "foco em testes"}),
    )
    .await
    .unwrap();
    for needle in [
        "Resolução em etapas com janelas de reação",
        "Testes de snapshot",
        "foco em testes",
        "pauta_create_card",
        "## LinkedIn",
        "3000",
        "related_board",
    ] {
        assert!(text.contains(needle), "{needle} missing in:\n{text}");
    }
    assert!(
        prompt("draft_post_from_milestone", json!({"card_id": "x-nao-existe"}))
            .await
            .is_err()
    );
    assert!(
        prompt("draft_post_from_milestone", json!({"network": "mastodon"}))
            .await
            .is_err()
    );

    let text = prompt("art_brief", json!({"card_id": "01JBX3K9Q3CD", "format": "carousel"}))
        .await
        .unwrap();
    for needle in [
        "post do Instagram",
        "01JBX3K9Q3CD",
        "carousel: Carrossel (4:5) (1080x1350 px)",
        "story:",
        "pauta_request_art",
        "Copiar pedido",
        "não há API",
        "assets/01JBX3K9Q3CD",
    ] {
        assert!(text.contains(needle), "{needle} missing in:\n{text}");
    }
    assert!(
        prompt("art_brief", json!({"card_id": "01JBZ8M4T1XY"})).await.is_err(),
        "a project card has no network"
    );

    let text = prompt("weekly_review", json!({})).await.unwrap();
    assert!(
        text.contains("\"cadence\"") && text.contains("não altere nada"),
        "{text}"
    );

    let text = prompt("plan_posting_week", json!({"network": "instagram"}))
        .await
        .unwrap();
    assert!(text.contains("## Instagram") && text.contains("2026-"), "{text}");
    assert!(
        text.contains("Por que Rust não tem coletor de lixo") || text.contains("candidatos"),
        "{text}"
    );
    assert!(!text.contains("## LinkedIn"), "filtered to one network");

    let text = prompt("suggest_topics", json!({"focus": "Rust"})).await.unwrap();
    assert!(text.contains("sobre Rust") && text.contains("pauta_add_idea"), "{text}");
    assert!(text.contains("Já publiquei"), "{text}");
    let text = prompt("suggest_topics", json!({})).await.unwrap();
    assert!(text.contains("etiquetas abaixo"), "{text}");

    client.cancel().await.unwrap();
    // Reading resources and getting prompts never writes.
    assert!(!dir.path().join(".pauta/log.jsonl").exists());
}
