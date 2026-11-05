import { describe, expect, it } from "vitest";
import type { Board, Card, Column, Index, PautaEvent, When } from "../api";
import { localDay, projectSummaries, updatedLabel } from "./projects";

const TODAY = "2026-11-03";

const columns: Column[] = [
  { id: "inbox", name: "A fazer", role: "inbox", limit: null },
  { id: "doing", name: "Fazendo", role: null, limit: null },
  { id: "review", name: "Revisão", role: null, limit: null },
  { id: "done", name: "Pronto", role: "done", limit: null },
];

const board = (id: string, o: Partial<Board> = {}): Board => ({ id, name: id, kind: "project", color: null, columns, dir: "", hash: "", ...o });
const when = (date: string, time: string | null = null): When => ({ date, time: time && `${time}:00`, at: null });
const card = (id: string, o: Partial<Card>): Card =>
  ({
    id, board: "p", title: id, kind: "task", column: "inbox", rank: "m", tags: [], network: null, publication: null, due: null,
    reminders: null, links: { board: null, siblings: [], article: null }, created: null, updated: null, metrics: null, art: null,
    assets: [], column_since: null, body: "", checklist: { done: 0, total: 0 }, path: "", hash: "", ...o,
  }) as Card;
const event = (id: string, date: string, o: Partial<PautaEvent> = {}): PautaEvent =>
  ({ id, title: id, kind: "event", when: when(date), end: null, reminders: null, links: { board: "p", siblings: [], article: null }, tags: [], body: "", path: "", hash: "", ...o }) as PautaEvent;
const index = (boards: Board[], cards: Card[], events: PautaEvent[] = []): Index =>
  ({ config: { timezone: "America/Recife" }, boards, cards, events, invalid: [], readiness: {} }) as unknown as Index;
const one = (cards: Card[], events: PautaEvent[] = []) => projectSummaries(index([board("p")], cards, events), TODAY)[0];

describe("localDay", () => {
  it("uses the day in the configured time zone", () => {
    expect(localDay("2026-11-04T01:30:00Z", "America/Recife")).toBe("2026-11-03");
    expect(localDay("2026-11-04T01:30:00Z", "UTC")).toBe("2026-11-04");
  });
});

describe("updatedLabel", () => {
  it("names today, yesterday and older days", () => {
    expect(updatedLabel("2026-11-03", TODAY)).toBe("atualizado hoje");
    expect(updatedLabel("2026-11-02", TODAY)).toBe("atualizado ontem");
    expect(updatedLabel("2026-10-28", TODAY)).toBe("atualizado em 28 out");
  });
});

describe("last", () => {
  const at = "2026-11-02T12:00:00Z";
  it("created equal to updated reads as created", () => {
    expect(one([card("a", { created: at, updated: at })]).last).toEqual({ text: "Criado: a", kind: "card", id: "a" });
  });
  it("moved into a done column reads as done", () => {
    const c = card("a", { column: "done", created: "2026-10-01T00:00:00Z", updated: at, column_since: at });
    expect(one([c]).last?.text).toBe("Concluído: a");
  });
  it("moved elsewhere names the column", () => {
    const c = card("a", { column: "review", created: "2026-10-01T00:00:00Z", updated: at, column_since: at });
    expect(one([c]).last?.text).toBe("Movido para Revisão: a");
  });
  it("anything else is an edit", () => {
    const c = card("a", { column: "doing", created: "2026-10-01T00:00:00Z", updated: at, column_since: "2026-10-20T00:00:00Z" });
    expect(one([c]).last?.text).toBe("Editado: a");
  });
  it("created wins over moved when all three are equal", () => {
    expect(one([card("a", { created: at, updated: at, column_since: at })]).last?.text).toBe("Criado: a");
  });
  it("compares instants, not strings", () => {
    // Same instant written differently.
    expect(one([card("a", { created: "2026-11-02T12:00:00Z", updated: "2026-11-02T12:00:00.000+00:00" })]).last?.text).toBe("Criado: a");
  });
  it("the most recent card wins", () => {
    const s = one([card("old", { updated: "2026-10-01T00:00:00Z" }), card("new", { updated: "2026-11-01T00:00:00Z" })]);
    expect(s.last?.id).toBe("new");
  });
  it("a moment is the greatest of the three", () => {
    const s = one([card("a", { created: "2026-10-01T00:00:00Z", updated: "2026-10-02T00:00:00Z" }), card("b", { column_since: "2026-10-05T00:00:00Z" })]);
    expect(s.last?.id).toBe("b");
  });
  it("the day follows the configured time zone", () => {
    expect(one([card("a", { updated: "2026-11-04T01:30:00Z" })]).updated).toBe("2026-11-03");
  });
});

