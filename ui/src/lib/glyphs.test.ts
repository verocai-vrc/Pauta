import { describe, expect, it } from "vitest";

// Icons come from Icon.svelte. The arrows ← → ↑ ↓ are left out on purpose: they are still valid as
// text, as in "Alt+← → ↑ ↓" and the column mapping of the CSV import.
const GLYPHS = /[☀▦▤▥⚙＋⌕⌚⧉↗🗀🗑◀▶✕✓☑⚠⚑★▲◆🎨⇄↩⇩]/gu;

const sources = import.meta.glob<string>("../**/*.svelte", { query: "?raw", import: "default", eager: true });

describe("icons", () => {
  it("reads the components", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(20);
  });

  it("no component draws an icon with a Unicode glyph", () => {
    const found = Object.entries(sources).flatMap(([file, text]) =>
      (text.match(GLYPHS) ?? []).map((g) => `${file}: ${g}`),
    );
    expect(found).toEqual([]);
  });
});
