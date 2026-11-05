import { describe, expect, it } from "vitest";
import type { AgendaItem } from "../api";
import { greeting, isLate, nextScheduled, nextUp, nextUpText, splitPosts } from "./brief";

const net = (id: string | null) => (id === "linkedin" ? "LinkedIn" : (id ?? ""));

function item(kind: AgendaItem["kind"], id: string, title: string, date: string, time: string | null, o: Partial<AgendaItem> = {}): AgendaItem {
  return {
    kind,
    id,
    board: null,
    title,
    when: { date, time: time ? `${time}:00` : null, at: null },
    end: null,
    network: kind === "publication" ? "linkedin" : null,
    status: null,
    column: null,
    done: false,
    ...o,
  };
}

const postAt = (date: string, time: string, o: Partial<AgendaItem> = {}) => item("publication", `p-${date}-${time}`, "Post", date, time, o);
const dueOn = (date: string, title: string) => item("due", `d-${date}`, title, date, null);
const eventAt = (date: string, time: string, title: string) => item("event", `e-${date}`, title, date, time);

describe("greeting", () => {
  it("follows the hour of the day", () => {
    expect(greeting("04:59:00")).toBe("Boa noite.");
    expect(greeting("05:00:00")).toBe("Bom dia.");
    expect(greeting("11:59:00")).toBe("Bom dia.");
    expect(greeting("12:00:00")).toBe("Boa tarde.");
    expect(greeting("17:59:00")).toBe("Boa tarde.");
    expect(greeting("18:00:00")).toBe("Boa noite.");
    expect(greeting("00:00:00")).toBe("Boa noite.");
  });
});

describe("nextUpText", () => {
  it("says the time for a post of today", () => {
    expect(nextUpText(postAt("2026-11-03", "09:00"), "2026-11-03", net)).toBe("Próximo: 09:00 no LinkedIn.");
  });
  it("names the day for a later post", () => {
    expect(nextUpText(postAt("2026-11-04", "09:00"), "2026-11-03", net)).toBe("Próximo: amanhã, 09:00 no LinkedIn.");
  });
  it("names a deadline and its day", () => {
    expect(nextUpText(dueOn("2026-11-03", "Enviar"), "2026-11-03", net)).toBe("Próximo: Enviar, hoje.");
  });
  it("adds the time of an event", () => {
    expect(nextUpText(eventAt("2026-11-20", "14:00", "Banca"), "2026-11-03", net)).toBe("Próximo: Banca, sex, 20 nov às 14:00.");
  });
});

describe("nextUp", () => {
  const date = "2026-11-03";
  it("skips done items and times that already passed today", () => {
    const items = [postAt(date, "08:00"), postAt(date, "09:00", { done: true }), postAt(date, "11:00", { id: "later" })];
    expect(nextUp(items, date, "10:00:00")?.id).toBe("later");
  });
  it("keeps an item at the current minute", () => {
    expect(nextUp([postAt(date, "10:00", { id: "now" })], date, "10:00:00")?.id).toBe("now");
  });
  it("puts an all-day item after the timed ones of the same day", () => {
    const due = dueOn(date, "Enviar");
    const ev = eventAt(date, "23:00", "Banca");
    expect(nextUp([due, ev], date, "10:00:00")).toBe(ev);
  });
  it("keeps an all-day item that is today even late at night", () => {
    const due = dueOn(date, "Enviar");
    expect(nextUp([due], date, "23:30:00")).toBe(due);
  });
  it("orders by date first, whatever the input order", () => {
    const a = postAt("2026-11-05", "09:00", { id: "a" });
    const b = postAt("2026-11-04", "18:00", { id: "b" });
    expect(nextUp([a, b], date, "10:00:00")?.id).toBe("b");
  });
  it("returns null with nothing ahead", () => {
    expect(nextUp([], date, "10:00:00")).toBeNull();
    expect(nextUp([postAt(date, "08:00")], date, "10:00:00")).toBeNull();
  });
});

describe("splitPosts", () => {
  it("leads with the first pending post and puts published ones last", () => {
    const a = postAt("2026-11-03", "07:00", { id: "a", done: true });
    const b = postAt("2026-11-03", "09:00", { id: "b" });
    const c = postAt("2026-11-03", "18:00", { id: "c" });
    const { lead, rest } = splitPosts([a, b, c]);
    expect(lead?.id).toBe("b");
    expect(rest.map((p) => p.id)).toEqual(["c", "a"]);
  });
  it("has no lead when everything is published", () => {
    const a = postAt("2026-11-03", "07:00", { id: "a", done: true });
    expect(splitPosts([a])).toEqual({ lead: null, rest: [a] });
    expect(splitPosts([])).toEqual({ lead: null, rest: [] });
  });
});

describe("isLate", () => {
  const date = "2026-11-03";
  it("is true once the time has passed", () => {
    expect(isLate(postAt(date, "09:00"), date, "10:00:00")).toBe(true);
    expect(isLate(postAt(date, "09:00"), date, "08:30:00")).toBe(false);
  });
  it("is never true for a published post", () => {
    expect(isLate(postAt(date, "09:00", { done: true }), date, "10:00:00")).toBe(false);
  });
});

describe("nextScheduled", () => {
  const date = "2026-11-03";
  it("finds the first pending publication after today", () => {
    const items = [dueOn("2026-11-04", "Enviar"), postAt("2026-11-06", "09:00", { id: "x", done: true }), postAt("2026-11-07", "09:00", { id: "y" }), postAt("2026-11-05", "09:00", { id: "z" })];
    expect(nextScheduled(items, date)?.id).toBe("z");
  });
  it("returns null without one", () => {
    expect(nextScheduled([dueOn("2026-11-04", "Enviar")], date)).toBeNull();
  });
});