describe("next", () => {
  it("the nearest dated item wins, an overdue one before a future one", () => {
    const s = one([card("future", { due: when("2026-11-10") }), card("late", { due: when("2026-11-01") })]);
    expect(s.next).toEqual({ text: "late · dom, 1 nov", kind: "card", id: "late" });
  });
  it("names the day relative to today", () => {
    expect(one([card("a", { due: when("2026-11-04") })]).next?.text).toBe("a · amanhã");
  });
  it("counts an event of the board from today on", () => {
    const s = one([card("far", { due: when("2026-11-20") })], [event("e", "2026-11-05")]);
    expect(s.next).toEqual({ text: "e · qui, 5 nov", kind: "event", id: "e" });
  });
  it("ignores past events and events of other boards", () => {
    const s = one([card("a", { column: "doing" })], [event("old", "2026-11-02"), event("other", "2026-11-04", { links: { board: "x", siblings: [], article: null } })]);
    expect(s.next?.id).toBe("a");
  });
  it("ignores the due date of a card that is done", () => {
    const s = one([card("a", { column: "done", due: when("2026-11-04") }), card("b", { column: "doing" })]);
    expect(s.next?.id).toBe("b");
  });
  it("same date: timed before all-day, then card before event", () => {
    const timed = one([card("c", { due: when("2026-11-05") })], [event("e", "2026-11-05", { when: when("2026-11-05", "09:00") })]);
    expect(timed.next?.id).toBe("e");
    const tie = one([card("c", { due: when("2026-11-05") })], [event("e", "2026-11-05")]);
    expect(tie.next?.id).toBe("c");
  });
  it("without dates, the most advanced column comes first, then to do", () => {
    const s = one([
      card("inbox", { column: "inbox" }),
      card("doing", { column: "doing" }),
      card("review", { column: "review" }),
      card("done", { column: "done" }),
    ]);
    expect(s.next).toEqual({ text: "review", kind: "card", id: "review" });
    expect(one([card("inbox", { column: "inbox" }), card("doing", { column: "doing" })]).next?.id).toBe("doing");
  });
  it("falls back to the inbox, by rank", () => {
    const s = one([card("b", { column: "inbox", rank: "z" }), card("a", { column: "inbox", rank: "c" }), card("d", { column: "done" })]);
    expect(s.next?.id).toBe("a");
  });
  it("within a column, by rank", () => {
    expect(one([card("b", { column: "doing", rank: "n" }), card("a", { column: "doing", rank: "d" })]).next?.id).toBe("a");
  });
  it("nothing to do when only done cards are left", () => {
    expect(one([card("a", { column: "done" })]).next).toBeNull();
  });
  it("a board without inbox or done columns does not break the rules", () => {
    const b = board("p", { columns: [{ id: "x", name: "X", role: null, limit: null }, { id: "y", name: "Y", role: null, limit: null }] });
    const s = projectSummaries(index([b], [card("a", { column: "x" }), card("b", { column: "y" })]), TODAY)[0];
    expect(s.next?.id).toBe("b");
    expect(s.last).toBeNull();
  });
});

describe("order and filtering", () => {
  it("leaves content boards out", () => {
    const i = index([board("c", { kind: "content" }), board("p")], [card("a", { board: "p" })]);
    expect(projectSummaries(i, TODAY).map((s) => s.id)).toEqual(["p"]);
  });
  it("sorts by the moment, empty boards last in board order", () => {
    const i = index(
      [board("empty1"), board("old"), board("empty2", { kind: "generic" }), board("new")],
      [card("o", { board: "old", updated: "2026-10-01T00:00:00Z" }), card("n", { board: "new", updated: "2026-11-01T00:00:00Z" })],
    );
    const s = projectSummaries(i, TODAY);
    expect(s.map((x) => x.id)).toEqual(["new", "old", "empty1", "empty2"]);
    expect(s.map((x) => x.empty)).toEqual([false, false, true, true]);
  });
  it("sorts by the instant, not by the day", () => {
    const i = index(
      [board("a"), board("b")],
      [card("x", { board: "a", updated: "2026-11-02T10:00:00Z" }), card("y", { board: "b", updated: "2026-11-02T20:00:00Z" })],
    );
    expect(projectSummaries(i, TODAY).map((s) => s.id)).toEqual(["b", "a"]);
  });
});

describe("cards without a moment", () => {
  it("never produce NaN or Invalid Date and keep next", () => {
    const s = one([card("a", { column: "doing" }), card("b", { created: "not a date", updated: "", column_since: null })]);
    expect(s.updated).toBeNull();
    expect(s.last).toBeNull();
    expect(s.empty).toBe(false);
    expect(s.next?.id).toBe("a");
    expect(JSON.stringify(s)).not.toMatch(/NaN|Invalid/);
  });
  it("ignores them when other cards have a moment", () => {
    const s = one([card("a", {}), card("b", { updated: "2026-11-01T00:00:00Z" })]);
    expect(s.last?.id).toBe("b");
  });
  it("sorts after boards with a moment and before empty ones", () => {
    const i = index([board("empty"), board("none"), board("dated")], [card("a", { board: "none" }), card("b", { board: "dated", updated: "2026-11-01T00:00:00Z" })]);
    expect(projectSummaries(i, TODAY).map((s) => s.id)).toEqual(["dated", "none", "empty"]);
  });
});
