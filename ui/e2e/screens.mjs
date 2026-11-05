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
      item("due", "t", "Enviar o rascunho ao orientador", w("2026-10-28", null), { board: "tcc2" }),
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
// Records the commands sent to the backend, so the global shortcuts can be checked end to end.
const recordCalls = () => {
  window.__calls = [];
  window.__override = (cmd) => void window.__calls.push(cmd);
};
const check = await newPage("light", { init: recordCalls });
const shortcuts = await check.locator("nav kbd").allTextContents();
const expected = ["Alt+1", "Alt+2", "Alt+3", "Alt+4", "Alt+5", "Ctrl+Shift+I", "Ctrl+K"];
if ([...shortcuts].sort().join() !== [...expected].sort().join()) problems.push(`sidebar shortcuts: ${shortcuts}`);
// Every shortcut shown in the sidebar is announced to assistive technology and drawn as decoration.
const kbdProblems = await check.evaluate(() =>
  [...document.querySelectorAll("nav button")].flatMap((b) => {
    const kbd = b.querySelector("kbd");
    const keys = kbd?.textContent.replace("Ctrl", "Control");
    const out = [];
    if (kbd?.getAttribute("aria-hidden") !== "true") out.push(`${keys}: kbd is not aria-hidden`);
    if (b.getAttribute("aria-keyshortcuts") !== keys) out.push(`${keys}: aria-keyshortcuts is ${b.getAttribute("aria-keyshortcuts")}`);
    return out;
  }),
);
problems.push(...kbdProblems.map((p) => `sidebar: ${p}`));
const clipped = await check.evaluate(() =>
  [...document.querySelectorAll("nav button")].filter((b) => b.scrollWidth > b.clientWidth).map((b) => b.textContent),
);
if (clipped.length) problems.push(`sidebar text does not fit: ${clipped}`);
// WebKit sets this text wider than Chromium, so widen it here: the label stays whole and the
// shortcut is what moves to a second line inside the button.
const capture = await check.evaluate(() => {
  const button = document.querySelector("nav .capture");
  const label = button.querySelector(".label");
  label.style.letterSpacing = "1.5px";
  const range = document.createRange();
  range.selectNodeContents(label);
  const lines = new Set([...range.getClientRects()].map((r) => Math.round(r.top))).size;
  const kbd = button.querySelector("kbd").getBoundingClientRect();
  const box = button.getBoundingClientRect();
  const out = { lines, inside: kbd.right <= box.right + 0.5 && kbd.bottom <= box.bottom + 0.5 };
  label.style.letterSpacing = "";
  return out;
});
if (capture.lines !== 1) problems.push(`capture label breaks into ${capture.lines} lines`);
if (!capture.inside) problems.push("capture shortcut falls outside the button");
for (const [key, label] of [["1", "Hoje"], ["2", "Quadros"], ["3", "Calendário"], ["4", "Ritmo"], ["5", "Configurações"]]) {
  await check.keyboard.press(`Alt+${key}`);
  const current = await check.locator('nav [aria-current="page"]').textContent();
  if (!current?.includes(label)) problems.push(`Alt+${key} did not open ${label}: ${current}`);
}

// Ctrl+K opens the palette and toggles it closed; Ctrl+Shift+I asks the backend for the capture window.
await check.keyboard.press("Control+k");
await check.waitForSelector("dialog[open]", { timeout: 2000 }).catch(() => problems.push("Ctrl+K did not open the palette"));
await check.keyboard.press("Escape");
await check.keyboard.press("Control+Shift+I");
if (!(await check.evaluate(() => window.__calls)).includes("open_capture")) problems.push("Ctrl+Shift+I did not open the capture window");

// Done items are marked by a check that stays inside the item, never by color or strike-through alone.
const doneProblems = (page, scope) =>
  page.evaluate((scope) => {
    const items = [...document.querySelectorAll(`${scope} .item.done`)];
    const out = items.flatMap((item) => {
      const box = item.getBoundingClientRect();
      const check = item.querySelector('svg[data-icon="check"]');
      const found = [];
      if (getComputedStyle(item).textDecorationLine.includes("line-through")) found.push("struck through");
      if (!check) return [...found, "no check icon"];
      const c = check.getBoundingClientRect();
      if (c.left < box.left - 0.5 || c.right > box.right + 0.5) found.push(`check outside the item (${c.left}-${c.right} in ${box.left}-${box.right})`);
      return found;
    });
    return { n: items.length, out };
  }, scope);

await check.keyboard.press("Alt+3");
await check.waitForSelector(".cells");
const struck = await check.evaluate(() =>
  [...document.querySelectorAll(".cells *")].filter((e) => getComputedStyle(e).textDecorationLine.includes("line-through")).length,
);
if (struck) problems.push(`calendar: ${struck} element(s) still struck through`);
const month = await doneProblems(check, ".cells");
if (month.n === 0) problems.push("calendar month: no done item to check");
problems.push(...month.out.map((o) => `calendar month: done item: ${o}`));

