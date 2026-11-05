// Renders every screen of the built UI in light and dark, takes screenshots and runs
// axe-core (WCAG 2.2 AA) on each state.
//
//   pnpm build && pnpm test:a11y            # fails on any violation
//   SHOTS=/some/dir pnpm test:a11y          # also keeps the screenshots
import { audit, finish, newPage, problems } from "./harness.mjs";

// The fixtures only have a finished post, so this fills every section of Today, with titles long
// enough to wrap. Runs in the browser as an init script.
const fullToday = () => {
  const w = (date, time) => ({ date, time: time ? time + ":00" : null, at: time ? date + "T" + time + ":00Z" : null });
  const item = (kind, id, title, when, o = {}) => ({ kind, id, board: "conteudo", title, when, end: null, network: null, status: null, column: null, done: false, ...o });
  const long = "Um título realmente muito longo de cartão que precisa quebrar em várias linhas sem nunca estourar a largura nem passar por cima da margem do caderno";
  const today = {
    date: "2026-10-28", time: "10:00:00",
    posts: [
      item("publication", "01FIXTURE0000", "Artigo publicado no SBC: resumo para leigos", w("2026-10-28", "08:30"), { network: "linkedin", done: true, status: "published" }),
      item("publication", "01JBX3K9Q2AB", long, w("2026-10-28", "18:30"), { network: "instagram" }),
    ],
    overdue: [item("due", "01JBX3K9Q2AB", "Revisar capítulo 3 do TCC2", w("2026-10-27", null), { board: "tcc2" })],
    upcoming: [
      item("due", "a", "Entrega parcial do TCC2", w("2026-10-29", null), { board: "tcc2" }),
      item("publication", "b", long, w("2026-10-30", "09:00"), { network: "linkedin" }),
      item("event", "c", "Banca de qualificação", w("2026-11-02", "14:00"), { board: null }),
    ],
    empty_queues: [{ network: "instagram", label: "Instagram", days: 5, candidates: [{ id: "x", board: "conteudo", title: "Carrossel: 5 mitos sobre IA generativa", column: "draft", network: "instagram" }] }],
    stale: [{ id: "s", board: "conteudo", title: "Ideia parada há semanas", column: "idea", network: null }],
    cadence: [{ network: "linkedin", label: "LinkedIn", goal: 2, per: "week", done: 1, scheduled: 0, period_start: "2026-10-26", period_end: "2026-11-01" }],
    invalid_files: 0,
  };
  window.__override = (cmd) => (cmd === "get_today" ? today : undefined);
};

const screens = [
  ["today", async (p) => { await p.waitForSelector("text=Seu ritmo"); }],
  ["today-full", async (p) => { await p.waitForSelector("text=Entrega parcial"); }, fullToday],
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
  [
    "error-toast",
    async (p) => {
      await p.click("nav >> text=Quadros");
      await p.click('[data-id="01JBX3K9Q2AB"]');
      await p.click('aside button:has-text("Sugerir horário")');
      await p.waitForSelector(".toast.error");
    },
    () => {
      window.__override = (cmd) => (cmd === "find_slots" ? Promise.reject("falha simulada") : undefined);
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

// ---- behavior that the redesign must keep or fix ------------------------------
const check = await newPage("light");
const shortcuts = await check.locator("nav kbd").allTextContents();
const expected = ["Alt+1", "Alt+2", "Alt+3", "Alt+4", "Alt+5", "Ctrl+Shift+I", "Ctrl+K"];
if ([...shortcuts].sort().join() !== [...expected].sort().join()) problems.push(`sidebar shortcuts: ${shortcuts}`);
const clipped = await check.evaluate(() =>
  [...document.querySelectorAll("nav button")].filter((b) => b.scrollWidth > b.clientWidth).map((b) => b.textContent),
);
if (clipped.length) problems.push(`sidebar text does not fit: ${clipped}`);
for (const [key, label] of [["1", "Hoje"], ["2", "Quadros"], ["3", "Calendário"], ["4", "Ritmo"], ["5", "Configurações"]]) {
  await check.keyboard.press(`Alt+${key}`);
  const current = await check.locator('nav [aria-current="page"]').textContent();
  if (!current?.includes(label)) problems.push(`Alt+${key} did not open ${label}: ${current}`);
}

await check.keyboard.press("Alt+3");
await check.waitForSelector(".cells");
const struck = await check.evaluate(() =>
  [...document.querySelectorAll(".cells *")].filter((e) => getComputedStyle(e).textDecorationLine.includes("line-through")).length,
);
if (struck) problems.push(`calendar: ${struck} element(s) still struck through`);

await check.keyboard.press("Alt+4");
await check.waitForSelector(".lbl");
const cut = await check.evaluate(() =>
  [...document.querySelectorAll(".lbl")].filter((l) => {
    const box = l.closest(".chart").getBoundingClientRect();
    return l.scrollHeight > l.clientHeight || l.getBoundingClientRect().bottom > box.bottom;
  }).length,
);
if (cut) problems.push(`rhythm: ${cut} chart label(s) cut off`);
await check.close();

const narrow = await newPage("light", { init: fullToday });
await narrow.setViewportSize({ width: 720, height: 900 });
await narrow.waitForSelector("text=Entrega parcial");
const overflow = await narrow.evaluate(() => {
  const c = document.querySelector("main.content");
  return c.scrollWidth - c.clientWidth;
});
if (overflow > 0) problems.push(`today at 720px overflows by ${overflow}px`);
await narrow.close();

await finish(`ok: ${screens.length * 2} states, no page errors, no axe violations, behavior checks passed`);
