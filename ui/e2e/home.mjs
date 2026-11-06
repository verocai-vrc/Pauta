// Behavior of the Início screen, on the built UI. One group per widget, so it can run while the
// others are unfinished:
//
//   pnpm build && node ui/e2e/home.mjs
//   ONLY=ideas,projects node ui/e2e/home.mjs
import path from "node:path";
import { finish, newPage, problems, shots } from "./harness.mjs";
import { allPostedHome, emptyHome, fullHome, lateHome, longNextHome, noContentBoardHome, noPostsHome, typicalHome } from "./home-fixtures.mjs";

const only = process.env.ONLY?.split(",");
const groups = [];
const group = (name, fn) => groups.push([name, fn]);
const expect = (ok, message) => void (ok || problems.push(message));
const w = (name) => `[data-widget="${name}"]`;
const onView = async (p, label) => (await p.locator('nav [aria-current="page"]').textContent())?.includes(label);
const calls = (p, cmd) => p.evaluate((cmd) => window.__calls.filter((c) => c.cmd === cmd), cmd);
const text = (p, selector) => p.locator(selector).first().textContent({ timeout: 3000 }).catch(() => "");
const more = (name) => `${w(name)} .more`;

// Wraps a scenario so `body` can change what a command answers. `r` is the scenario's own answer.
const layered = (base, body) => ({
  content: `${base.content};(() => { const inner = window.__override; window.__override = (cmd, args) => { const r = inner(cmd, args); ${body}; return r; }; })();`,
});
// Captures the app's minute interval, and puts document.hasFocus() under the test's control.
const withTick = (base) => ({
  content: `${base.content};(() => {
    const real = window.setInterval;
    window.setInterval = (f, ms, ...a) => { if (ms === 60000) window.__tick = f; return real(f, ms, ...a); };
    window.__focused = true;
    document.hasFocus = () => window.__focused;
  })();`,
});
// The backend's answer when midnight has passed.
const nextDay = (p) =>
  p.evaluate(async () => {
    const today = await window.__TAURI_INTERNALS__.invoke("get_today", {});
    window.__today = { ...today, date: "2026-11-04" };
  });

async function home(init, size) {
  const p = await newPage("light", { init });
  if (size) await p.setViewportSize(size);
  await p.waitForSelector(w("ideas"));
  await p.waitForFunction(() => window.__calls?.some((c) => c.cmd === "get_agenda"));
  await p.waitForTimeout(100);
  return p;
}

const onTodayTab = (p) => p.locator("button[aria-pressed='true']:text-is('Hoje')").count();

group("nav", async () => {
  const p = await home(typicalHome);
  expect(await onView(p, "Início"), "the app does not open on Início");
  const labels = await p.locator("nav ul .label").allTextContents();
  expect(labels.join() === "Início,Quadros,Calendário,Ritmo,Configurações", `sidebar: ${labels}`);
  expect((await p.locator("h1").count()) === 1, "Início needs exactly one h1");
  expect(!(await p.locator("main.content.paper").count()), "Início must not sit on ruled paper");

  await p.keyboard.press("Alt+2");
  await p.keyboard.press("Alt+1");
  expect(await onView(p, "Início"), "Alt+1 does not open Início");

  // The Hoje tab replaces the date controls, and "Hoje" stops meaning two things.
  await p.keyboard.press("Alt+3");
  await p.waitForSelector(".cells");
  const tabs = await p.locator("[role=group] button[aria-pressed]").allTextContents();
  expect(tabs.join() === "Hoje,Mês,Semana,Agenda", `calendar tabs: ${tabs}`);
  expect(await p.locator("button:text-is('Ir para hoje')").count(), "no 'Ir para hoje' button");
  await p.click("button:text-is('Hoje')");
  await p.waitForSelector(".today");
  expect((await p.locator("h1").textContent()) === "Calendário", "the calendar h1 changed");
  expect((await p.getByRole("heading", { name: "Hoje", exact: true, level: 2 }).count()) === 1, "Hoje is not an h2");
  for (const gone of ["button:text-is('Ir para hoje')", `button[aria-label="Anterior"]`, "select"]) {
    expect(!(await p.locator(gone).count()), `the Hoje tab still shows ${gone}`);
  }
  expect(await p.locator("button:has-text('Novo evento')").count(), "the Hoje tab lost 'Novo evento'");
  expect(await p.locator("main.content.paper").count(), "the Hoje tab is not on ruled paper");

  // The tray, the notifications and the palette all still reach Hoje.
  await p.keyboard.press("Alt+1");
  await p.evaluate(() => window.__emit("pauta://navigate", "today"));
  await p.waitForSelector(".today");
  expect((await onView(p, "Calendário")) && (await onTodayTab(p)), "pauta://navigate today does not open the Hoje tab");
  await p.evaluate(() => window.__emit("pauta://navigate", "rhythm"));
  await p.waitForSelector("text=Posts por semana");
  expect(await onView(p, "Ritmo"), "pauta://navigate no longer changes the view");

  await p.keyboard.press("Alt+1");
  await p.keyboard.press("Control+k");
  await p.waitForSelector("dialog[open]");
  await p.keyboard.type("Hoje");
  await p.keyboard.press("Enter");
  await p.waitForSelector(".today");
  expect(await onTodayTab(p), "the palette does not open the Hoje tab");
  await p.keyboard.press("Control+k");
  await p.waitForSelector("dialog[open]");
  await p.keyboard.type("Início");
  await p.keyboard.press("Enter");
  await p.waitForSelector(w("ideas"));
  await p.close();
});

