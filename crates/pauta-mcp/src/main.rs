//! `pauta-mcp`: MCP server over stdio, started by Claude Code / Claude Desktop.
//!
//! It opens the data folder directly through `pauta-core`; the app does not need
//! to be running. Usage: `pauta-mcp [--root <data folder>]` (or `PAUTA_ROOT`).

mod prompts;
mod resources;
mod server;

use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut root = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => root = args.next().map(|r| pauta_core::locate::expand(&r)),
            "--help" | "-h" => {
                println!("pauta-mcp [--root <pasta de dados>]\nServidor MCP (stdio) do Pauta.");
                return Ok(());
            }
            other => anyhow::bail!("argumento desconhecido: {other}"),
        }
    }
    let root = root.unwrap_or_else(pauta_core::locate::resolve_root);
    // Fail early with a clear message (on stderr, which clients show in their logs).
    pauta_core::Workspace::open(&root)?;
    let service = server::Pauta::new(root).serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
