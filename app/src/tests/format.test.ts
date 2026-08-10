import { describe, expect, it } from "vitest";
import type { DateSpec } from "../core";
import { daysFrom, isOverdue, isToday, monthOf, whenLabel } from "../format";

const spec = (at: string, has_time = false): DateSpec => ({
  at,
  tz: "America/Santiago",
  floating: false,
  has_time,
});

const noon = (day: string) => new Date(`${day} 12:00:00`);

describe("monthOf", () => {
  it("says nothing about a task that never closed", () => {
    expect(monthOf(undefined)).toBe("");
    expect(monthOf("")).toBe("");
    expect(monthOf("whenever")).toBe("");
  });

  it("names the year only once it stops being this one", () => {
    expect(monthOf("2026-03-04 10:00:00", noon("2026-08-10"))).toBe("March");
    expect(monthOf("2025-03-04 10:00:00", noon("2026-08-10"))).toBe("March 2025");
  });
});

describe("daysFrom", () => {
  it("counts calendar days, not twenty-four hour spans", () => {
    expect(daysFrom("2026-08-11 01:00:00", new Date("2026-08-10 23:00:00"))).toBe(1);
    expect(daysFrom("2026-08-10 23:00:00", new Date("2026-08-10 01:00:00"))).toBe(0);
  });

  it("goes negative for what already passed", () => {
    expect(daysFrom("2026-08-09 09:00:00", noon("2026-08-10"))).toBe(-1);
  });
});

describe("isOverdue and isToday", () => {
  it("today is not overdue, however late in the day", () => {
    const now = new Date("2026-08-10 23:30:00");
    expect(isOverdue(spec("2026-08-10 09:00:00", true), now)).toBe(false);
    expect(isToday(spec("2026-08-10 09:00:00", true), now)).toBe(true);
  });

  it("yesterday is", () => {
    expect(isOverdue(spec("2026-08-09 09:00:00"), noon("2026-08-10"))).toBe(true);
  });
});

describe("whenLabel", () => {
  it("names the near days and dates the far ones", () => {
    const now = noon("2026-08-10");
    expect(whenLabel(spec("2026-08-10 09:00:00"), now)).toBe("today");
    expect(whenLabel(spec("2026-08-11 09:00:00"), now)).toBe("tomorrow");
    expect(whenLabel(spec("2026-08-09 09:00:00"), now)).toBe("yesterday");
    expect(whenLabel(spec("2026-08-20 09:00:00"), now)).toMatch(/Aug/);
  });

  it("shows a clock only where the parser found one", () => {
    const now = noon("2026-08-10");
    expect(whenLabel(spec("2026-08-10 09:00:00", true), now)).toMatch(/^today 09:00/);
    expect(whenLabel(spec("2026-08-10 09:00:00", false), now)).toBe("today");
  });
});