group("fit", async () => {
  // 1920x1000 is what a maximized window really gets on a 1080p screen, under the system bar and the title bar.
  for (const [width, height] of [[1280, 820], [1920, 1000], [1920, 1080]]) {
    const p = await home(fullHome, { width, height });
    const m = await p.evaluate(() => {
      const c = document.querySelector("main.content");
      const widgets = [...document.querySelectorAll("[data-widget]")];
      const cols = [...new Set(widgets.map((x) => Math.round(x.getBoundingClientRect().width)))];
      // The Brief is a card too, and the one where a deadline's title lands.
      const spill = [...widgets, document.querySelector(".brief")].flatMap((x) => {
        const box = x.getBoundingClientRect();
        return [...x.querySelectorAll("*")]
          .filter((e) => !e.closest(".sr-only") && e.getClientRects().length && e.getBoundingClientRect().right > box.right + 0.5)
          .map((e) => `${x.dataset.widget ?? "brief"}: ${e.tagName}.${e.className}`);
      });
      return { y: c.scrollHeight - c.clientHeight, x: c.scrollWidth - c.clientWidth, cols, spill: spill.slice(0, 5), widgets: widgets.length };
    });
    if (shots) await p.screenshot({ path: path.join(shots, `home-full-${width}x${height}.png`) });
    expect(m.widgets === 6, `${width}px: ${m.widgets} widgets besides the Brief`);
    expect(m.y <= 0, `${width}x${height}: the worst day needs ${m.y}px of scrolling`);
    expect(m.x <= 0, `${width}x${height}: overflows sideways by ${m.x}px`);
    expect(m.cols.length === 1, `${width}x${height}: columns have different widths: ${m.cols}`);
    expect(!m.spill.length, `${width}x${height}: text spills out of a card: ${m.spill}`);
    await p.close();
  }
  // The next commitment is a deadline with a title that cannot break (the Brief once grew sideways).
  for (const [width, height] of [[1280, 820], [1920, 1000], [1100, 700], [720, 480]]) {
    const p = await home(longNextHome, { width, height });
    const m = await p.evaluate(() => {
      const c = document.querySelector("main.content");
      const next = document.querySelector(".brief .next");
      const line = parseFloat(getComputedStyle(next).lineHeight);
      return { x: c.scrollWidth - c.clientWidth, rows: Math.round(next.getBoundingClientRect().height / line), title: next.getAttribute("title") };
    });
    expect(m.x <= 0, `${width}x${height}: a long deadline in the Brief overflows sideways by ${m.x}px`);
    expect(m.rows === 1, `${width}x${height}: the next-up line takes ${m.rows} lines`);
    expect(m.title?.includes("TituloSemEspacos"), `${width}x${height}: the next-up line has no title attribute`);
    await p.close();
  }

  const scale = await home(fullHome, { width: 1920, height: 1080 });
  const big = await scale.evaluate(() => getComputedStyle(document.querySelector(".kicker")).fontSize);
  expect(big === "13px", `large window: kicker is ${big}`);
  await scale.close();

  for (const width of [1100, 1099, 900, 720]) {
    const p = await home(fullHome, { width, height: 700 });
    const x = await p.evaluate(() => {
      const c = document.querySelector("main.content");
      return c.scrollWidth - c.clientWidth;
    });
    expect(x <= 0, `${width}px: overflows sideways by ${x}px`);
    await p.close();
  }
});