await check.click("button:has-text('Agenda')");
const agenda = await doneProblems(check, ".agenda");
if (agenda.n === 0) problems.push("calendar agenda: no done item to check");
problems.push(...agenda.out.map((o) => `calendar agenda: done item: ${o}`));

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

// The week view with three done posts at the same hour: the blocks share the day, so each one is
// narrow, and the check must still be whole. The narrowest window is the hard case.
const overlap = await newPage("light", {
  init: () => {
    const done = (id, time) => ({
      kind: "publication", id, board: "conteudo", title: "Post " + id, end: null, network: "linkedin", status: "published", column: null, done: true,
      when: { date: "2026-10-28", time: time + ":00", at: "2026-10-28T" + time + ":00Z" },
    });
    window.__override = (cmd) => (cmd === "get_agenda" ? [done("a", "09:00"), done("b", "09:10"), done("c", "09:20")] : undefined);
  },
});
await overlap.keyboard.press("Alt+3");
await overlap.click("button:has-text('Semana')");
await overlap.waitForSelector(".week .item.done");
for (const width of [1360, 720]) {
  await overlap.setViewportSize({ width, height: 900 });
  await overlap.locator(".week .item.done").first().scrollIntoViewIfNeeded();
  const week = await doneProblems(overlap, ".week");
  if (week.n !== 3) problems.push(`calendar week at ${width}px: ${week.n} done items, expected 3`);
  problems.push(...week.out.map((o) => `calendar week at ${width}px: done item: ${o}`));
}
await overlap.close();

// A timed post with a long title: a short block must never show a line of text cut in half.
const timed = await newPage("light", {
  init: () => {
    const when = { date: "2026-10-28", time: "09:00:00", at: "2026-10-28T09:00:00Z" };
    const post = { kind: "publication", id: "a", board: "conteudo", title: "Um título longo que certamente não cabe na largura de um dia", when, end: null, network: "linkedin", status: "scheduled", column: null, done: false };
    window.__override = (cmd) => (cmd === "get_agenda" ? [post] : undefined);
  },
});
await timed.keyboard.press("Alt+3");
await timed.click("button:has-text('Semana')");
await timed.waitForSelector(".week .col .item");
const sliced = await timed.evaluate(() => {
  const item = document.querySelector(".week .col .item");
  const bottom = item.getBoundingClientRect().bottom;
  const range = document.createRange();
  range.selectNodeContents(item.querySelector(".txt"));
  return [...range.getClientRects()].filter((r) => r.bottom > bottom + 0.5).length;
});
if (sliced) problems.push(`calendar week: ${sliced} line(s) of a title cut by the block's edge`);
await timed.close();

const narrow = await newPage("light", { init: fullToday });
await narrow.setViewportSize({ width: 720, height: 900 });
await narrow.waitForSelector("text=Entrega parcial");
const overflow = await narrow.evaluate(() => {
  const c = document.querySelector("main.content");
  return c.scrollWidth - c.clientWidth;
});
if (overflow > 0) problems.push(`today at 720px overflows by ${overflow}px`);
// The counts do not fuse with the section names, and are not dimmed by opacity.
for (const name of ["Postar hoje 2", "Atrasados 1", "Próximos 7 dias 4"]) {
  if ((await narrow.getByRole("heading", { name, exact: true, level: 2 }).count()) !== 1) problems.push(`today: no heading named "${name}"`);
}
const counts = await narrow.evaluate(() =>
  [...document.querySelectorAll(".section-label .n")].map((n) => getComputedStyle(n).opacity),
);
if (counts.length !== 3 || counts.some((o) => o !== "1")) problems.push(`today: section counts are dimmed: ${counts}`);
// A deadline today stands out from the others in the gutter; a late item is red.
const gutters = await narrow.evaluate(() => {
  const color = (v) => {
    const probe = document.createElement("span");
    probe.style.color = `var(${v})`;
    document.body.append(probe);
    const c = getComputedStyle(probe).color;
    probe.remove();
    return c;
  };
  const gut = (text) => [...document.querySelectorAll(".gut")].find((g) => g.textContent.includes(text));
  return {
    today: getComputedStyle(gut("hoje")).color === color("--warning"),
    later: getComputedStyle(gut("amanhã")).color === color("--warning"),
    late: getComputedStyle(gut("ontem")).color === color("--danger"),
  };
});
if (!gutters.today) problems.push("today: a deadline due today is not amber");
if (gutters.later) problems.push("today: a deadline for tomorrow is amber");
if (!gutters.late) problems.push("today: a late item is not red");
await narrow.close();

await finish(`ok: ${screens.length * 2} states, no page errors, no axe violations, behavior checks passed`);
