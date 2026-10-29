//! Integration tests over a real temporary data folder (section 15.1).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use jiff::Timestamp;
use pauta_core::clock::FixedClock;
use pauta_core::frontmatter::Doc;
use pauta_core::reminders::ReminderState;
use pauta_core::schedule;
use pauta_core::store::{BoardPatch, CardPatch, NewCard, NewEvent, PublicationPatch};
use pauta_core::time::{format_instant, parse_instant};
use pauta_core::{Error, Workspace};

const RUST_LI: &str = "01JBX3K9Q2AB";
const RUST_IG: &str = "01JBX3K9Q3CD";
const CRLF: &str = "01JBX9CRLF01";
const TASK: &str = "01JBZ8M4T1XY";

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let dst = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            fs::copy(e.path(), dst).unwrap();
        }
    }
}

fn ts(s: &str) -> Timestamp {
    parse_instant(s, &pauta_core::time::zone("America/Recife").unwrap()).unwrap()
}

/// The sample folder copied to a temp dir, with the clock at Wed 2026-10-28 21:00 (Recife).
fn sample() -> (tempfile::TempDir, Workspace, Arc<FixedClock>) {
    let dir = tempfile::tempdir().unwrap();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample-data");
    copy_dir(&src, dir.path());
    let clock = Arc::new(FixedClock::new(ts("2026-10-28T21:00")));
    let ws = Workspace::open(dir.path()).unwrap().with_clock(clock.clone());
    (dir, ws, clock)
}