group("post", async () => {
  const p = await home(typicalHome);
  expect((await text(p, "h1")) === "Bom dia.", `greeting: ${await text(p, "h1")}`);
  expect((await text(p, ".kicker")).trim() === "Terça-feira, 3 de novembro", `date: ${await text(p, ".kicker")}`);
  expect((await text(p, ".dek")).startsWith("Dois posts para publicar"), `summary: ${await text(p, ".dek")}`);
  expect(await p.locator("text=Próximo: 09:00 no LinkedIn.").count(), "the Brief has no next-up line");

  const post = w("post");
  for (const name of ["Copiar texto", "Abrir LinkedIn", "Anexos", "Marcar como publicado"]) {
    expect((await p.locator(`${post} button:has-text("${name}")`).count()) === 1, `next post: no '${name}' button`);
  }
  expect((await p.locator(`${post} .w-row`).count()) === 1, "the later post is not a single line");
  expect((await text(p, `${post} .w-row`)).includes("Carrossel"), "the later post is missing");
  expect((await text(p, more("post"))).includes("Ver mais"), "no Ver mais in Postar hoje");
  expect(!(await text(p, more("post"))).includes("+"), "Postar hoje counts a hidden item that fits");

  await p.click(`${post} button:has-text("Copiar texto")`);
  expect((await calls(p, "copy_post_text")).length === 1, "Copiar texto does not reach the backend");
  await p.click(`${post} button:has-text("Marcar como publicado")`);
  await p.click("dialog[open] button[type=submit]");
  await p.waitForTimeout(100);
  expect((await calls(p, "mark_published")).length === 1, "Marcar como publicado does not reach the backend");

  await p.click(more("post"));
  await p.waitForSelector(".today");
  expect((await onView(p, "Calendário")) && (await onTodayTab(p)), "Ver mais in Postar hoje does not open Hoje");
  await p.close();

  // The publish dialog belongs to one post: the next lead must not inherit the link, or the id.
  const nextLead = await home(typicalHome);
  const lead2 = "01JBX3K9Q3CD";
  const publishLead = async (pg, url) => {
    await pg.click(`${post} button:has-text("Marcar como publicado")`);
    if (url) await pg.fill("dialog[open] input[type=url]", url);
    await pg.click("dialog[open] button[type=submit]");
    await pg.waitForTimeout(100);
  };
  const leadPublished = (pg) =>
    pg.evaluate(async () => {
      const t = await window.__TAURI_INTERNALS__.invoke("get_today", {});
      window.__today = { ...t, posts: t.posts.map((x, i) => (i === 0 ? { ...x, done: true, status: "published" } : x)) };
      window.__emit("pauta://changed", null);
    });
  await publishLead(nextLead, "https://www.linkedin.com/posts/primeiro");
  await leadPublished(nextLead);
  await nextLead.waitForFunction(() => document.querySelector('[data-widget="post"] .lead .title')?.textContent.includes("Carrossel"));
  await nextLead.click(`${post} button:has-text("Marcar como publicado")`);
  expect((await nextLead.inputValue("dialog[open] input[type=url]")) === "", "the next post's dialog opens with the previous post's link");
  await nextLead.click("dialog[open] button[type=submit]");
  await nextLead.waitForTimeout(100);
  const sent = (await calls(nextLead, "mark_published")).map((c) => c.args);
  expect(sent.length === 2 && sent[1].id === lead2 && !sent[1].url, `the second publish sent ${JSON.stringify(sent[1])}`);
  await nextLead.close();

  // The lead changes while the dialog is open (MCP, an external edit): the dialog goes with it.
  const swapped = await home(typicalHome);
  await swapped.click(`${post} button:has-text("Marcar como publicado")`);
  await leadPublished(swapped);
  await swapped.waitForTimeout(200);
  expect(!(await swapped.locator("dialog[open]").count()), "a dialog opened for one post stayed open for the next");
  await swapped.close();

  const full = await home(fullHome);
  expect((await full.locator(`${post} .w-row`).count()) === 2, "Postar hoje ignores its limit");
  expect((await text(full, more("post"))).includes("+2 · Ver mais"), `Postar hoje: ${await text(full, more("post"))}`);
  expect((await text(full, post)).includes("falta imagem · sem texto"), "the next post's problems are not on one short line");
  await full.close();

  const late = await home(lateHome);
  expect(await late.locator(`${post} >> text=atrasado`).count(), "a late next post has no 'atrasado' tag");
  await late.close();

  const none = await home(noPostsHome);
  expect(await none.locator(`${post} >> text=Nada para postar hoje.`).count(), "no empty message in Postar hoje");
  expect(await none.locator(`${post} >> text=Próximo post:`).count(), "an empty day does not show the next post");
  await none.close();

  const done = await home(allPostedHome);
  expect(await done.locator(`${post} >> text=Tudo publicado hoje.`).count(), "no 'Tudo publicado hoje.'");
  expect(((await done.locator(`${post} .w-row .sr-only`).first().textContent()) ?? "").includes("Publicado"), "a published row says it only with a check mark");
  await done.close();

  const night = await home(emptyHome);
  expect((await text(night, "h1")) === "Boa noite.", `greeting at 20:00: ${await text(night, "h1")}`);
  expect(!(await night.locator("text=/^Próximo:/").count()), "a next-up line with nothing next");
  await night.close();
});

