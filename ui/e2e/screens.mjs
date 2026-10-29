// Renders every screen of the built UI in light and dark, takes screenshots and runs
// axe-core (WCAG 2.2 AA) on each state.
//
//   pnpm build && pnpm test:a11y            # fails on any violation
//   SHOTS=/some/dir pnpm test:a11y          # also keeps the screenshots
import { audit, finish, newPage, problems } from "./harness.mjs";

const screens = [
  ["today", async (p) => { await p.waitForSelector("text=Seu ritmo"); }],
  ["boards", async (p) => { await p.click("nav >> text=Quadros"); await p.waitForSelector("[data-id]"); }],
  ["card-editor", async (p) => { await p.click("nav >> text=Quadros"); await p.click('[data-id="01JBX3K9Q2AB"]'); await p.waitForSelector("aside.editor"); }],
  ["card-editor-art", async (p) => { await p.click("nav >> text=Quadros"); await p.click('[data-id="01FIXTUREART1"]'); await p.waitForSelector("text=Arquivos de arte (2)"); }],
  ["card-editor-published", async (p) => { await p.click("nav >> text=Quadros"); await p.click('[data-id="01FIXTURE0000"]'); await p.waitForSelector("text=Resultados"); }],
  ["calendar-month", async (p) => { await p.click("nav >> text=Calendário"); await p.waitForSelector(".cells"); }],
  ["calendar-week", async (p) => { await p.click("nav >> text=Calendário"); await p.click("button:has-text('Semana')"); await p.waitForSelector(".week .col"); }],
  ["calendar-agenda", async (p) => { await p.click("nav >> text=Calendário"); await p.click("button:has-text('Agenda')"); }],
  ["rhythm", async (p) => { await p.click("nav >> text=Ritmo"); await p.waitForSelector("text=Posts por semana"); }],
  [
    "metrics-import",
    async (p) => {
      await p.click("nav >> text=Ritmo");
      await p.click("button:has-text('Importar de um CSV')");
      await p.setInputFiles("input[type=file]", { name: "posts.csv", mimeType: "text/csv", buffer: Buffer.from("url,likes\nx,1\n") });
      await p.waitForSelector("text=Posts que serão atualizados (2)");
    },
    () => {
      const m = (id, title, before, after) => ({ line: 2, card_id: id, title, network: "linkedin", before, after });
      const none = { impressions: null, reactions: null, comments: null, shares: null, saves: null, clicks: null, recorded: null };
      window.__override = (cmd) =>
        cmd === "preview_metrics_import"
          ? {
              columns: [{ field: "url", header: "Link da publicação" }, { field: "impressions", header: "Impressões" }, { field: "reactions", header: "Reações" }],
              matched: [
                m("01FIXTURE0000", "Post publicado de exemplo 0", { ...none, impressions: 800, reactions: 10 }, { ...none, impressions: 1250, reactions: 10, recorded: "2026-10-28" }),
                m("01FIXTURE0100", "Post publicado de exemplo 1", null, { ...none, impressions: 400, reactions: 3, recorded: "2026-10-28" }),
              ],
              unmatched: [[4, "https://linkedin.com/posts/desconhecido"]],
              skipped_cells: [[3, "Impressões", "1,5 mil"]],
            }
          : undefined;
    },
  ],
  ["settings", async (p) => { await p.click("nav >> text=Configurações"); await p.waitForSelector("text=Conector MCP"); }],
  ["palette", async (p) => { await p.keyboard.press("Control+k"); await p.waitForSelector("dialog[open]"); }],
];

for (const scheme of ["light", "dark"]) {
  for (const [name, go, init] of screens) {
    const page = await newPage(scheme, { init });
    try {
      await go(page);
      await page.waitForTimeout(150);
      await audit(page, name, scheme);
    } catch (e) {
      problems.push(`[${scheme}] ${name}: could not render: ${e.message.split("\n")[0]}`);
    }
    await page.close();
  }
}

await finish(`ok: ${screens.length * 2} states, no page errors, no axe violations`);