fn all_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.file_name().unwrap() == ".pauta" {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push((p.clone(), fs::read(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn sample_folder_loads_and_round_trips_unchanged() {
    let (dir, ws, _) = sample();
    let before = all_files(dir.path());
    let index = ws.load().unwrap();
    assert_eq!(index.boards.len(), 2);
    assert_eq!(index.cards.len(), 4);
    assert_eq!(index.events.len(), 1);
    assert_eq!(index.invalid.len(), 1, "{:?}", index.invalid);
    assert!(index.invalid[0].path.ends_with("cabecalho-quebrado--broken.md"));

    for (path, bytes) in &before {
        if path.extension().is_some_and(|e| e == "md") {
            let text = String::from_utf8(bytes.clone()).unwrap();
            assert_eq!(Doc::parse(&text).render(), text, "{}", path.display());
        }
    }
    // Saving every card with no changes writes nothing.
    for c in &index.cards {
        ws.update_card(&c.id, Some(&c.hash), &CardPatch::default()).unwrap();
    }
    assert_eq!(all_files(dir.path()), before);
}

#[test]
fn editing_one_field_changes_only_that_field() {
    let (_dir, ws, _) = sample();
    let card = ws.card(RUST_LI).unwrap();
    let original = fs::read_to_string(&card.path).unwrap();
    ws.update_card(
        RUST_LI,
        Some(&card.hash),
        &CardPatch {
            tags: Some(vec!["rust".into(), "ciencia".into()]),
            ..Default::default()
        },
    )
    .unwrap();
    let edited = fs::read_to_string(&card.path).unwrap();
    let changed: Vec<(&str, &str)> = original.lines().zip(edited.lines()).filter(|(a, b)| a != b).collect();
    assert_eq!(
        changed,
        vec![
            ("tags: [rust, divulgacao]", "tags: [rust, ciencia]"),
            (
                "updated: 2026-10-28T21:02:00-03:00",
                "updated: 2026-10-28T21:00:00-03:00"
            ),
        ]
    );
    assert!(edited.contains("mood: empolgado               # campo desconhecido"));
    assert!(edited.contains("  board: dungeon-meister      # projeto relacionado, opcional"));

    // CRLF files stay CRLF.
    let crlf = ws.card(CRLF).unwrap();
    ws.update_card(
        CRLF,
        None,
        &CardPatch {
            title: Some("Novo título".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let text = fs::read_to_string(&crlf.path).unwrap();
    assert!(text.contains("title: \"Novo título\"\r\n"), "{text:?}");
    assert!(!text.replace("\r\n", "").contains('\n'), "{text:?}");
}

#[test]
fn concurrent_edit_is_refused_not_overwritten() {
    let (_dir, ws, _) = sample();
    let card = ws.card(TASK).unwrap();
    let external = fs::read_to_string(&card.path)
        .unwrap()
        .replace("Testes de snapshot", "Testes (editado no Vim)");
    fs::write(&card.path, &external).unwrap();
    let err = ws
        .update_card(
            TASK,
            Some(&card.hash),
            &CardPatch {
                title: Some("minha versão".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
    match err {
        Error::Conflict { disk_content, .. } => assert_eq!(disk_content, external),
        other => panic!("expected conflict, got {other:?}"),
    }
    assert_eq!(fs::read_to_string(&card.path).unwrap(), external);
}

#[test]
fn invalid_files_are_never_rewritten() {
    let (dir, ws, _) = sample();
    let broken = dir
        .path()
        .join("boards/dungeon-meister/cards/cabecalho-quebrado--broken.md");
    let before = fs::read(&broken).unwrap();
    ws.create_card(
        "dungeon-meister",
        &NewCard {
            title: "Outra tarefa".into(),
            column: Some("todo".into()),
            ..Default::default()
        },
    )
    .unwrap();
    ws.move_card(TASK, "todo", Some(0)).unwrap();
    assert_eq!(fs::read(&broken).unwrap(), before);
}

#[test]
fn moving_writes_only_the_moved_card() {
    let (dir, ws, _) = sample();
    let ids: Vec<String> = ["A", "B", "C"]
        .iter()
        .map(|t| {
            ws.create_card(
                "dungeon-meister",
                &NewCard {
                    title: (*t).into(),
                    column: Some("todo".into()),
                    ..Default::default()
                },
            )
            .unwrap()
            .id
        })
        .collect();
    let before = all_files(dir.path());
    ws.move_card(&ids[2], "todo", Some(0)).unwrap();
    let after = all_files(dir.path());
    let changed: Vec<_> = before.iter().zip(&after).filter(|(a, b)| a != b).collect();
    assert_eq!(changed.len(), 1);
    let order: Vec<String> = ws
        .load()
        .unwrap()
        .column_cards("dungeon-meister", "todo")
        .iter()
        .map(|c| c.title.clone())
        .collect();
    assert_eq!(order, ["C", "A", "B"]);
    ws.move_card(&ids[0], "doing", None).unwrap();
    assert_eq!(ws.card(&ids[0]).unwrap().column, "doing");
    assert!(ws.move_card(&ids[0], "nope", None).is_err());
}

#[test]
fn scheduling_uses_next_free_slot_per_network() {
    let (_dir, ws, _) = sample();
    let tz = ws.config().unwrap().tz;
    // Instagram: Wed/Sat 19:00. Now is Wed 21:00, so the next slot is Saturday.
    let ig = ws.schedule_post(RUST_IG, None).unwrap();
    let p = ig.publication.unwrap();
    assert_eq!(
        format_instant(p.at.unwrap().at.unwrap(), &tz),
        "2026-10-31T19:00:00-03:00"
    );
    assert_eq!(p.status.as_str(), "scheduled");
    assert_eq!(ig.column, "scheduled");

    // LinkedIn: Tue/Thu 09:00. Tuesday 11-03 is taken by the sample card.
    let new = ws
        .create_card(
            "conteudo",
            &NewCard {
                title: "Outro post".into(),
                network: Some("linkedin".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let index = ws.load().unwrap();
    let slots = schedule::find_slots(&index, "linkedin", ws.now(), 2, None).unwrap();
    let slots: Vec<String> = slots.into_iter().map(|t| format_instant(t, &tz)).collect();
    assert_eq!(slots, ["2026-10-29T09:00:00-03:00", "2026-11-05T09:00:00-03:00"]);
    let li = ws.schedule_post(&new.id, None).unwrap();
    assert_eq!(
        format_instant(li.publication.unwrap().at.unwrap().at.unwrap(), &tz),
        "2026-10-29T09:00:00-03:00"
    );
    // That slot is now used, so the next card gets the following one.
    let index = ws.load().unwrap();
    let next = schedule::find_slots(&index, "linkedin", ws.now(), 1, None).unwrap();
    assert_eq!(format_instant(next[0], &tz), "2026-11-05T09:00:00-03:00");
    // An explicit time wins.
    let li = ws.schedule_post(&new.id, Some("2026-11-06T12:30")).unwrap();
    assert_eq!(li.publication.unwrap().at.unwrap().time.unwrap().hour(), 12);
    // Cards without a network cannot be scheduled.
    assert!(ws.schedule_post(TASK, None).is_err());
}

#[test]
fn mark_published_records_link_and_moves_to_done() {
    let (_dir, ws, _) = sample();
    let c = ws
        .mark_published(RUST_LI, Some("https://linkedin.com/posts/1"), None)
        .unwrap();
    let p = c.publication.unwrap();
    assert_eq!(p.status.as_str(), "published");
    assert_eq!(p.url.as_deref(), Some("https://linkedin.com/posts/1"));
    assert_eq!(p.published_at, Some(ws.now()));
    assert_eq!(c.column, "published");
    let text = fs::read_to_string(&c.path).unwrap();
    assert!(text.contains("  at: 2026-11-03T09:00:00-03:00\n"), "{text}");
}

#[test]
fn duplicating_for_another_network_links_siblings() {
    let (_dir, ws, _) = sample();
    let copy = ws.duplicate_for_network(TASK, "instagram");
    assert!(copy.is_ok());
    let copy = copy.unwrap();
    let orig = ws.card(TASK).unwrap();
    assert_eq!(orig.links.siblings, vec![copy.id.clone()]);
    assert_eq!(copy.links.siblings, vec![TASK.to_string()]);
    assert_eq!(copy.network.as_deref(), Some("instagram"));
    assert_eq!(copy.body, orig.body);
}

#[test]
fn publication_patch_validates() {
    let (_dir, ws, _) = sample();
    let err = ws.update_card(
        TASK,
        None,
        &CardPatch {
            publication: Some(Some(PublicationPatch {
                at: Some(Some("2026-11-04T10:00".into())),
                ..Default::default()
            })),
            ..Default::default()
        },
    );
    assert!(err.is_err(), "no network → no publication");
    let c = ws
        .update_card(
            RUST_IG,
            None,
            &CardPatch {
                publication: Some(Some(PublicationPatch {
                    at: Some(Some("2026-11-04T10:00".into())),
                    status: Some("scheduled".into()),
                    url: None,
                })),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(c.publication.unwrap().at.unwrap().time.unwrap().hour(), 10);
    assert!(
        ws.update_card(
            RUST_IG,
            None,
            &CardPatch {
                publication: Some(Some(PublicationPatch {
                    status: Some("publicando".into()),
                    ..Default::default()
                })),
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn agenda_and_today() {
    let (_dir, ws, _) = sample();
    let index = ws.load().unwrap();
    let dec = schedule::agenda(&index, "2026-12-01".parse().unwrap(), "2026-12-10".parse().unwrap());
    assert_eq!(dec.len(), 1);
    assert_eq!(dec[0].title, "Entrega do capítulo 3 do TCC2");
    assert!(dec[0].when.all_day());

    let today = schedule::today(&index, ws.now());
    assert!(today.posts.is_empty());
    assert!(today.overdue.is_empty());
    assert_eq!(today.invalid_files, 1);
    // LinkedIn has a post within 7 days; Instagram does not.
    let empty: Vec<&str> = today.empty_queues.iter().map(|q| q.network.as_str()).collect();
    assert_eq!(empty, ["instagram"]);
    assert_eq!(today.empty_queues[0].candidates[0].id, RUST_IG);

    // A week later the LinkedIn post is overdue.
    let later = ts("2026-11-04T10:00");
    let t = schedule::today(&index, later);
    assert_eq!(t.overdue.len(), 1);
    assert_eq!(t.overdue[0].id, RUST_LI);

    // The clock shown on Today is wall time in the configured zone (America/Recife, UTC-3),
    // cut to the minute, and agrees with the date even just before midnight.
    let late = schedule::today(&index, "2026-11-04T02:59:50Z".parse().unwrap());
    assert_eq!(late.date.to_string(), "2026-11-03");
    assert_eq!(late.time.to_string(), "23:59:00");
}

#[test]
fn reminders_fresh_missed_snoozed_and_quiet() {
    let (dir, ws, _) = sample();
    let index = ws.load().unwrap();
    let mut state = ReminderState::default();

    let due = state.due(&index, ts("2026-11-03T08:00"));
    assert_eq!(due.fresh.len(), 1);
    assert_eq!(due.fresh[0].spec, "-1h");
    assert!(due.missed.is_empty());
    state.mark_delivered(due.fresh.iter().map(|r| r.key.clone()), ts("2026-11-03T08:00"));
    assert!(state.due(&index, ts("2026-11-03T08:05")).fresh.is_empty());

    // Snooze the on-time reminder by 15 minutes.
    let due = state.due(&index, ts("2026-11-03T09:00"));
    assert_eq!(due.fresh[0].spec, "0");
    let key = due.fresh[0].key.clone();
    state.mark_delivered([key.clone()], ts("2026-11-03T09:00"));
    state.snooze(&key, ts("2026-11-03T09:15"));
    assert!(state.due(&index, ts("2026-11-03T09:10")).fresh.is_empty());
    assert_eq!(state.due(&index, ts("2026-11-03T09:16")).fresh.len(), 1);

    // Computer was off all morning: both reminders come back as one missed item.
    let fresh_state = ReminderState::default();
    let due = fresh_state.due(&index, ts("2026-11-03T12:00"));
    assert!(due.fresh.is_empty());
    assert_eq!(due.missed.len(), 1);
    assert_eq!(due.missed_keys.len(), 2);
    let mut after = fresh_state.clone();
    after.mark_delivered(due.all_keys().collect::<Vec<_>>(), ts("2026-11-03T12:00"));
    assert!(after.due(&index, ts("2026-11-03T12:01")).missed.is_empty());

    // Deadline reminders: 7 days and 1 day before, and on the day, at 09:00.
    let r = schedule::reminders_between(&index, ts("2026-11-27T00:00"), ts("2026-12-06T00:00"));
    let times: Vec<String> = r
        .iter()
        .filter(|r| r.item.kind == schedule::AgendaKind::Event)
        .map(|r| format_instant(r.fire_at, &index.config.tz))
        .collect();
    assert_eq!(
        times,
        [
            "2026-11-28T09:00:00-03:00",
            "2026-12-04T09:00:00-03:00",
            "2026-12-05T09:00:00-03:00"
        ]
    );

    // A post at 23:30 is announced at the end of quiet hours.
    ws.create_card(
        "conteudo",
        &NewCard {
            title: "Post noturno".into(),
            network: Some("instagram".into()),
            publication_at: Some("2026-11-07T23:30".into()),
            reminders: Some(vec!["0".into()]),
            ..Default::default()
        },
    )
    .unwrap();
    let index = ws.load().unwrap();
    let r = schedule::reminders_between(&index, ts("2026-11-07T20:00"), ts("2026-11-08T12:00"));
    let night: Vec<_> = r.iter().filter(|r| r.item.title == "Post noturno").collect();
    assert_eq!(
        format_instant(night[0].fire_at, &index.config.tz),
        "2026-11-08T07:00:00-03:00"
    );

    state.save(dir.path(), ts("2026-11-03T09:16")).unwrap();
    assert!(ReminderState::load(dir.path()).delivered.contains_key(&key));
}

#[test]
fn events_create_update_archive_restore() {
    let (dir, ws, _) = sample();
    let ev = ws
        .create_event(&NewEvent {
            title: "Defesa do TCC2".into(),
            kind: Some("milestone".into()),
            date: "2026-12-15T14:00".into(),
            board: Some("tcc2".into()),
            ..Default::default()
        })
        .unwrap();
    let text = fs::read_to_string(&ev.path).unwrap();
    assert!(text.contains("start: 2026-12-15T14:00:00-03:00"), "{text}");
    let ev = ws
        .update_event(
            &ev.id,
            Some(&ev.hash),
            &pauta_core::store::EventPatch {
                date: Some("2026-12-16".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let text = fs::read_to_string(&ev.path).unwrap();
    assert!(text.contains("date: 2026-12-16") && !text.contains("start:"), "{text}");

    let archived = ws.archive_event(&ev.id).unwrap();
    assert!(archived.starts_with(dir.path().join("archive/calendar")));
    assert!(ws.event(&ev.id).is_err());
    ws.restore_archived(&archived).unwrap();
    assert!(ws.event(&ev.id).is_ok());
}

#[test]
fn trash_and_archive_are_recoverable() {
    let (dir, ws, _) = sample();
    let card = ws.card(TASK).unwrap();
    let original = fs::read(&card.path).unwrap();
    ws.trash_card(TASK).unwrap();
    assert!(ws.card(TASK).is_err());
    let trash = ws.list_trash().unwrap();
    assert_eq!(trash.len(), 1);
    assert_eq!(trash[0].title, card.title);
    ws.restore_trash(&trash[0].path).unwrap();
    assert_eq!(fs::read(&card.path).unwrap(), original);

    let archived = ws.archive_card(TASK).unwrap();
    assert!(archived.starts_with(dir.path().join("archive/cards/dungeon-meister")));
    let items = ws.list_archive().unwrap();
    assert_eq!(items.len(), 1);
    ws.restore_archived(&items[0].path).unwrap();
    assert_eq!(fs::read(&card.path).unwrap(), original);
}

#[test]
fn boards_create_edit_and_guard_columns() {
    let (_dir, ws, _) = sample();
    let b = ws
        .create_board("Abstract-Tex", pauta_core::model::BoardKind::Project, None)
        .unwrap();
    assert_eq!(b.id, "abstract-tex");
    assert_eq!(b.columns.len(), 5);
    assert!(ws.config().unwrap().board_order.contains(&"abstract-tex".to_string()));

    // Renaming a column keeps its id; cards are untouched.
    let dm = ws.load().unwrap().board("dungeon-meister").unwrap().clone();
    let mut cols = dm.columns.clone();
    cols[2].name = "Em andamento".into();
    let dm2 = ws
        .update_board(
            "dungeon-meister",
            Some(&dm.hash),
            &BoardPatch {
                columns: Some(cols.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(dm2.columns[2].name, "Em andamento");
    assert_eq!(dm2.columns[2].limit, Some(2));
    // Removing a column that still has cards is refused.
    cols.remove(2);
    assert!(
        ws.update_board(
            "dungeon-meister",
            None,
            &BoardPatch {
                columns: Some(cols),
                ..Default::default()
            }
        )
        .is_err()
    );
    ws.archive_board("abstract-tex").unwrap();
    assert!(!ws.config().unwrap().board_order.contains(&"abstract-tex".to_string()));
}

#[test]
fn init_seeds_agreed_boards() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::init(dir.path().join("Pauta"), &pauta_core::templates::default_seed()).unwrap();
    let index = ws.load().unwrap();
    let names: Vec<&str> = index.boards.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names, ["Conteúdo", "TCC2", "Dungeon Meister", "Abstract-Tex", "Pauta"]);
    assert!(index.invalid.is_empty());
    let cfg = &index.config;
    assert_eq!(cfg.networks.len(), 2);
    assert_eq!(cfg.network("linkedin").unwrap().slots[0].days, ["tue", "thu"]);
}

#[test]
fn newer_format_is_refused() {
    let (dir, _ws, _) = sample();
    let cfg = dir.path().join("pauta.yaml");
    let text = fs::read_to_string(&cfg)
        .unwrap()
        .replace("schema_version: 1", "schema_version: 99");
    fs::write(&cfg, text).unwrap();
    assert!(Workspace::open(dir.path()).is_err());
}

#[test]
fn config_patch_preserves_unknown_keys() {
    let (dir, ws, _) = sample();
    let mut nets = ws.config().unwrap().networks;
    nets[0].slots[0].days = vec!["mon".into(), "wed".into()];
    ws.update_config(&pauta_core::config::ConfigPatch {
        networks: Some(nets),
        ..Default::default()
    })
    .unwrap();
    let text = fs::read_to_string(dir.path().join("pauta.yaml")).unwrap();
    assert!(text.contains("days: [mon, wed]"), "{text}");
    assert!(text.contains("cadence_goal: { posts: 2, per: week }"), "{text}");
    assert!(
        text.contains("quiet_hours: { from: \"23:00\", to: \"07:00\" }"),
        "{text}"
    );
}

#[test]
fn write_log_records_actor() {
    let (dir, ws, _) = sample();
    ws.with_actor("mcp:pauta_move_card")
        .move_card(TASK, "review", None)
        .unwrap();
    let log = fs::read_to_string(dir.path().join(".pauta/log.jsonl")).unwrap();
    assert!(log.contains("\"actor\":\"mcp:pauta_move_card\""), "{log}");
    assert!(log.contains("\"op\":\"move_card\""), "{log}");
}

#[test]
fn sections_can_be_replaced_or_appended() {
    let (_dir, ws, _) = sample();
    use pauta_core::markdown::SectionMode;
    let c = ws
        .update_card(
            RUST_LI,
            None,
            &CardPatch {
                section: Some(pauta_core::store::SectionEdit {
                    title: "Notas".into(),
                    text: "Nova fonte".into(),
                    mode: SectionMode::Append,
                }),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        c.body
            .contains("Fontes, ideias soltas, prompt usado no Claude Design...\n\nNova fonte\n")
    );
    assert!(c.body.starts_with("\nOwnership e borrowing"));
}

// ---- regressions from the first code review --------------------------------

#[test]
fn mark_published_without_url_keeps_the_existing_link() {
    let (_dir, ws, _) = sample();
    ws.update_card(
        RUST_LI,
        None,
        &CardPatch {
            publication: Some(Some(PublicationPatch {
                url: Some(Some("https://linkedin.com/posts/already-here".into())),
                ..Default::default()
            })),
            ..Default::default()
        },
    )
    .unwrap();
    let c = ws.mark_published(RUST_LI, None, None).unwrap();
    let p = c.publication.unwrap();
    assert_eq!(p.status.as_str(), "published");
    assert_eq!(p.url.as_deref(), Some("https://linkedin.com/posts/already-here"));
    // A new link replaces it.
    let c = ws
        .mark_published(RUST_LI, Some("https://linkedin.com/posts/new"), None)
        .unwrap();
    assert_eq!(
        c.publication.unwrap().url.as_deref(),
        Some("https://linkedin.com/posts/new")
    );
}

#[test]
fn multibyte_ids_do_not_panic() {
    let (_dir, ws, _) = sample();
    // 9 bytes: the 6-from-the-end byte offset falls inside a 'ç'.
    for id in ["ççççx", "ççççççççç", "日本語のカードです", "é"] {
        assert!(matches!(ws.card(id), Err(Error::NotFound(_))), "{id}");
        assert!(ws.update_card(id, None, &CardPatch::default()).is_err());
        assert!(ws.archive_card(id).is_err());
    }
}

#[test]
fn bad_schema_versions_are_errors_not_panics() {
    for bad in ["0", "-3", "4294967297", "99999999999"] {
        let (dir, _ws, _) = sample();
        let cfg = dir.path().join("pauta.yaml");
        let text = fs::read_to_string(&cfg)
            .unwrap()
            .replace("schema_version: 1", &format!("schema_version: {bad}"));
        fs::write(&cfg, text).unwrap();
        let err = Workspace::open(dir.path()).unwrap_err();
        assert!(err.to_string().contains("schema_version"), "{bad}: {err}");
    }
}

#[test]
fn day_counts_are_range_checked() {
    use pauta_core::config::ConfigPatch;
    let (dir, ws, _) = sample();
    for days in [0, -1, 3651, i64::MAX] {
        let err = ws.update_config(&ConfigPatch {
            auto_archive_days: Some(Some(days)),
            ..Default::default()
        });
        assert!(err.is_err(), "auto_archive_days {days}");
        let err = ws.update_config(&ConfigPatch {
            empty_queue_days: Some(days),
            ..Default::default()
        });
        assert!(err.is_err(), "empty_queue_days {days}");
        let err = ws.update_config(&ConfigPatch {
            stale_after_days: Some(days),
            ..Default::default()
        });
        assert!(err.is_err(), "stale_after_days {days}");
    }
    // Nothing was written by the rejected patches.
    assert!(
        !fs::read_to_string(dir.path().join("pauta.yaml"))
            .unwrap()
            .contains("auto_after_days")
    );

    // Hand-edited values are rejected on load too, instead of archiving everything or overflowing.
    let cfg = dir.path().join("pauta.yaml");
    let original = fs::read_to_string(&cfg).unwrap();
    for (key, edited) in [
        (
            "auto_after_days",
            format!("{original}archive:\n  auto_after_days: -1\n"),
        ),
        (
            "empty_queue_days",
            original.replace("empty_queue_days: 7", "empty_queue_days: 999999999999999999"),
        ),
        (
            "stale_after_days",
            original.replace("stale_after_days: 14", "stale_after_days: 0"),
        ),
    ] {
        fs::write(&cfg, edited).unwrap();
        let err = ws.config().unwrap_err();
        assert!(err.to_string().contains(key), "{key}: {err}");
    }
    fs::write(&cfg, original).unwrap();
    let ok = ws.update_config(&ConfigPatch {
        auto_archive_days: Some(Some(30)),
        ..Default::default()
    });
    assert_eq!(ok.unwrap().auto_archive_days, Some(30));
}

#[test]
fn board_folder_name_is_the_id_even_if_the_yaml_disagrees() {
    let (dir, ws, _) = sample();
    fs::rename(dir.path().join("boards/conteudo"), dir.path().join("boards/content")).unwrap();
    let index = ws.load().unwrap();
    let b = index.boards.iter().find(|b| b.name == "Conteúdo").unwrap();
    assert_eq!(b.id, "content", "yaml still says `id: conteudo`");
    assert!(index.cards.iter().any(|c| c.id == RUST_LI && c.board == "content"));

    let card = ws
        .create_card(
            "content",
            &NewCard {
                title: "Depois de renomear".into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(card.path.starts_with(dir.path().join("boards/content/cards")));
    assert_eq!(
        ws.assets_dir(&card),
        dir.path().join("boards/content/assets").join(&card.id)
    );
    ws.update_board(
        "content",
        None,
        &BoardPatch {
            name: Some("Conteúdo!".into()),
            ..Default::default()
        },
    )
    .unwrap();
    ws.archive_card(&card.id).unwrap();
}

fn write_card(dir: &Path, board: &str, id: &str, title: &str, column: &str, rank: &str) -> PathBuf {
    let path = dir.join(format!(
        "boards/{board}/cards/{}--{}.md",
        title.to_lowercase(),
        id.to_lowercase()
    ));
    fs::write(
        &path,
        format!("---\nid: {id}\ntitle: \"{title}\"\ncolumn: {column}\nrank: \"{rank}\"\n---\n\nbody\n"),
    )
    .unwrap();
    path
}

#[test]
fn rejected_updates_never_touch_other_cards() {
    let (dir, ws, _) = sample();
    // Ranks ending in 0 are not valid, so placing anything next to them forces a re-spacing.
    let a = write_card(dir.path(), "dungeon-meister", "01HANDEDIT0A", "A", "todo", "a0");
    let b = write_card(dir.path(), "dungeon-meister", "01HANDEDIT0B", "B", "todo", "b0");
    let before = (fs::read(&a).unwrap(), fs::read(&b).unwrap());
    let task = ws.card(TASK).unwrap();

    // Empty title, unknown network, stale hash: each is refused after the rank is planned.
    let bad_title = CardPatch {
        column: Some("todo".into()),
        title: Some("  ".into()),
        ..Default::default()
    };
    assert!(ws.update_card(TASK, None, &bad_title).is_err());
    let bad_network = CardPatch {
        column: Some("todo".into()),
        network: Some(Some("mastodon".into())),
        ..Default::default()
    };
    assert!(ws.update_card(TASK, None, &bad_network).is_err());
    let moved = CardPatch {
        column: Some("todo".into()),
        ..Default::default()
    };
    assert!(matches!(
        ws.update_card(TASK, Some("not-the-hash"), &moved),
        Err(Error::Conflict { .. })
    ));
    // A bad date on create is refused too.
    assert!(
        ws.create_card(
            "dungeon-meister",
            &NewCard {
                title: "C".into(),
                column: Some("todo".into()),
                due: Some("amanhã".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!((fs::read(&a).unwrap(), fs::read(&b).unwrap()), before);
    assert_eq!(ws.card(TASK).unwrap().hash, task.hash);

    // The same move, valid this time, re-spaces the neighbours and keeps the order.
    ws.move_card(TASK, "todo", Some(1)).unwrap();
    let order: Vec<String> = ws
        .load()
        .unwrap()
        .column_cards("dungeon-meister", "todo")
        .iter()
        .map(|c| c.title.clone())
        .collect();
    assert_eq!(order, ["A", "Resolução em etapas com janelas de reação", "B"]);
}

#[test]
fn auto_archive_moves_only_old_done_cards_by_path() {
    use pauta_core::config::ConfigPatch;
    let (dir, ws, clock) = sample();
    ws.update_config(&ConfigPatch {
        auto_archive_days: Some(Some(30)),
        ..Default::default()
    })
    .unwrap();
    ws.mark_published(RUST_LI, None, None).unwrap();
    assert_eq!(ws.auto_archive().unwrap(), 0, "just finished");
    clock.set(ts("2026-12-30T10:00"));
    assert_eq!(ws.auto_archive().unwrap(), 1);
    assert!(ws.card(RUST_LI).is_err());
    assert!(ws.card(RUST_IG).is_ok(), "not done: stays");
    assert_eq!(ws.list_archive().unwrap().len(), 1);
    assert!(dir.path().join("archive/cards/conteudo").is_dir());
    assert_eq!(ws.auto_archive().unwrap(), 0);
}

// ---- consistency (section 10) ---------------------------------------------------

fn linkedin_goal(index: &pauta_core::Index, now: Timestamp) -> pauta_core::rhythm::CadenceProgress {
    pauta_core::rhythm::cadence(index, now)
        .into_iter()
        .find(|c| c.network == "linkedin")
        .unwrap()
}

/// Moves the clock and returns the index with that instant.
fn at_clock(ws: &Workspace, clock: &FixedClock, when: &str) -> (pauta_core::Index, Timestamp) {
    clock.set(ts(when));
    (ws.load().unwrap(), ts(when))
}

#[test]
fn cadence_counts_the_current_week_monday_to_sunday() {
    let (_dir, ws, clock) = sample();
    ws.mark_published(RUST_LI, Some("https://x/1"), None).unwrap(); // Wed 2026-10-28
    let extra = ws
        .create_card(
            "conteudo",
            &NewCard {
                title: "Outro".into(),
                network: Some("linkedin".into()),
                publication_at: Some("2026-10-29T09:00".into()),
                ..Default::default()
            },
        )
        .unwrap();

    let (index, now) = at_clock(&ws, &clock, "2026-10-28T21:00");
    let g = linkedin_goal(&index, now);
    assert_eq!((g.goal, g.done, g.scheduled), (2, 1, 1), "{g:?}");
    assert_eq!(g.period_start.to_string(), "2026-10-26");
    assert_eq!(g.period_end.to_string(), "2026-11-01");

    ws.create_card(
        "conteudo",
        &NewCard {
            title: "Semana que vem".into(),
            network: Some("linkedin".into()),
            publication_at: Some("2026-11-04T09:00".into()),
            ..Default::default()
        },
    )
    .unwrap();

    // Sunday night is still the same week; Monday starts a new one.
    let (index, now) = at_clock(&ws, &clock, "2026-11-01T23:30");
    let g = linkedin_goal(&index, now);
    assert_eq!(g.done, 1);
    let (index, now) = at_clock(&ws, &clock, "2026-11-02T00:30");
    let g = linkedin_goal(&index, now);
    assert_eq!(
        (g.done, g.scheduled),
        (0, 1),
        "only the Nov 4 post is ahead; the rest is last week"
    );
    assert_eq!(g.period_start.to_string(), "2026-11-02");
    let _ = extra;

    // The Today screen carries the same numbers.
    let (index, _) = at_clock(&ws, &clock, "2026-10-28T21:00");
    let today = pauta_core::schedule::today(&index, ws.now());
    assert_eq!(today.cadence.len(), 2);
}

#[test]
fn history_keeps_counting_archived_posts() {
    use pauta_core::config::ConfigPatch;
    let (_dir, ws, clock) = sample();
    ws.update_config(&ConfigPatch {
        auto_archive_days: Some(Some(30)),
        ..Default::default()
    })
    .unwrap();
    ws.mark_published(RUST_LI, None, Some("2026-10-20T09:00")).unwrap();
    ws.mark_published(RUST_IG, None, Some("2026-10-05T19:00")).unwrap();

    let (index, _) = at_clock(&ws, &clock, "2026-10-28T21:00");
    let h = pauta_core::rhythm::history(&index, ws.now(), 12, 6);
    assert_eq!(h.total_posts, 2);
    assert_eq!(h.weeks.len(), 12);
    assert_eq!(h.weeks.last().unwrap().start.to_string(), "2026-10-26");
    let week = |start: &str| h.weeks.iter().find(|b| b.start.to_string() == start).unwrap();
    assert_eq!(week("2026-10-19").counts["linkedin"], 1);
    assert_eq!(week("2026-10-05").counts["instagram"], 1);
    assert_eq!(h.months.last().unwrap().total, 2, "October");

    // Both cards get archived two months later; the history is unchanged.
    clock.set(ts("2026-12-30T10:00"));
    assert_eq!(ws.auto_archive().unwrap(), 2);
    let (index, _) = at_clock(&ws, &clock, "2026-12-30T10:00");
    assert!(index.cards.iter().all(|c| c.id != RUST_LI && c.id != RUST_IG));
    let h = pauta_core::rhythm::history(&index, ws.now(), 12, 6);
    assert_eq!(h.total_posts, 2);
    assert_eq!(h.months.iter().map(|b| b.total).sum::<u32>(), 2);
    let recent = pauta_core::rhythm::rhythm(&index, ws.now()).recent;
    assert!(recent.iter().all(|p| p.archived));
}

#[test]
fn stale_means_time_in_the_column_not_time_since_the_last_edit() {
    let (_dir, ws, clock) = sample();
    let card = ws
        .create_card(
            "dungeon-meister",
            &NewCard {
                title: "Parado".into(),
                column: Some("todo".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(card.column_since, Some(ts("2026-10-28T21:00")));

    // Twenty days later someone edits the tags: updated is fresh, the card is still stale.
    clock.set(ts("2026-11-17T09:00"));
    let card = ws
        .update_card(
            &card.id,
            None,
            &CardPatch {
                tags: Some(vec!["x".into()]),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(card.updated, Some(ts("2026-11-17T09:00")));
    assert_eq!(card.column_since, Some(ts("2026-10-28T21:00")), "an edit is not a move");
    let stale: Vec<String> = pauta_core::schedule::today(&ws.load().unwrap(), ws.now())
        .stale
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert!(stale.contains(&card.id), "{stale:?}");

    // Moving restarts the clock.
    let moved = ws.move_card(&card.id, "doing", None).unwrap();
    assert_eq!(moved.column_since, Some(ts("2026-11-17T09:00")));
    let stale = pauta_core::schedule::today(&ws.load().unwrap(), ws.now()).stale;
    assert!(stale.iter().all(|c| c.id != card.id));

    // A column changed by hand makes the record point at the wrong column: ignored.
    let text = fs::read_to_string(&moved.path)
        .unwrap()
        .replace("\ncolumn: doing\n", "\ncolumn: review\n");
    fs::write(&moved.path, text).unwrap();
    assert_eq!(ws.card(&card.id).unwrap().column_since, None);
}

#[test]
fn cadence_goal_is_validated_and_editable() {
    use pauta_core::config::{CadenceGoal, ConfigPatch, Period};
    let (dir, ws, _) = sample();
    let mut nets = ws.config().unwrap().networks;
    assert_eq!(
        nets[0].cadence_goal,
        Some(CadenceGoal {
            posts: 2,
            per: Period::Week
        })
    );
    nets[0].cadence_goal = Some(CadenceGoal {
        posts: 8,
        per: Period::Month,
    });
    nets[1].cadence_goal = None;
    ws.update_config(&ConfigPatch {
        networks: Some(nets.clone()),
        ..Default::default()
    })
    .unwrap();
    let text = fs::read_to_string(dir.path().join("pauta.yaml")).unwrap();
    assert!(text.contains("cadence_goal: { posts: 8, per: month }"), "{text}");
    assert!(!text.contains("posts: 2"), "{text}");
    assert_eq!(ws.config().unwrap().networks[1].cadence_goal, None);

    nets[0].cadence_goal = Some(CadenceGoal {
        posts: 0,
        per: Period::Week,
    });
    assert!(
        ws.update_config(&ConfigPatch {
            networks: Some(nets),
            ..Default::default()
        })
        .is_err()
    );

    // Hand-edited nonsense is reported, not ignored.
    let cfg = dir.path().join("pauta.yaml");
    let good = fs::read_to_string(&cfg).unwrap();
    fs::write(&cfg, good.replace("per: month", "per: year")).unwrap();
    assert!(ws.config().unwrap_err().to_string().contains("cadence_goal.per"));
    fs::write(&cfg, good.replace("posts: 8", "posts: -2")).unwrap();
    assert!(ws.config().unwrap_err().to_string().contains("cadence_goal.posts"));
}

#[test]
fn a_snooze_ending_in_quiet_hours_waits_for_them_to_end() {
    let (_dir, ws, _) = sample();
    ws.create_card(
        "conteudo",
        &NewCard {
            title: "Post das dez".into(),
            network: Some("instagram".into()),
            publication_at: Some("2026-11-07T22:00".into()),
            reminders: Some(vec!["0".into()]),
            ..Default::default()
        },
    )
    .unwrap();
    let index = ws.load().unwrap(); // quiet hours 23:00 to 07:00
    let mut state = ReminderState::default();
    let due = state.due(&index, ts("2026-11-07T22:00"));
    let r = due
        .fresh
        .iter()
        .find(|r| r.item.title == "Post das dez")
        .unwrap()
        .clone();
    state.mark_delivered([r.key.clone()], ts("2026-11-07T22:00"));
    // Snoozed at 22:55 for an hour: 23:55 is inside quiet hours.
    state.snooze(&r.key, ts("2026-11-07T23:55"));
    for quiet in ["2026-11-07T23:56", "2026-11-08T03:00", "2026-11-08T06:59"] {
        assert!(state.due(&index, ts(quiet)).fresh.is_empty(), "{quiet}");
    }
    let due = state.due(&index, ts("2026-11-08T07:00"));
    assert!(
        due.fresh.iter().any(|x| x.key == r.key),
        "delivered when quiet hours end"
    );
}

// ---- metrics by hand and .ics export (sections 17.1, 17.3) ----------------------

fn metrics(impressions: Option<u64>, reactions: u64, comments: u64, shares: u64) -> CardPatch {
    use pauta_core::store::MetricsInput;
    CardPatch {
        metrics: Some(Some(MetricsInput {
            impressions,
            reactions: Some(reactions),
            comments: Some(comments),
            shares: Some(shares),
            ..Default::default()
        })),
        ..Default::default()
    }
}

#[test]
fn metrics_are_recorded_on_published_posts_only() {
    let (_dir, ws, _) = sample();
    // Not published yet: refused, nothing written.
    let before = fs::read(ws.card(RUST_LI).unwrap().path).unwrap();
    assert!(ws.update_card(RUST_LI, None, &metrics(Some(10), 1, 1, 1)).is_err());
    assert_eq!(fs::read(ws.card(RUST_LI).unwrap().path).unwrap(), before);
    // No network at all: refused.
    assert!(ws.update_card(TASK, None, &metrics(None, 1, 0, 0)).is_err());

    ws.mark_published(RUST_LI, None, None).unwrap();
    let card = ws.update_card(RUST_LI, None, &metrics(Some(1200), 30, 4, 2)).unwrap();
    let m = card.metrics.clone().unwrap();
    assert_eq!(
        (m.impressions, m.reactions, m.comments, m.shares, m.saves),
        (Some(1200), Some(30), Some(4), Some(2), None)
    );
    assert_eq!(m.engagement(), 36);
    assert_eq!(m.recorded.unwrap().to_string(), "2026-10-28");
    let text = fs::read_to_string(&card.path).unwrap();
    assert!(
        text.contains(
            "metrics:\n  impressions: 1200\n  reactions: 30\n  comments: 4\n  shares: 2\n  recorded: 2026-10-28\n"
        ),
        "{text}"
    );
    assert!(text.contains("mood: empolgado"), "other fields untouched");

    // Correcting a number replaces the block; an empty form removes it.
    let card = ws.update_card(RUST_LI, None, &metrics(Some(1500), 31, 4, 2)).unwrap();
    assert_eq!(card.metrics.unwrap().impressions, Some(1500));
    let card = ws
        .update_card(
            RUST_LI,
            None,
            &CardPatch {
                metrics: Some(Some(Default::default())),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(card.metrics.is_none());
    assert!(!fs::read_to_string(&card.path).unwrap().contains("metrics:"));
    ws.update_card(RUST_LI, None, &metrics(None, 5, 0, 0)).unwrap();
    let card = ws
        .update_card(
            RUST_LI,
            None,
            &CardPatch {
                metrics: Some(None),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(card.metrics.is_none());
}

#[test]
fn insights_come_from_the_authors_own_numbers() {
    let (_dir, ws, clock) = sample();
    let post = |title: &str, at: &str, imp: Option<u64>, r: u64, c: u64, s: u64| {
        let card = ws
            .create_card(
                "conteudo",
                &NewCard {
                    title: title.into(),
                    network: Some("linkedin".into()),
                    publication_at: Some(at.into()),
                    ..Default::default()
                },
            )
            .unwrap();
        ws.mark_published(&card.id, None, Some(at)).unwrap();
        ws.update_card(&card.id, None, &metrics(imp, r, c, s)).unwrap();
    };
    post("Terça A", "2026-10-13T09:00", Some(1000), 8, 1, 1); // Tue, engagement 10
    post("Terça B", "2026-10-20T09:00", Some(2000), 15, 3, 2); // Tue, engagement 20
    post("Quinta", "2026-10-15T09:00", None, 4, 1, 0); // Thu, engagement 5, no impressions
    // A published post without numbers does not take part.
    let plain = ws
        .create_card(
            "conteudo",
            &NewCard {
                title: "Sem números".into(),
                network: Some("instagram".into()),
                ..Default::default()
            },
        )
        .unwrap();
    ws.mark_published(&plain.id, None, Some("2026-10-16T19:00")).unwrap();

    clock.set(ts("2026-10-28T21:00"));
    let index = ws.load().unwrap();
    let i = pauta_core::rhythm::insights(&index);
    assert_eq!((i.posts_with_metrics, i.min_samples), (3, 3));
    assert_eq!(i.networks.len(), 1);
    let li = &i.networks[0];
    assert_eq!(
        (li.network.as_str(), li.posts, li.engagement, li.impressions),
        ("linkedin", 3, 35, 3000)
    );
    assert!((li.avg_engagement - 35.0 / 3.0).abs() < 1e-9);
    // The rate only counts posts that have impressions: 30 / 3000.
    assert!((li.engagement_rate.unwrap() - 0.01).abs() < 1e-9);
    let tue = li.by_weekday.iter().find(|d| d.weekday == 1).unwrap();
    assert_eq!((tue.posts, tue.avg_engagement), (2, 15.0));
    let thu = li.by_weekday.iter().find(|d| d.weekday == 3).unwrap();
    assert_eq!((thu.posts, thu.avg_engagement), (1, 5.0));
    assert_eq!(
        i.top.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(),
        ["Terça B", "Terça A", "Quinta"]
    );
    // Rhythm bundles it.
    assert_eq!(
        pauta_core::rhythm::rhythm(&index, ws.now()).insights.posts_with_metrics,
        3
    );
}

fn ics_at(ws: &Workspace, include_done: bool) -> String {
    let index = ws.load().unwrap();
    pauta_core::ics::export(&index, ws.now(), pauta_core::ics::IcsOptions { include_done })
}

#[test]
fn ics_export_is_valid_and_carries_posts_deadlines_and_alarms() {
    let (_dir, ws, _) = sample();
    let text = ics_at(&ws, false);

    // Well formed: CRLF everywhere, lines folded at 75 octets, parses as iCalendar.
    assert!(!text.replace("\r\n", "").contains('\n'), "bare LF");
    assert!(text.split("\r\n").all(|l| l.len() <= 75), "unfolded line");
    let cals: Vec<_> = ical::IcalParser::new(std::io::BufReader::new(text.as_bytes())).collect();
    assert_eq!(cals.len(), 1);
    let cal = cals.into_iter().next().unwrap().expect("parses");
    assert_eq!(cal.events.len(), 3, "the post, the card deadline and the TCC2 event");

    let prop = |ev: &ical::parser::ical::component::IcalEvent, name: &str| {
        ev.properties
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.value.clone())
            .unwrap_or_default()
    };
    let by_uid = |uid: &str| cal.events.iter().find(|e| prop(e, "UID") == uid).unwrap();

    // 09:00 in Recife (-03:00) is 12:00Z; a post is a tentative 30-minute block with two alarms.
    let post = by_uid("pub-01JBX3K9Q2AB@pauta");
    assert_eq!(prop(post, "SUMMARY"), "[LinkedIn] Por que Rust não tem coletor de lixo");
    assert_eq!(
        (prop(post, "DTSTART").as_str(), prop(post, "DTEND").as_str()),
        ("20261103T120000Z", "20261103T123000Z")
    );
    assert_eq!(prop(post, "STATUS"), "TENTATIVE");
    let triggers: Vec<String> = post.alarms.iter().map(|a| prop_alarm(a, "TRIGGER")).collect();
    assert_eq!(triggers, ["20261103T110000Z", "20261103T120000Z"], "-1h and on time");

    // All-day items use dates with an exclusive end, and alarms at 09:00 local.
    let tcc = by_uid("event-01JBY7P0D5EF@pauta");
    assert_eq!(prop(tcc, "SUMMARY"), "Prazo: Entrega do capítulo 3 do TCC2");
    assert!(text.contains("DTSTART;VALUE=DATE:20261205\r\nDTEND;VALUE=DATE:20261206\r\n"));
    let triggers: Vec<String> = tcc.alarms.iter().map(|a| prop_alarm(a, "TRIGGER")).collect();
    assert_eq!(triggers, ["20261128T120000Z", "20261204T120000Z", "20261205T120000Z"]);
    assert_eq!(
        prop(by_uid("due-01JBZ8M4T1XY@pauta"), "SUMMARY"),
        "Prazo: Resolução em etapas com janelas de reação"
    );
}

fn prop_alarm(a: &ical::parser::ical::component::IcalAlarm, name: &str) -> String {
    a.properties
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| p.value.clone())
        .unwrap_or_default()
}

#[test]
fn ics_skips_finished_items_and_past_alarms_and_escapes_text() {
    let (_dir, ws, clock) = sample();
    // Finished posts are left out unless asked for.
    ws.mark_published(RUST_LI, None, None).unwrap();
    assert!(!ics_at(&ws, false).contains("pub-01JBX3K9Q2AB@pauta"));
    assert!(ics_at(&ws, true).contains("pub-01JBX3K9Q2AB@pauta"));

    // An alarm that already passed is not exported.
    let (_dir2, ws2, clock2) = sample();
    clock2.set(ts("2026-11-03T08:30")); // after the -1h alarm (08:00), before the post
    let text = ics_at(&ws2, false);
    assert!(text.contains("TRIGGER;VALUE=DATE-TIME:20261103T120000Z"));
    assert!(!text.contains("TRIGGER;VALUE=DATE-TIME:20261103T110000Z"));
    let _ = clock;

    // Commas, semicolons, backslashes and line breaks are escaped; long multibyte titles fold on character boundaries.
    let long = "ç".repeat(120);
    for title in ["Olá, mundo; barra \\ fim", long.as_str()] {
        ws.create_card(
            "conteudo",
            &NewCard {
                title: title.into(),
                due: Some("2026-11-20".into()),
                ..Default::default()
            },
        )
        .unwrap();
    }
    let text = ics_at(&ws, false);
    assert!(
        text.contains("SUMMARY:Prazo: Olá\\, mundo\\; barra \\\\ fim\r\n"),
        "{text}"
    );
    let unfolded = text.replace("\r\n ", "");
    assert!(
        unfolded.contains(&format!("SUMMARY:Prazo: {long}\r\n")),
        "folding must not split or drop characters"
    );
    assert!(text.split("\r\n").all(|l| l.len() <= 75));
}

// ---- artwork (ADR 0007) ----------------------------------------------------------

fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, bytes).unwrap();
    p
}

#[test]
fn media_formats_and_art_settings_have_defaults_and_are_editable() {
    use pauta_core::config::{ArtPatch, ConfigPatch, MediaFormat};
    let (dir, ws, _) = sample();
    let cfg = ws.config().unwrap();
    // The sample pauta.yaml has no `media` or `art`: built-in defaults apply.
    let li = cfg.network("linkedin").unwrap();
    assert!(!li.media.requires_media);
    assert_eq!(
        li.media.formats.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(),
        ["image", "carousel"]
    );
    let ig = cfg.network("instagram").unwrap();
    assert!(ig.media.requires_media);
    assert_eq!(
        ig.media.formats.iter().find(|f| f.id == "story").unwrap().size,
        "1080x1920"
    );
    assert_eq!(cfg.art.tool_url, "https://claude.ai");
    assert_eq!(cfg.art.brand, None);

    // Edit a format, add one, flip requires_media, and set the art settings.
    let mut nets = cfg.networks.clone();
    nets[0].media.requires_media = true;
    nets[0].media.formats.push(MediaFormat {
        id: "poll".into(),
        label: "Enquete".into(),
        size: "1200x628".into(),
        note: Some("sem imagem, só texto".into()),
    });
    let updated = ws
        .update_config(&ConfigPatch {
            networks: Some(nets.clone()),
            art: Some(ArtPatch {
                tool_url: Some("https://claude.ai/design".into()),
                brand: Some("Vinho #7a2e3b, serifa, muito espaço em branco".into()),
            }),
            ..Default::default()
        })
        .unwrap();
    assert!(updated.network("linkedin").unwrap().media.requires_media);
    assert_eq!(updated.network("linkedin").unwrap().media.formats.len(), 3);
    assert_eq!(updated.art.tool_url, "https://claude.ai/design");
    assert_eq!(
        updated.art.brand.as_deref(),
        Some("Vinho #7a2e3b, serifa, muito espaço em branco")
    );
    let text = fs::read_to_string(dir.path().join("pauta.yaml")).unwrap();
    assert!(
        text.contains("cadence_goal: { posts: 2, per: week }"),
        "other keys survive: {text}"
    );
    assert!(
        text.contains("  - { id: poll, label: \"Enquete\", size: \"1200x628\", note: \"sem imagem, só texto\" }")
            || text.contains("id: poll"),
        "{text}"
    );

    // Clearing the link returns to the default; bad input is refused without writing.
    let c = ws
        .update_config(&ConfigPatch {
            art: Some(ArtPatch {
                tool_url: Some(String::new()),
                brand: Some(String::new()),
            }),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        (c.art.tool_url.as_str(), c.art.brand.clone()),
        ("https://claude.ai", None)
    );
    let before = fs::read(dir.path().join("pauta.yaml")).unwrap();
    assert!(
        ws.update_config(&ConfigPatch {
            art: Some(ArtPatch {
                tool_url: Some("ftp://x".into()),
                brand: None
            }),
            ..Default::default()
        })
        .is_err()
    );
    nets[0].media.formats[0].size = "grande".into();
    assert!(
        ws.update_config(&ConfigPatch {
            networks: Some(nets.clone()),
            ..Default::default()
        })
        .is_err()
    );
    nets[0].media.formats[0].size = "1200x627".into();
    nets[0].media.formats[1].id = nets[0].media.formats[0].id.clone();
    assert!(
        ws.update_config(&ConfigPatch {
            networks: Some(nets),
            ..Default::default()
        })
        .is_err(),
        "duplicate format id"
    );
    assert_eq!(fs::read(dir.path().join("pauta.yaml")).unwrap(), before);

    // Hand-edited nonsense is reported.
    let cfg_path = dir.path().join("pauta.yaml");
    let good = fs::read_to_string(&cfg_path).unwrap();
    assert!(good.contains("art: {}"), "{good}");
    fs::write(
        &cfg_path,
        good.replace("art: {}", "art:\n  tool_url: \"javascript:alert(1)\""),
    )
    .unwrap();
    assert!(ws.config().unwrap_err().to_string().contains("art.tool_url"));
}

#[test]
fn requesting_art_records_it_on_the_card_and_prepares_the_brief_section() {
    let (_dir, ws, _) = sample();
    // An idea for Instagram: requesting art moves it to production.
    let idea = ws
        .create_card(
            "conteudo",
            &NewCard {
                title: "Carrossel sobre ownership".into(),
                network: Some("instagram".into()),
                body: Some("Legenda do carrossel.".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(idea.column, "idea");
    let card = ws.request_art(&idea.id, Some("carousel"), None).unwrap();
    let art = card.art.clone().unwrap();
    assert_eq!(
        (art.status.as_str(), art.format.as_deref()),
        ("requested", Some("carousel"))
    );
    assert_eq!(art.requested_at, Some(ws.now()));
    assert_eq!(card.column, "producing");
    assert!(
        card.body.contains("## Arte"),
        "a section is ready for the brief: {}",
        card.body
    );
    assert!(card.body.contains("Legenda do carrossel."));
    let text = fs::read_to_string(&card.path).unwrap();
    assert!(
        text.contains("art:\n  status: requested\n  format: carousel\n  requested_at: 2026-10-28T21:00:00-03:00\n"),
        "{text}"
    );

    // A brief replaces the section; asking again keeps what the author wrote.
    let card = ws
        .request_art(&idea.id, None, Some("Fundo vinho, 5 slides, um conceito por slide"))
        .unwrap();
    assert!(card.body.contains("Fundo vinho, 5 slides"));
    assert_eq!(
        card.art.unwrap().format.as_deref(),
        Some("carousel"),
        "format kept when not given"
    );
    let card = ws.request_art(&idea.id, Some("story"), None).unwrap();
    assert!(card.body.contains("Fundo vinho, 5 slides"));
    assert_eq!(card.art.unwrap().format.as_deref(), Some("story"));

    // Errors: unknown format (listing the valid ones), card without a network.
    let err = ws
        .request_art(&idea.id, Some("hologram"), None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("image") && err.contains("story"), "{err}");
    assert!(ws.request_art(TASK, None, None).is_err());
    // A card already further along is not pulled back.
    let scheduled = ws.request_art(RUST_LI, None, None).unwrap();
    assert_eq!(scheduled.column, "scheduled");
}

#[test]
fn readiness_says_what_is_missing_and_clears_as_art_arrives() {
    let (dir, ws, _) = sample();
    let codes = |ws: &Workspace, id: &str| -> Vec<String> {
        ws.load()
            .unwrap()
            .readiness
            .get(id)
            .map(|p| {
                p.iter()
                    .map(|p| format!("{}:{}", p.code, p.detail.clone().unwrap_or_default()))
                    .collect()
            })
            .unwrap_or_default()
    };
    // Instagram needs media; the sample card has text and no files.
    assert_eq!(codes(&ws, RUST_IG), ["media_missing:"]);
    // LinkedIn does not need media.
    assert!(codes(&ws, RUST_LI).is_empty());

    ws.request_art(RUST_IG, Some("image"), None).unwrap();
    assert_eq!(codes(&ws, RUST_IG), ["art_pending:waiting"]);
    assert!(ws.set_art_status(RUST_IG, "ready").is_err(), "nothing attached yet");

    // The exported file lands in the folder (by the app, by hand, or by Claude Code).
    let src = write_file(dir.path(), "arte.png", b"png-bytes");
    let dest = ws.attach_asset(RUST_IG, &src).unwrap();
    assert!(dest.starts_with(dir.path().join("boards/conteudo/assets").join(RUST_IG)));
    assert_eq!(fs::read(&dest).unwrap(), b"png-bytes");
    assert_eq!(ws.card(RUST_IG).unwrap().assets, ["arte.png"]);
    assert_eq!(codes(&ws, RUST_IG), ["art_pending:files_present"]);

    let card = ws.set_art_status(RUST_IG, "ready").unwrap();
    assert_eq!(card.art.unwrap().status.as_str(), "ready");
    assert!(codes(&ws, RUST_IG).is_empty(), "ready to post");
    // Back to waiting, then the record is cleared.
    ws.set_art_status(RUST_IG, "requested").unwrap();
    assert_eq!(codes(&ws, RUST_IG), ["art_pending:files_present"]);
    ws.set_art_status(RUST_IG, "none").unwrap();
    assert!(ws.card(RUST_IG).unwrap().art.is_none());
    assert!(
        codes(&ws, RUST_IG).is_empty(),
        "files are there, so media is not missing"
    );
    assert!(ws.set_art_status(RUST_IG, "maybe").is_err());

    // Text problems: empty and over the limit (2200 on Instagram). The `## Arte` brief never counts.
    let long = ws
        .create_card(
            "conteudo",
            &NewCard {
                title: "Legenda longa".into(),
                network: Some("instagram".into()),
                body: Some(format!("{}\n\n## Arte\n\n{}", "ã".repeat(2300), "brief ".repeat(100))),
                publication_at: Some("2026-11-07T19:00".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        codes(&ws, &long.id).contains(&"over_limit:2300/2200".to_string()),
        "{:?}",
        codes(&ws, &long.id)
    );
    let empty = ws
        .create_card(
            "conteudo",
            &NewCard {
                title: "Vazio".into(),
                network: Some("linkedin".into()),
                publication_at: Some("2026-11-05T09:00".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(codes(&ws, &empty.id), ["no_text:"]);
    // Published posts are not checked.
    ws.mark_published(RUST_LI, None, None).unwrap();
    assert!(!ws.load().unwrap().readiness.contains_key(RUST_LI));
}

#[test]
fn attaching_files_is_safe_and_never_overwrites() {
    let (dir, ws, _) = sample();
    let src = write_file(dir.path(), "slide.png", b"one");
    let first = ws.attach_asset(TASK, &src).unwrap();
    fs::write(&src, b"two").unwrap();
    let second = ws.attach_asset(TASK, &src).unwrap();
    assert_ne!(first, second);
    assert_eq!(second.file_name().unwrap(), "slide-2.png");
    assert_eq!(fs::read(&first).unwrap(), b"one", "the first file is untouched");
    assert_eq!(ws.card(TASK).unwrap().assets, ["slide-2.png", "slide.png"]);

    // Hidden names lose the dot; temp files from a copy in flight are not listed.
    let hidden = write_file(dir.path(), ".segredo.pdf", b"x");
    assert_eq!(
        ws.attach_asset(TASK, &hidden).unwrap().file_name().unwrap(),
        "segredo.pdf"
    );
    let assets = ws.assets_dir(&ws.card(TASK).unwrap());
    fs::write(assets.join(".copy-abc123"), b"partial").unwrap();
    assert!(!ws.card(TASK).unwrap().assets.iter().any(|a| a.starts_with('.')));

    // Not a file, missing, unknown card.
    assert!(ws.attach_asset(TASK, dir.path()).is_err());
    assert!(ws.attach_asset(TASK, &dir.path().join("nao-existe.png")).is_err());
    assert!(ws.attach_asset("NAO-EXISTE", &src).is_err());
    // The source stays where it was (copy, not move).
    assert!(src.exists());
}

#[test]
fn the_art_brief_has_everything_the_design_tool_needs() {
    use pauta_core::config::{ArtPatch, ConfigPatch};
    let (_dir, ws, _) = sample();
    ws.update_config(&ConfigPatch {
        art: Some(ArtPatch {
            tool_url: None,
            brand: Some("Vinho #7a2e3b, serifa".into()),
        }),
        ..Default::default()
    })
    .unwrap();
    ws.request_art(RUST_IG, Some("carousel"), Some("Cinco slides, um conceito por slide"))
        .unwrap();
    let index = ws.load().unwrap();
    let text = pauta_core::art::request_text(&index, index.card(RUST_IG).unwrap()).unwrap();
    for needle in [
        "post do Instagram",
        "Carrossel (4:5) — 1080x1350 px",
        "todos os slides com a mesma proporção",
        "Legenda curta para o carrossel. #rust",
        "Cinco slides, um conceito por slide",
        "Vinho #7a2e3b, serifa",
        "alt text",
        "Exportar em 1080x1350 px",
    ] {
        assert!(text.contains(needle), "{needle} missing in:\n{text}");
    }
    // The brief is not post text, and the post text is not the brief.
    let post = pauta_core::art::text_for_posting(&index, index.card(RUST_IG).unwrap());
    assert!(!post.contains("Cinco slides"), "{post}");

    // Without a section or brand notes there are sensible fallbacks; no network is refused.
    let plain = pauta_core::art::request_text(&index, index.card(RUST_LI).unwrap()).unwrap();
    assert!(
        plain.contains("proponha duas alternativas") && plain.contains("Formato: Imagem única — 1200x627 px"),
        "{plain}"
    );
    assert!(pauta_core::art::request_text(&index, index.card(TASK).unwrap()).is_err());

    // `## Arte` is excluded from the fallback text (a body with no network section).
    let c = ws
        .create_card(
            "conteudo",
            &NewCard {
                title: "Sem seção da rede".into(),
                network: Some("linkedin".into()),
                body: Some("Texto único.\n\n## Arte\n\nbrief secreto\n\n## Notas\n\nnota".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let index = ws.load().unwrap();
    assert_eq!(
        pauta_core::art::text_for_posting(&index, index.card(&c.id).unwrap()),
        "Texto único."
    );
}

#[test]
fn metrics_import_from_csv_matches_by_url_and_previews_first() {
    let (_dir, ws, _) = sample();
    ws.mark_published(RUST_LI, Some("https://www.linkedin.com/posts/arthur_rust-123/"), None)
        .unwrap();
    // Existing numbers: the file's empty cells must not erase them.
    ws.update_card(RUST_LI, None, &metrics(Some(100), 5, 1, 0)).unwrap();
    let path = ws.card(RUST_LI).unwrap().path;
    let before = fs::read(&path).unwrap();

    // Semicolon-separated, Portuguese headers, a BOM, quoted thousands, one unknown post.
    let csv = "\u{feff}Link da publicação;Impressões;Reações;Comentários;Cliques\n\
        linkedin.com/posts/arthur_rust-123?utm_source=x;\"1.250\";40;;7\n\
        https://linkedin.com/posts/outro-post;900;1;1;1\n";
    let preview = ws.preview_metrics_import(csv).unwrap();
    assert_eq!(preview.matched.len(), 1);
    assert_eq!(
        preview.unmatched,
        [(3, "https://linkedin.com/posts/outro-post".to_string())]
    );
    let m = &preview.matched[0];
    assert_eq!(m.card_id, RUST_LI);
    assert_eq!(
        (m.after.impressions, m.after.reactions, m.after.comments, m.after.clicks),
        (Some(1250), Some(40), Some(1), Some(7)),
        "empty comments cell keeps the old 1"
    );
    assert_eq!(
        preview.columns.iter().find(|c| c.field == "url").unwrap().header,
        "Link da publicação"
    );
    assert_eq!(fs::read(&path).unwrap(), before, "a preview never writes");

    let done = ws.import_metrics(csv).unwrap();
    assert_eq!(done.applied, [RUST_LI]);
    assert!(done.failed.is_empty());
    let card = ws.card(RUST_LI).unwrap();
    let met = card.metrics.unwrap();
    assert_eq!(
        (met.impressions, met.reactions, met.comments, met.clicks),
        (Some(1250), Some(40), Some(1), Some(7))
    );
    assert!(fs::read_to_string(&card.path).unwrap().contains("mood: empolgado"));
}

#[test]
fn metrics_import_rejects_unusable_files_and_flags_odd_cells() {
    let (_dir, ws, _) = sample();
    // No URL column / no numbers: a clear error, not an empty import.
    assert!(ws.preview_metrics_import("Título,Curtidas\nx,1\n").is_err());
    assert!(ws.preview_metrics_import("URL,Título\nhttps://x.com/a,x\n").is_err());
    assert!(ws.preview_metrics_import("").is_err());

    ws.mark_published(RUST_LI, Some("https://example.com/a"), None).unwrap();
    // "1,5 mil" and "12%" are not whole numbers: reported and left out, not guessed.
    let p = ws
        .preview_metrics_import("url,views,likes,comments\nhttps://example.com/a,\"1,5 mil\",12%,3\n")
        .unwrap();
    assert_eq!(p.skipped_cells.len(), 2);
    assert_eq!(p.matched[0].after.comments, Some(3));
    assert_eq!(p.matched[0].after.impressions, None);

    // A post that is not published yet is not a match, even with the same URL.
    let p = ws
        .preview_metrics_import("url,likes\nhttps://example.com/zzz,3\n")
        .unwrap();
    assert!(p.matched.is_empty() && p.unmatched.len() == 1);
}