group("warnings", async () => {
  const box = w("warnings");
  const p = await home(typicalHome);
  const rows = await p.locator(`${box} .w-row`).allTextContents();
  expect(rows.length === 4, `avisos: ${rows.length} rows`);
  expect(rows[0]?.includes("Revisar capítulo 3 do TCC2") && rows[0]?.includes("ontem"), `first aviso: ${rows[0]}`);
  expect(rows[1]?.includes("Post de amanhã: falta imagem"), `second aviso: ${rows[1]}`);
  expect(rows[2]?.includes("Instagram: nada agendado em 7 dias"), `third aviso: ${rows[2]}`);
  expect(rows[3]?.includes("2 cartões parados"), `fourth aviso: ${rows[3]}`);
  await p.click(`${box} .w-row >> nth=0`);
  await p.waitForSelector("aside.editor", { timeout: 3000 }).catch(() => problems.push("an overdue aviso does not open the card"));
  await p.keyboard.press("Escape");
  await p.waitForTimeout(100);
  await p.click(`${box} .w-row >> nth=2`);
  await p.waitForSelector(".today", { timeout: 3000 }).catch(() => problems.push("the empty queue aviso does not open Hoje"));
  await p.close();

  const full = await home(fullHome);
  const fullRows = await full.locator(`${box} .w-row`).allTextContents();
  expect(fullRows.length === 5, `avisos ignore the limit: ${fullRows.length}`);
  expect(fullRows[0]?.includes("2 arquivos com erro"), `invalid files come first: ${fullRows[0]}`);
  expect((await text(full, more("warnings"))).includes("+10 · Ver mais"), `avisos: ${await text(full, more("warnings"))}`);
  await full.click(more("warnings"));
  await full.waitForSelector(".today");
  await full.close();

  const late = await home(lateHome);
  expect(!(await late.locator(`${box} >> text=Por que Rust`).count()), "a late post of today is shown twice");
  await late.close();

  const calm = await home(emptyHome);
  expect(await calm.locator(`${box} >> text=Tudo em dia.`).count(), "no 'Tudo em dia.'");
  await calm.close();
});

