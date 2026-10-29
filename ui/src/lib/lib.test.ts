import { describe, expect, it } from "vitest";
import { addDays, addMonths, monthGrid, moveToDay, relativeDay, startOfWeek, weekday } from "./dates";
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

  it("rescheduling keeps the time of day", () => {
    expect(moveToDay({ date: "2026-11-03", time: "09:00:00", at: "x" }, "2026-11-05")).toBe("2026-11-05T09:00");
    expect(moveToDay({ date: "2026-12-05", time: null, at: null }, "2026-12-06")).toBe("2026-12-06");
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
