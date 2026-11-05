import { describe, expect, it } from "vitest";
import type { AgendaItem, Today } from "../api";
import { buildWarnings } from "./warnings";

const DATE = "2026-11-05";

function item(kind: AgendaItem["kind"], id: string, title: string, date: string, extra: Partial<AgendaItem> = {}): AgendaItem {
  return { kind, id, board: "b", title, when: { date, time: null, at: null }, end: null, network: null, status: null, column: null, done: false, ...extra } as AgendaItem;
}

function today(extra: Partial<Today> = {}): Today {
  return { date: DATE, time: "08:00:00", posts: [], upcoming: [], overdue: [], empty_queues: [], stale: [], cadence: [], invalid_files: 0, ...extra };
}

const media = [{ code: "media_missing" as const, detail: null }];

describe("buildWarnings", () => {
  it("returns nothing when all is well", () => {
    expect(buildWarnings(today(), {}, [], null)).toEqual([]);
  });

  it("follows the order of section 5.3", () => {
    const lead = item("publication", "lead", "Lead", DATE, { network: "linkedin" });
    const next = item("publication", "next", "Next", "2026-11-06", { network: "linkedin" });
    const out = buildWarnings(
      today({
        invalid_files: 2,
        posts: [lead],
        overdue: [item("due", "late", "Revisar capítulo", "2026-11-04")],
        empty_queues: [{ network: "instagram", label: "Instagram", days: 7, candidates: [] }],
        stale: [
          { id: "s1", board: "b", title: "x", column: "draft", network: null },
          { id: "s2", board: "b", title: "y", column: "draft", network: null },
        ],
      }),
      { lead: media, next: media },
      [lead, next],
      "lead",
    );
    expect(out.map((w) => [w.tone, w.text, w.aux, w.card])).toEqual([
      ["danger", "2 arquivos com erro", null, null],
      ["danger", "Revisar capítulo", "ontem", "late"],
      ["warn", "Post de amanhã: falta imagem", null, "next"],
      ["warn", "Instagram: nada agendado em 7 dias", null, null],
      ["quiet", "2 cartões parados", null, null],
    ]);
  });

  it("keeps the singular forms", () => {
    const out = buildWarnings(today({ invalid_files: 1, stale: [{ id: "s", board: "b", title: "x", column: "draft", network: null }] }), {}, [], null);
    expect(out.map((w) => w.text)).toEqual(["1 arquivo com erro", "1 cartão parado"]);
  });

  it("hides the late post of today that Postar hoje already shows, but not a deadline of the same card", () => {
    const post = item("publication", "p", "Post", DATE, { network: "linkedin" });
    const deadline = item("due", "p", "Prazo do mesmo cartão", "2026-11-03");
    const out = buildWarnings(today({ posts: [post], overdue: [{ ...post, when: { date: DATE, time: "09:00:00", at: null } }, deadline] }), {}, [], "p");
    expect(out.map((w) => w.text)).toEqual(["Prazo do mesmo cartão"]);
  });

  it("shows a late post of today that is already done", () => {
    const done = item("publication", "p", "Já publicado", DATE, { done: true });
    const out = buildWarnings(today({ posts: [done], overdue: [done] }), {}, [], null);
    expect(out).toHaveLength(1);
  });

  it("includes problems of posts up to three days ahead, not four, and never the lead", () => {
    const agenda = [
      item("publication", "lead", "Lead", DATE),
      item("publication", "in", "In", "2026-11-08"),
      item("publication", "out", "Out", "2026-11-09"),
    ];
    const out = buildWarnings(today(), { lead: media, in: media, out: media }, agenda, "lead");
    expect(out.map((w) => w.card)).toEqual(["in"]);
    expect(out[0].text).toBe("Post de dom, 8 nov: falta imagem");
  });

  it("shows only the first problem of a post and skips posts that are done or ready", () => {
    const agenda = [
      item("publication", "a", "A", "2026-11-06"),
      item("publication", "b", "B", "2026-11-06", { done: true }),
      item("publication", "c", "C", "2026-11-06"),
    ];
    const out = buildWarnings(
      today(),
      { a: [{ code: "no_text", detail: null }, ...media], b: media },
      agenda,
      null,
    );
    expect(out.map((w) => w.text)).toEqual(["Post de amanhã: sem texto"]);
  });

  it("words the art problems by what is missing", () => {
    const agenda = [item("publication", "a", "A", "2026-11-06"), item("publication", "b", "B", "2026-11-06")];
    const out = buildWarnings(
      today(),
      { a: [{ code: "art_pending", detail: "files_present" }], b: [{ code: "art_pending", detail: null }] },
      agenda,
      null,
    );
    expect(out.map((w) => w.text)).toEqual(["Post de amanhã: marcar a arte como pronta", "Post de amanhã: arte a caminho"]);
  });

  it("ignores items of the agenda that are not posts", () => {
    const out = buildWarnings(today(), { d: media }, [item("due", "d", "D", "2026-11-06")], null);
    expect(out).toEqual([]);
  });
});