group("agenda", async () => {
  const box = w("agenda");
  const p = await home(typicalHome);
  const body = await text(p, box);
  for (const day of ["Hoje", "Amanhã", "Qui, 5 nov", "Seg, 9 nov"]) expect(body.includes(day), `agenda: no group '${day}'`);
  expect(!body.includes("Por que Rust") && !body.includes("Carrossel"), "agenda repeats today's posts");
  expect(!body.includes("Prazo já cumprido"), "agenda shows a finished item");
  expect(body.includes("Semana de provas"), "agenda lost an event that began earlier");
  const first = await text(p, `${box} .w-row`);
  expect(first.includes("Semana de provas"), `the event that began earlier is not under Hoje: ${first}`);
  await p.click(`${box} .w-row:has-text("Bastidores do TCC")`);
  await p.waitForSelector("aside.editor", { timeout: 3000 }).catch(() => problems.push("a post in the agenda does not open its card"));
  await p.keyboard.press("Escape");
  await p.waitForTimeout(100);
  await p.click(more("agenda"));
  await p.waitForSelector(".agenda");
  expect((await onView(p, "Calendário")) && (await p.locator("button[aria-pressed='true']:text-is('Agenda')").count()), "Ver mais in Agenda does not open the agenda view");
  await p.keyboard.press("Alt+1");
  await p.waitForSelector(box);
  const kinds = await p.locator(`${box} .w-row .sr-only`).allTextContents();
  expect(["Evento:", "Prazo:", "LinkedIn:"].every((k) => kinds.includes(k)), `agenda: no kind for screen readers: ${kinds}`);
  await p.close();

  const full = await home(fullHome);
  expect((await full.locator(`${box} .w-row`).count()) === 12, "agenda ignores its limit");
  expect((await text(full, more("agenda"))).includes("+18 · Ver mais"), `agenda: ${await text(full, more("agenda"))}`);
  await full.close();

  const roomy = await home(fullHome, { width: 1920, height: 1080 });
  expect((await roomy.locator(`${box} .w-row`).count()) === 14, "agenda does not use the room of a large window");
  await roomy.close();

  const none = await home(emptyHome);
  expect(await none.locator(`${box} >> text=Nada nos próximos 7 dias.`).count(), "no empty message in Agenda");
  await none.close();

  // Until the agenda arrives, and when it never does, Início does not claim the week is empty.
  const slow = await home(layered(typicalHome, `if (cmd === "get_agenda" && window.__slowAgenda) return new Promise((res) => setTimeout(() => res(r), 500))`));
  await slow.keyboard.press("Alt+2");
  await slow.evaluate(() => (window.__slowAgenda = true));
  await slow.keyboard.press("Alt+1");
  await slow.waitForSelector(box);
  await slow.waitForTimeout(100);
  expect(!(await text(slow, box)).includes("Nada nos próximos"), "the agenda says it is empty while it is on its way");
  await slow.waitForFunction((s) => document.querySelector(s).textContent.includes("Bastidores"), box, { timeout: 3000 });
  await slow.close();

  const failed = await home(layered(typicalHome, `if (cmd === "get_agenda") return Promise.reject("falha simulada")`));
  expect(!(await text(failed, box)).includes("Nada nos próximos"), "a failed fetch makes the agenda say it is empty");
  expect(await failed.locator(".toast.error").count(), "a failed agenda shows no error");
  await failed.close();
});

group("rhythm", async () => {
  const box = w("rhythm");
  const p = await home(typicalHome);
  expect((await p.locator(`${box} [role=img]`).count()) === 2, "Ritmo: one meter per network");
  expect((await text(p, box)).includes("1 de 2 nesta semana"), `Ritmo: ${await text(p, box)}`);
  await p.click(more("rhythm"));
  await p.waitForSelector("text=Posts por semana");
  expect(await onView(p, "Ritmo"), "Ver mais in Ritmo does not open Ritmo");
  await p.close();

  // Three networks do not fit a normal window.
  const full = await home(fullHome);
  expect((await text(full, more("rhythm"))).includes("+1 · Ver mais"), `Ritmo: ${await text(full, more("rhythm"))}`);
  await full.close();

  // However large the goal, the dots are one line: at most six, then the ellipsis. Where the text does
  // not fit beside them (a 1100px window has columns of 250px) it drops below, and nothing leaves the card.
  for (const [width, height] of [[1280, 820], [1100, 700], [1920, 1080]]) {
    const pg = await home(fullHome, { width, height });
    const rows = await pg.evaluate(() =>
      [...document.querySelectorAll('[data-widget="rhythm"] .meter')].map((m) => {
        const mid = (e) => Math.round(e.getBoundingClientRect().top + e.getBoundingClientRect().height / 2);
        const dots = [...m.querySelectorAll(".dot")];
        const parts = [...m.querySelectorAll(".net, .dots span:not(.dot), .of"), ...dots];
        const widget = m.closest("[data-widget]").getBoundingClientRect();
        return {
          dots: dots.length,
          lines: new Set(parts.map(mid)).size,
          dotLines: new Set([...dots, ...m.querySelectorAll(".dots span:not(.dot)")].map(mid)).size,
          squashed: dots.some((d) => Math.abs(d.getBoundingClientRect().width - d.getBoundingClientRect().height) > 0.5),
          spill: parts.some((e) => e.getBoundingClientRect().right > widget.right),
        };
      }),
    );
    expect(rows.length >= 2, `${width}x${height}: Ritmo shows ${rows.length} rows`);
    expect(rows[0].dots === 6, `${width}x${height}: a goal of 12 shows ${rows[0].dots} dots`);
    for (const [i, r] of rows.entries()) {
      expect(r.dotLines === 1, `${width}x${height}: Ritmo row ${i}: the dots break into ${r.dotLines} lines`);
      expect(width < 1280 || r.lines === 1, `${width}x${height}: Ritmo row ${i} breaks into ${r.lines} lines`);
      expect(!r.squashed, `${width}x${height}: Ritmo row ${i} squashes its dots`);
      expect(!r.spill, `${width}x${height}: Ritmo row ${i} leaves the card`);
    }
    await pg.close();
  }

  const none = await home(emptyHome);
  expect(await none.locator(`${box} >> text=Nenhuma meta definida.`).count(), "no empty message in Ritmo");
  await none.close();
});

