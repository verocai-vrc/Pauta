import { describe, expect, it } from "vitest";
import { addDays, addMonths, longDate, monthGrid, moveToDay, relativeDay, startOfWeek, weekday } from "./dates";
import type { AgendaItem, Today } from "./api";
import { todaySummary } from "./summary";
import { tn } from "./i18n";
import { ptBR } from "./locales/pt-BR";
import { charCount, postText, toggleTask } from "./markdown";

describe("dates", () => {
  it("weeks start on Monday", () => {
    expect(weekday("2026-11-02")).toBe(0); // Monday
    expect(weekday("2026-11-08")).toBe(6); // Sunday
    expect(startOfWeek("2026-11-05")).toBe("2026-11-02");
  });

  it("month grid covers the month in 6 weeks", () => {
    const g = monthGrid("2026-11-15");
    expect(g).toHaveLength(42);
    expect(g[0]).toBe("2026-10-26");
    expect(g).toContain("2026-11-30");
  });

  it("adds days and months across boundaries", () => {
    expect(addDays("2026-12-31", 1)).toBe("2027-01-01");
    expect(addMonths("2026-12-15", 1)).toBe("2027-01-01");
    expect(addMonths("2026-03-31", -1)).toBe("2026-02-01");
  });

  it("relative labels", () => {
    expect(relativeDay("2026-10-28", "2026-10-28")).toBe("hoje");
    expect(relativeDay("2026-10-29", "2026-10-28")).toBe("amanhã");
    expect(relativeDay("2026-11-04", "2026-10-28")).toBe("qua, 4 nov");
  });

  it("long date spells out the weekday and the month", () => {
    expect(longDate("2026-10-28")).toBe("Quarta-feira, 28 de outubro");
    expect(longDate("2026-11-01")).toBe("Domingo, 1º de novembro");
    expect(longDate("2026-11-11")).toBe("Quarta-feira, 11 de novembro");
  });

  it("rescheduling keeps the time of day", () => {
    expect(moveToDay({ date: "2026-11-03", time: "09:00:00", at: "x" }, "2026-11-05")).toBe("2026-11-05T09:00");
    expect(moveToDay({ date: "2026-12-05", time: null, at: null }, "2026-12-06")).toBe("2026-12-06");
  });
});

describe("today summary", () => {
  const item = (id: string, done = false) => ({ id, done }) as unknown as AgendaItem;
  const make = (posts: AgendaItem[], overdue: number, upcoming: number) =>
    ({
      posts,
      overdue: Array.from({ length: overdue }, (_, i) => item(`o${i}`)),
      upcoming: Array.from({ length: upcoming }, (_, i) => item(`u${i}`)),
    }) as unknown as Today;

  it("joins the three counts", () => {
    expect(todaySummary(make([item("a")], 1, 3))).toBe(
      "Um post para publicar, um item atrasado e três compromissos nos próximos 7 dias.",
    );
  });

  it("uses plurals and digits past ten", () => {
    expect(todaySummary(make([item("a"), item("b")], 2, 12))).toBe(
      "Dois posts para publicar, dois itens atrasados e 12 compromissos nos próximos 7 dias.",
    );
  });

  it("leaves out zero counts and posts already done", () => {
    expect(todaySummary(make([item("a", true), item("b")], 0, 2))).toBe(
      "Um post para publicar e dois compromissos nos próximos 7 dias.",
    );
    expect(todaySummary(make([], 3, 0))).toBe("Três itens atrasados.");
  });

  it("counts a late post of today once, not as a post and as overdue", () => {
    const today = {
      posts: [item("a"), item("b")],
      overdue: [{ id: "a", kind: "publication" }, item("z")],
      upcoming: [],
    } as unknown as Today;
    expect(todaySummary(today)).toBe("Dois posts para publicar e um item atrasado.");
  });

  it("keeps an overdue deadline on a card that also has a post today", () => {
    // A card's publication and its deadline are two agenda items with the same card id.
    const today = {
      posts: [{ id: "a", kind: "publication", done: false }],
      overdue: [{ id: "a", kind: "due" }],
      upcoming: [],
    } as unknown as Today;
    expect(todaySummary(today)).toBe("Um post para publicar e um item atrasado.");
  });

  it("uses the singular for exactly one upcoming item", () => {
    expect(todaySummary(make([], 0, 1))).toBe("Um compromisso nos próximos 7 dias.");
  });

  it("says so when nothing is pending", () => {
    expect(todaySummary(make([item("a", true)], 0, 0))).toBe("Nada pendente por aqui.");
  });
});

