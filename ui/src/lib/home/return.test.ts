import { describe, expect, it } from "vitest";
import { shouldReturnHome } from "./return";

describe("shouldReturnHome", () => {
  it("does nothing before a day has been seen", () => {
    expect(shouldReturnHome(null, "2026-11-04", false)).toBe(false);
  });

  it("stays put on the same day", () => {
    expect(shouldReturnHome("2026-11-04", "2026-11-04", false)).toBe(false);
  });

  it("returns on a new day", () => {
    expect(shouldReturnHome("2026-11-03", "2026-11-04", false)).toBe(true);
  });

  it("does not pull the author out of an editor, modal, palette or trash", () => {
    expect(shouldReturnHome("2026-11-03", "2026-11-04", true)).toBe(false);
  });
});