group("ideas", async () => {
  const box = w("ideas");
  const field = `${box} input`;
  const p = await home(typicalHome);
  const rows = await p.locator(`${box} .w-row`).allTextContents();
  expect(rows.length === 3 && rows[0]?.includes("Thread sobre ownership"), `ideias: ${rows}`);
  expect((await text(p, more("ideas"))).includes("+1 · Ver mais"), `ideias: ${await text(p, more("ideas"))}`);

  await p.fill(field, "   ");
  await p.press(field, "Enter");
  expect(!(await calls(p, "create_card")).length, "a blank idea was saved");

  await p.fill(field, "Uma ideia nova");
  await p.press(field, "Enter");
  await p.waitForFunction((s) => document.querySelector(s).value === "", field);
  const made = await calls(p, "create_card");
  expect(made.length === 1 && made[0].args.board === "conteudo" && made[0].args.card.title === "Uma ideia nova" && made[0].args.card.top === true, `create_card: ${JSON.stringify(made)}`);
  expect(await p.locator("text=Ideia guardada.").count(), "no 'Ideia guardada.' message");

  // Enter twice while the first save is on its way.
  await p.evaluate(() => {
    window.__createCard = () => new Promise((r) => setTimeout(() => r({ id: "n" }), 300));
  });
  await p.fill(field, "Duas vezes");
  await p.press(field, "Enter");
  await p.press(field, "Enter");
  await p.waitForTimeout(450);
  expect((await calls(p, "create_card")).length === 2, "a double Enter saved the idea twice");

  await p.evaluate(() => {
    window.__createCard = () => Promise.reject("falha simulada");
  });
  await p.fill(field, "Não posso perder");
  await p.press(field, "Enter");
  await p.waitForSelector(".toast.error", { timeout: 3000 }).catch(() => problems.push("a failed save shows no error"));
  expect((await p.inputValue(field)) === "Não posso perder", "a failed save lost the text");

  await p.click(`${box} button[aria-label="Adicionar ideia"]`);
  await p.click(more("ideas"));
  await p.waitForSelector("[data-id]");
  expect((await onView(p, "Quadros")) && (await p.locator('[role=tab][aria-selected="true"]:has-text("Conteúdo")').count()), "Ver mais in Ideias does not open the content board");
  await p.close();

  const none = await home(emptyHome);
  expect(await none.locator(`${box} >> text=Nenhuma ideia guardada.`).count(), "no empty message in Ideias");
  await none.close();

  const noBoard = await home(noContentBoardHome);
  expect(await noBoard.locator(`${box} >> text=Crie um quadro de conteúdo`).count(), "no message without a content board");
  expect(!(await noBoard.locator(field).count()), "the idea field stays without a content board");
  await noBoard.close();
});

