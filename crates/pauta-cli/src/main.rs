//! Debugging aid for the data folder (milestone 1). Not a user-facing tool.
//!
//! pauta-cli [--root DIR] boards | cards [BOARD] | today | agenda [FROM TO] | invalid | init
//!           | index-json | agenda-json FROM TO   (JSON as the app receives it)

use anyhow::Context;
use pauta_core::schedule;
use pauta_core::time::{When, fmt_hm};
use pauta_core::{Workspace, locate};

fn when(w: &When) -> String {
    w.time
        .map_or_else(|| w.date.to_string(), |t| format!("{} {}", w.date, fmt_hm(t)))
}

fn main() -> anyhow::Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let root = match args.iter().position(|a| a == "--root") {
        Some(i) => {
            let r = args.get(i + 1).context("--root precisa de um caminho")?.clone();
            args.drain(i..=i + 1);
            locate::expand(&r)
        }
        None => locate::resolve_root(),
    };
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    if cmd == "init" {
        let ws = Workspace::init(&root, &pauta_core::templates::default_seed())?;
        println!("pasta de dados pronta em {}", ws.root().display());
        return Ok(());
    }
    // PAUTA_NOW pins the clock, so fixtures and examples are reproducible.
    let ws = Workspace::open(&root)?.with_clock(locate::clock());
    let index = ws.load()?;
    match cmd {
        "boards" => {
            for b in &index.boards {
                println!("{} ({}, {})", b.name, b.id, b.kind.as_str());
                for c in &b.columns {
                    println!(
                        "  {:<12} {:<12} {}",
                        c.id,
                        c.name,
                        index.column_cards(&b.id, &c.id).len()
                    );
                }
            }
        }
        "cards" => {
            for c in index.cards.iter().filter(|c| args.get(1).is_none_or(|b| *b == c.board)) {
                let net = c.network.as_deref().unwrap_or("-");
                println!("{:<26} {:<16} {:<10} {:<9} {}", c.id, c.board, c.column, net, c.title);
            }
        }
        "today" => println!("{}", serde_json::to_string_pretty(&schedule::today(&index, ws.now()))?),
        "rhythm" => println!(
            "{}",
            serde_json::to_string_pretty(&pauta_core::rhythm::rhythm(&index, ws.now()))?
        ),
        "ics" => print!(
            "{}",
            pauta_core::ics::export(
                &index,
                ws.now(),
                pauta_core::ics::IcsOptions {
                    include_done: args.iter().any(|a| a == "--all")
                }
            )
        ),
        "index-json" => println!("{}", serde_json::to_string(&index)?),
        "agenda-json" => {
            let from = args.get(1).context("agenda-json DE ATÉ")?.parse()?;
            let to = args.get(2).context("agenda-json DE ATÉ")?.parse()?;
            println!("{}", serde_json::to_string(&schedule::agenda(&index, from, to))?);
        }
        "agenda" => {
            let tz = index.config.tz.clone();
            let today = ws.now().to_zoned(tz).date();
            let from = args.get(1).map(|s| s.parse()).transpose()?.unwrap_or(today);
            let to = args
                .get(2)
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(from.saturating_add(jiff::ToSpan::days(30)));
            for i in schedule::agenda(&index, from, to) {
                println!(
                    "{:<17} {:<12} {}",
                    when(&i.when),
                    format!("{:?}", i.kind).to_lowercase(),
                    i.title
                );
            }
        }
        "invalid" => {
            for f in &index.invalid {
                println!("{}: {}", f.path.display(), f.error);
            }
        }
        _ => println!(
            "uso: pauta-cli [--root DIR] boards | cards [QUADRO] | today | agenda [DE ATÉ] | invalid | init | rhythm | ics [--all] | index-json | agenda-json DE ATÉ"
        ),
    }
    Ok(())
}
