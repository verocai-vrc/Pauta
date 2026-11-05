// Shared setup for the browser checks: the built UI served locally, headless
// Chromium, and a mocked Tauri bridge fed with real backend output (make-fixtures.sh).
// A test can intercept any command with `window.__override(cmd, args)`, set from an
// init script; return `undefined` to fall through to the fixtures.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { chromium } from "playwright-core";

const require = createRequire(import.meta.url);
const root = path.resolve(import.meta.dirname, "../..");
const dist = process.env.PAUTA_DIST ?? path.join(root, "ui/dist");
export const shots = process.env.SHOTS;
if (shots) fs.mkdirSync(shots, { recursive: true });

// ---- fixtures from the real backend ------------------------------------------
const fx = fs.mkdtempSync(path.join(os.tmpdir(), "pauta-fx-"));
execFileSync(path.join(import.meta.dirname, "make-fixtures.sh"), [fx], { stdio: "inherit" });
const read = (f) => JSON.parse(fs.readFileSync(path.join(fx, f), "utf8"));
const data = { index: read("index.json"), today: read("today.json"), rhythm: read("rhythm.json"), agenda: read("agenda.json") };

// ---- static server ------------------------------------------------------------
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".svg": "image/svg+xml" };
const server = http.createServer((req, res) => {
  const p = path.join(dist, decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = fs.existsSync(p) && fs.statSync(p).isFile() ? p : path.join(dist, "index.html");
  res.setHeader("content-type", types[path.extname(file)] ?? "application/octet-stream");
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, r));
const url = `http://localhost:${server.address().port}/`;

// ---- browser ------------------------------------------------------------------
function chromiumPath() {
  if (process.env.CHROMIUM_PATH) return process.env.CHROMIUM_PATH;
  try {
    const p = chromium.executablePath();
    if (fs.existsSync(p)) return p;
  } catch {
    // No managed browser installed: fall through to the cache lookup below.
  }
  const cache = path.join(os.homedir(), ".cache/ms-playwright");
  for (const d of fs.existsSync(cache) ? fs.readdirSync(cache).sort().reverse() : []) {
    if (!d.startsWith("chromium_headless_shell")) continue;
    const bin = path.join(cache, d, "chrome-headless-shell-linux64/chrome-headless-shell");
    if (fs.existsSync(bin)) return bin;
  }
  throw new Error("No Chromium found. Run `pnpm exec playwright-core install chromium` or set CHROMIUM_PATH.");
}

export const browser = await chromium.launch({ executablePath: chromiumPath() });
const axeSource = fs.readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");
export const problems = [];

export async function newPage(scheme = "light", { init } = {}) {
  const page = await browser.newPage({ viewport: { width: 1360, height: 900 }, colorScheme: scheme });
  page.on("pageerror", (e) => problems.push(`[${scheme}] page error: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && problems.push(`[${scheme}] console: ${m.text()}`));
  if (init) await page.addInitScript(init);
  await page.addInitScript((data) => {
    let cb = 0;
    window.__TAURI_INTERNALS__ = {
      transformCallback: () => ++cb,
      unregisterCallback: () => {},
      invoke: async (cmd, args) => {
        const over = window.__override?.(cmd, args);
        if (over !== undefined) return over;
        switch (cmd) {
          case "app_status": return { root: "/home/arthur/Documentos/Pauta", suggested: "", error: null };
          case "get_index": return data.index;
          case "get_today": return data.today;
          case "get_rhythm": return data.rhythm;
          case "get_agenda": return data.agenda;
          case "get_preferences": return { close_to_tray: true, autostart: true, theme: "system" };
          case "mcp_snippet": return { binary: "/usr/bin/pauta-mcp", binary_exists: true, claude_code: "claude mcp add pauta -- pauta-mcp", claude_desktop: "{}" };
          case "list_assets": return [];
          case "find_slots": return [{ local: "2026-10-31T19:00", date: "2026-10-31", time: "19:00" }];
          case "list_trash": case "list_archive": return [];
          default: return null;
        }
      },
    };
  }, data);
  await page.goto(url);
  await page.waitForSelector("nav");
  return page;
}

const fonts = ["Fraunces Variable", "Source Sans 3 Variable"];

export async function audit(page, name, scheme) {
  const loaded = await page.evaluate(async () => {
    await document.fonts.ready;
    return [...document.fonts].filter((f) => f.status === "loaded").map((f) => f.family);
  });
  for (const f of fonts) {
    if (!loaded.some((l) => l.includes(f))) problems.push(`[${scheme}] ${name}: font not loaded: ${f}`);
  }

  // axe cannot measure contrast over a background image and would skip that text without saying
  // so. The ruling is only hairlines, so the audit measures the text against the plain paper.
  const noRuling = await page.addStyleTag({ content: ":root { --ruling: none !important; }" });
  await page.addScriptTag({ content: axeSource });
  const result = await page.evaluate(() =>
    window.axe.run(document, { runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"] } }),
  );
  for (const v of result.violations) {
    for (const node of v.nodes) {
      const d = node.any?.[0]?.data;
      const detail = v.id === "color-contrast" && d ? ` [${d.fgColor} on ${d.bgColor}, ${d.contrastRatio}:1, needs ${d.expectedContrastRatio}:1]` : "";
      problems.push(`[${scheme}] ${name}: ${v.id} (${v.impact}) ${node.target.join(" ")} :: ${v.help}${detail}`);
    }
  }
  await noRuling.evaluate((el) => el.remove());
  if (shots) await page.screenshot({ path: path.join(shots, `${scheme}-${name}.png`) });
}


export async function finish(okMessage) {
  await browser.close();
  server.close();
  fs.rmSync(fx, { recursive: true, force: true });
  if (problems.length) {
    console.error(`\n${problems.length} problem(s):\n` + problems.map((p) => " - " + p).join("\n"));
    process.exit(1);
  }
  console.log(okMessage);
}