group("projects", async () => {
  const box = w("projects");
  const p = await home(typicalHome);
  const body = await text(p, box);
  for (const line of [
    "TCC2",
    "atualizado hoje",
    "Movido para Revisão: Revisar capítulo 3",
    "Entrega parcial do TCC2 · amanhã",
    "Dungeon Meister",
    "atualizado ontem",
    "Editado: Mapa da masmorra",
    "Concluído: Redesenho do caderno",
    "Tela Início",
    "Abstract-Tex",
    "atualizado em 28 out",
    "Criado: Ajustar o modelo do resumo",
  ]) {
    expect(body.includes(line), `projetos: no '${line}'`);
  }
  expect(body.indexOf("TCC2") < body.indexOf("Dungeon Meister"), "projetos: the last updated does not come first");
  expect(body.indexOf("Pauta") < body.indexOf("Abstract-Tex"), "projetos: order");
  await p.click(`${box} button:has-text("Entrega parcial do TCC2")`);
  await p.waitForSelector("aside.editor", { timeout: 3000 }).catch(() => problems.push("Próximo does not open the card"));
  await p.keyboard.press("Escape");
  await p.waitForTimeout(100);
  await p.click(`${box} button:text-is("Pauta")`);
  await p.waitForSelector('[role=tab][aria-selected="true"]:has-text("Pauta")', { timeout: 3000 }).catch(() => problems.push("the project name does not open its board"));
  await p.keyboard.press("Alt+1");
  await p.waitForSelector(box);
  await p.click(more("projects"));
  await p.waitForSelector("[role=tab]");
  expect(await onView(p, "Quadros"), "Ver mais in Projetos does not open Quadros");
  await p.close();

  const full = await home(fullHome);
  expect((await text(full, more("projects"))).includes("+2 · Ver mais"), `projetos: ${await text(full, more("projects"))}`);
  await full.close();

  const none = await home(emptyHome);
  expect(await none.locator(`${box} >> text=Nenhum cartão ainda`).count(), "no empty message in Projetos");
  await none.close();
});