describe("plurals", () => {
  it("picks the singular for exactly one", () => {
    expect(tn("cadence.missing", 1)).toBe("Falta 1 para a meta.");
    expect(tn("cadence.missing", 2)).toBe("Faltam 2 para a meta.");
    expect(tn("import.apply", 1)).toBe("Importar 1 post");
    expect(tn("import.apply", 3)).toBe("Importar 3 posts");
    expect(tn("insights.summary", 1, { avg: "2.0" })).toBe("1 post, média de 2.0 de engajamento");
  });

  it("leaves no optional plural left in a message", () => {
    expect(Object.entries(ptBR).filter(([, v]) => /\(s\)|\(es\)/.test(v))).toEqual([]);
  });
});

describe("markdown", () => {
  const body = "\n## LinkedIn\n\nTexto do post.\n\n## Instagram\n\nLegenda.\n\n## Notas\n\nFontes.\n";

  it("post text picks the network section", () => {
    expect(postText(body, ["Instagram", "instagram"])).toBe("Legenda.");
    expect(postText("Olá mundo\n\n## Notas\n\nprivado\n", ["LinkedIn"])).toBe("Olá mundo");
    expect(postText("```\n## Notas\n```\ntexto", ["LinkedIn"])).toBe("```\n## Notas\n```\ntexto");
  });

  it("never counts the Arte brief as post text", () => {
    expect(postText("Texto único.\n\n## Arte\n\nbrief secreto\n\n## Notas\n\nnota\n", ["LinkedIn"])).toBe("Texto único.");
    expect(postText("\n## Instagram\n\nLegenda.\n\n## Arte\n\nbrief\n", ["Instagram"])).toBe("Legenda.");
  });

  it("toggles the n-th checkbox only", () => {
    const b = "- [ ] a\n- [x] b\n- [ ] c\n";
    expect(toggleTask(b, 1)).toBe("- [ ] a\n- [ ] b\n- [ ] c\n");
    expect(toggleTask(b, 2)).toBe("- [ ] a\n- [x] b\n- [x] c\n");
  });

  it("counts code points", () => {
    expect(charCount("ação 🚀")).toBe(6);
  });
});

describe("week grid layout", () => {
  const item = (time: string | null, end: string | null = null, date = "2026-11-03") =>
    ({
      kind: "publication", id: time ?? "x", board: null, title: "t", network: null, status: null, column: null, done: false,
      when: { date, time, at: null },
      end: end ? { date, time: end, at: null } : null,
    }) as never;

  it("puts overlapping blocks in separate lanes and gives separate ones the full width", async () => {
    const { layoutDay } = await import("./weekgrid");
    const placed = layoutDay([item("09:00:00"), item("09:15:00"), item("12:00:00"), item(null)], "2026-11-03");
    expect(placed).toHaveLength(3); // the all-day item is not in the time grid
    expect(placed.map((p) => [p.lane, p.lanes])).toEqual([[0, 2], [1, 2], [0, 1]]);
  });

  it("respects an end time and ignores other days", async () => {
    const { layoutDay } = await import("./weekgrid");
    const [a, b] = layoutDay([item("09:00:00", "11:00:00"), item("10:30:00"), item("10:30:00", null, "2026-11-04")], "2026-11-03");
    expect(a.end).toBe(11 * 60);
    expect([b.lane, b.lanes]).toEqual([1, 2]);
  });

  it("snaps drops to quarter hours", async () => {
    const { timeAtOffset, HOUR_PX } = await import("./weekgrid");
    expect(timeAtOffset(HOUR_PX * 9)).toBe("09:00");
    expect(timeAtOffset(HOUR_PX * 9 + HOUR_PX * 0.3)).toBe("09:15");
    expect(timeAtOffset(-20)).toBe("00:00");
    expect(timeAtOffset(HOUR_PX * 99)).toBe("23:45");
  });
});
