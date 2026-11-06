// End-to-end test of the real app through tauri-driver (WebKitWebDriver), section 15.3.
//
// Needs: `cargo install tauri-driver --locked`, the `webkit2gtk-driver` package, a display
// (xvfb-run) and `pnpm tauri build --debug --no-bundle`. It has to be a debug build, because only
// those honor PAUTA_NOW, and not a plain `cargo build`, whose binary loads devUrl instead of the
// embedded UI and fails with "Callback was not called before the unload event".
// Run: `xvfb-run -a pnpm test:e2e`.
//
// Mouse drag-and-drop is not covered: WebKitWebDriver does not synthesize HTML5 drag events.
// The keyboard equivalent is, and drag-and-drop is covered by hand and in the browser harness.
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { Builder, By, Key, until } from "selenium-webdriver";

const repo = path.resolve(import.meta.dirname, "../..");
const app = path.join(repo, "target/debug/pauta");
if (!fs.existsSync(app)) throw new Error("Build the app first: pnpm tauri build --debug --no-bundle");

// A throwaway data folder and config dir: the test never touches the real ones.
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "pauta-e2e-"));
const data = path.join(tmp, "data");
fs.cpSync(path.join(repo, "examples/sample-data"), data, { recursive: true });
fs.rmSync(path.join(data, "boards/dungeon-meister/cards/cabecalho-quebrado--broken.md"));
// Eight o'clock on the day of the sample post (09:00, reminders -1h and 0): the "-1h" reminder is due.
const env = {
  ...process.env,
  XDG_CONFIG_HOME: path.join(tmp, "config"),
  PAUTA_ROOT: data,
  PAUTA_NOW: "2026-11-03T08:00:30-03:00",
};

const driverProc = spawn("tauri-driver", [], { stdio: "inherit", env });
await new Promise((r) => setTimeout(r, 1500));

const driver = await new Builder()
  .usingServer("http://127.0.0.1:4444/")
  .withCapabilities({ "tauri:options": { application: app }, browserName: "wry" })
  .build();

const failures = [];
const pageState = () =>
  driver
    .executeScript(() =>
      JSON.stringify({
        focus: document.activeElement?.tagName,
        selects: [...document.querySelectorAll("aside select")].map((s) => s.value),
        headings: [...document.querySelectorAll("aside h2, aside h3")].map((h) => h.textContent.trim()),
        slots: document.querySelectorAll("aside .chip-btn").length,
        toasts: [...document.querySelectorAll('[role="status"], [role="alert"]')].map((t) => t.textContent.trim()),
      }),
    )
    .catch((e) => `unavailable (${e.name})`);
async function step(name, fn) {
  try {
    await fn();
    console.log(`ok   ${name}`);
  } catch (e) {
    // WebKitWebDriver often sends an empty message, so the error name and page state are the clue.
    const why = `${e.name}: ${e.message.trim().split("\n")[0] || "(no message)"}`;
    failures.push(`${name}: ${why}`);
    console.error(`FAIL ${name}: ${why}`);
    console.error(`     page: ${await pageState()}`);
  }
}
const $ = (css) => driver.wait(until.elementLocated(By.css(css)), 10000);
const nav = (label) => driver.findElement(By.xpath(`//nav//button[contains(., "${label}")]`)).click();
const columnOf = (name) => By.xpath(`//section[contains(@class,"column")][.//h3[normalize-space()="${name}"]]`);

try {
  await step("starts on Início with the sample data", async () => {
    await $("nav");
    // 08:00:30 on the simulated clock, so the greeting is the morning one.
    await driver.wait(until.elementLocated(By.xpath('//h1[normalize-space()="Bom dia."]')), 10000);
    // Under the real CSP, which the browser checks do not have.
    const fonts = await driver.executeAsyncScript((done) =>
      document.fonts.ready.then(() => done([...document.fonts].filter((f) => f.status === "loaded").map((f) => f.family))),
    );
    for (const f of ["Fraunces Variable", "Source Sans 3 Variable"]) {
      if (!fonts.some((l) => l.includes(f))) throw new Error(`font not loaded: ${f}`);
    }
  });

  await step("creates a card from the quick-add box", async () => {
    await nav("Quadros");
    await $("[data-id]");
    await driver.findElement(By.css('button[aria-label="Adicionar cartão em Ideia"]')).click();
    const box = await $('input[aria-label="Título do cartão e Enter"]');
    await box.sendKeys("Cartão do teste E2E", Key.ENTER);
    await driver.wait(until.elementLocated(By.xpath('//*[@data-id][contains(., "Cartão do teste E2E")]')), 10000);
  });

  await step("moves it to the next column with Alt+Right", async () => {
    const tile = await driver.findElement(By.xpath('//*[@data-id][contains(., "Cartão do teste E2E")]'));
    await tile.click(); // opens the editor; close it to return to the board
    await driver.wait(until.elementLocated(By.css("aside.editor")), 10000);
    await driver.actions().sendKeys(Key.ESCAPE).perform();
    const t = await driver.findElement(By.xpath('//*[@data-id][contains(., "Cartão do teste E2E")]'));
    await driver.executeScript("arguments[0].focus()", t);
    await driver.actions().keyDown(Key.ALT).sendKeys(Key.ARROW_RIGHT).keyUp(Key.ALT).perform();
    await driver.wait(until.elementLocated(By.xpath(`${columnOf("Rascunho").value}//*[@data-id][contains(., "Cartão do teste E2E")]`)), 10000);
  });

  await step("schedules it on LinkedIn at the next free slot", async () => {
    await driver.findElement(By.xpath('//*[@data-id][contains(., "Cartão do teste E2E")]')).click();
    await $("aside.editor");
    const select = await driver.findElement(By.xpath('//aside//label[contains(., "Rede")]//select'));
    await select.sendKeys("LinkedIn");
    await driver.findElement(By.xpath('//aside//button[contains(., "Sugerir horário")]')).click();
    await driver.wait(until.elementLocated(By.css("aside .chip-btn")), 10000).then((b) => b.click());
    await driver.findElement(By.xpath('//aside//button[normalize-space()="Agendar"]')).click();
    await driver.wait(until.elementLocated(By.xpath('//*[@role="status"][contains(., "Publicação agendada")]')), 10000);
    await driver.actions().sendKeys(Key.ESCAPE).perform();
  });

  await step("shows it in the calendar (agenda view)", async () => {
    await nav("Calendário");
    await driver.findElement(By.xpath('//button[normalize-space()="Agenda"]')).click();
    await driver.wait(until.elementLocated(By.xpath('//*[contains(@class,"item")][contains(., "Cartão do teste E2E")]')), 10000);
  });

  await step("delivers the due reminder with the simulated clock", async () => {
    // The notifier ticks every 20 s; with no notification daemon the delivery is still recorded.
    const file = path.join(data, ".pauta/reminders.json");
    const deadline = Date.now() + 60000;
    let found = false;
    while (Date.now() < deadline && !found) {
      found = fs.existsSync(file) && fs.readFileSync(file, "utf8").includes("pub:01JBX3K9Q2AB");
      if (!found) await new Promise((r) => setTimeout(r, 2000));
    }
    if (!found) throw new Error("no delivery recorded in .pauta/reminders.json");
  });
} finally {
  await driver.quit().catch(() => {});
  driverProc.kill();
  fs.rmSync(tmp, { recursive: true, force: true });
}

if (failures.length) {
  console.error(`\n${failures.length} failure(s)`);
  process.exit(1);
}
console.log("e2e ok");