group("return", async () => {
  const focus = async (p) => {
    await p.evaluate(() => window.dispatchEvent(new Event("focus")));
    await p.waitForTimeout(200);
  };
  const tick = async (p) => {
    await p.evaluate(() => window.__tick());
    await p.waitForTimeout(200);
  };

  const same = await home(typicalHome);
  await same.keyboard.press("Alt+2");
  await focus(same);
  expect(await onView(same, "Quadros"), "the same day must keep the current screen");
  await same.close();

  const fresh = await home(typicalHome);
  await fresh.keyboard.press("Alt+2");
  await nextDay(fresh);
  await focus(fresh);
  expect(await onView(fresh, "Início"), "a new day does not return to Início");
  await fresh.waitForFunction(() => window.__calls.some((c) => c.cmd === "get_agenda" && c.args.from === "2026-11-04"), null, { timeout: 3000 }).catch(() => problems.push("the agenda stays on yesterday after midnight"));
  await fresh.keyboard.press("Alt+2");
  await focus(fresh);
  expect(await onView(fresh, "Quadros"), "the second opening of the day must keep the screen");
  await fresh.close();

  // Início stays mounted across midnight: it must fetch the new week, not only a fresh mount.
  const mounted = await home(typicalHome);
  await nextDay(mounted);
  await mounted.evaluate(() => {
    window.__calls.length = 0;
    window.dispatchEvent(new Event("focus"));
  });
  await mounted.waitForTimeout(300);
  const from = await mounted.evaluate(() => window.__calls.filter((c) => c.cmd === "get_agenda").map((c) => c.args.from));
  expect(from.length > 0 && from.every((d) => d === "2026-11-04"), `mounted Início asked for the agenda from ${from}`);
  expect((await text(mounted, ".kicker")).includes("4 de novembro"), `the date did not change: ${await text(mounted, ".kicker")}`);
  await mounted.close();

  const busy = await home(typicalHome);
  await busy.keyboard.press("Alt+2");
  await busy.click('[data-id="01JBX3K9Q2AB"]');
  await busy.waitForSelector("aside.editor");
  await nextDay(busy);
  await focus(busy);
  expect(await onView(busy, "Quadros"), "a new day took the author out of the open editor");
  await busy.keyboard.press("Escape");
  await focus(busy);
  expect(await onView(busy, "Quadros"), "the day was not marked as seen while the editor was open");
  await busy.close();

  // Any open dialog counts, not only the four flags of the store: its text would be lost.
  const dialog = await home(typicalHome);
  await dialog.keyboard.press("Alt+2");
  await dialog.waitForSelector("[data-id]");
  await dialog.click('button:has-text("Novo quadro")');
  await dialog.waitForSelector("dialog[open]");
  await dialog.fill("dialog[open] input[type=text]", "Quadro quase pronto");
  await nextDay(dialog);
  await focus(dialog);
  expect(await onView(dialog, "Quadros"), "a new day left a dialog with typed text");
  expect(await dialog.locator("dialog[open]").count(), "a new day closed the dialog");
  expect((await dialog.inputValue("dialog[open] input[type=text]")) === "Quadro quase pronto", "the dialog lost its text");
  await dialog.close();

  // The publish dialog on the Hoje tab: the author goes to the browser to copy the link.
  const publish = await home(typicalHome);
  await publish.click(more("post"));
  await publish.waitForSelector(".today");
  await publish.click('.today button:has-text("Marcar como publicado") >> nth=0');
  await publish.waitForSelector("dialog[open]");
  await publish.fill("dialog[open] input[type=url]", "https://www.linkedin.com/posts/colado");
  await nextDay(publish);
  await focus(publish);
  expect((await publish.locator("dialog[open]").count()) === 1 && (await onView(publish, "Calendário")), "a new day took the author out of the publish dialog");
  await publish.close();

  // An explicit request (tray, notification) beats the return, in either order of arrival.
  for (const order of ["focus-then-navigate", "navigate-then-focus"]) {
    const p = await home(typicalHome);
    await p.keyboard.press("Alt+2");
    await nextDay(p);
    await p.evaluate((order) => {
      const f = () => window.dispatchEvent(new Event("focus"));
      const n = () => window.__emit("pauta://navigate", "today");
      if (order === "focus-then-navigate") {
        f();
        n();
      } else {
        n();
        f();
      }
    }, order);
    await p.waitForTimeout(300);
    expect((await onView(p, "Calendário")) && (await onTodayTab(p)), `${order}: the tray's Hoje lost to the return to Início`);
    // The day is seen, so the next opening stays where the author is.
    await p.keyboard.press("Alt+2");
    await focus(p);
    expect(await onView(p, "Quadros"), `${order}: the return came back later`);
    await p.close();
  }

  // The minute rule: whoever works across midnight is not sent home when switching windows later.
  const across = await home(withTick(typicalHome));
  await across.keyboard.press("Alt+2");
  await nextDay(across);
  await tick(across);
  await focus(across);
  expect(await onView(across, "Quadros"), "working across midnight, the author was sent home on the next focus");
  await across.close();

  // Without the focus the tick must not mark the day as seen: the next opening is the first one.
  const away = await home(withTick(typicalHome));
  await away.keyboard.press("Alt+2");
  await away.evaluate(() => (window.__focused = false));
  await nextDay(away);
  await tick(away);
  await away.evaluate(() => (window.__focused = true));
  await focus(away);
  expect(await onView(away, "Início"), "the tick of an unfocused window swallowed the return");
  await away.close();

  // The focus arrives while an unfocused tick is on its way.
  const racing = await home(withTick(typicalHome));
  await racing.keyboard.press("Alt+2");
  await racing.evaluate(() => (window.__focused = false));
  await nextDay(racing);
  await racing.evaluate(() => {
    window.__tick();
    window.__focused = true;
    window.dispatchEvent(new Event("focus"));
  });
  await racing.waitForTimeout(300);
  expect(await onView(racing, "Início"), "a tick that began unfocused swallowed the return");
  await racing.close();

  // The window can also become visible without a focus event.
  const visible = await home(typicalHome);
  await visible.keyboard.press("Alt+2");
  await nextDay(visible);
  await visible.evaluate(() => document.dispatchEvent(new Event("visibilitychange")));
  await visible.waitForTimeout(300);
  expect(await onView(visible, "Início"), "becoming visible on a new day does not return to Início");
  await visible.close();
});

const run = groups.filter(([name]) => !only || only.includes(name));
for (const [name, fn] of run) {
  try {
    await fn();
  } catch (e) {
    problems.push(`${name}: ${e.message.split("\n")[0]}`);
  }
}
await finish(`ok: ${run.map(([name]) => name).join(", ")}`);
