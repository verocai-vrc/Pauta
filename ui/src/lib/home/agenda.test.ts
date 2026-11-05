import { describe, expect, it } from "vitest";
import type { AgendaItem } from "../api";
import { groupByDay, homeAgenda } from "./agenda";

const DATE = "2026-11-05";

function item(kind: AgendaItem["kind"], id: string, date: string, extra: Partial<AgendaItem> = {}): AgendaItem {
  return { kind, id, board: "b", title: id, when: { date, time: null, at: null }, end: null, network: null, status: null, column: null, done: false, ...extra } as AgendaItem;
}

describe("homeAgenda", () => {
  it("drops finished items and the posts of today, and keeps deadlines of today", () => {
    const out = homeAgenda(
      [
        item("publication", "post-today", DATE),
        item("due", "due-today", DATE),
        item("due", "done", DATE, { done: true }),
        item("publication", "post-tomorrow", "2026-11-06"),
        item("event", "event", DATE),
      ],
      DATE,
    );
    expect(out.map((i) => i.id)).toEqual(["due-today", "post-tomorrow", "event"]);
  });

  it("keeps the order it receives", () => {
    const out = homeAgenda([item("due", "b", "2026-11-07"), item("due", "a", "2026-11-06")], DATE);
    expect(out.map((i) => i.id)).toEqual(["b", "a"]);
  });
});

describe("groupByDay", () => {
  it("labels the days and puts an event that began earlier under today", () => {
    const days = groupByDay(
      [
        item("event", "week", "2026-11-01", { end: { date: "2026-11-07", time: null, at: null } }),
        item("due", "a", DATE),
        item("due", "b", "2026-11-06"),
        item("event", "c", "2026-11-09"),
      ],
      DATE,
    );
    expect(days.map((d) => [d.date, d.label, d.today, d.items.map((i) => i.id)])).toEqual([
      [DATE, "Hoje", true, ["week", "a"]],
      ["2026-11-06", "Amanhã", false, ["b"]],
      ["2026-11-09", "Seg, 9 nov", false, ["c"]],
    ]);
  });

  it("capitalizes the weekday of other days", () => {
    const days = groupByDay([item("due", "a", "2026-11-05")], "2026-11-02");
    expect(days[0].label).toBe("Qui, 5 nov");
  });

  it("returns nothing for nothing", () => {
    expect(groupByDay([], DATE)).toEqual([]);
  });
});
