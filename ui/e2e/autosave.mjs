// Regression: text typed while an autosave is in flight must still be saved.
import { finish, newPage, problems } from "./harness.mjs";

const page = await newPage("light", {
  init: () => {
    window.__saves = [];
    window.__pending = [];
    window.__override = (cmd, args) => {
      if (cmd !== "update_card") return undefined;
      window.__saves.push(args.patch);
      // Held open until the test releases it, like a slow disk.
      return new Promise((resolve) => window.__pending.push(() => resolve({ ...args, id: args.id, hash: "h" + window.__saves.length })));
    };
  },
});
await page.click("nav >> text=Quadros");
await page.click('[data-id="01JBX3K9Q2AB"]');
const body = page.locator("textarea");
await body.waitFor();
await body.fill("primeira versão");
await page.waitForFunction(() => window.__saves.length === 1); // the autosave started and is still in flight
await body.fill("primeira versão e o resto digitado durante o salvamento");
await page.waitForTimeout(1000); // the debounce timer fires while the save is pending
if ((await page.evaluate(() => window.__saves.length)) !== 1) problems.push("a second save started while the first was in flight");
await page.evaluate(() => window.__pending.shift()());
await page.waitForFunction(() => window.__saves.length === 2, null, { timeout: 4000 }).catch(() => {});
const saves = await page.evaluate(() => window.__saves);
if (saves.length !== 2 || saves[1].body !== "primeira versão e o resto digitado durante o salvamento") {
  problems.push(`later edits were lost: ${JSON.stringify(saves.map((s) => s.body))}`);
}
await finish("ok: edits made during a save are saved afterwards");
