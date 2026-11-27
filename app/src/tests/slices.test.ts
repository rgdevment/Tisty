import { describe, expect, it } from "vitest";
import { t } from "../locales";
import { A_COLUMN, accepts, asView, invite, nothing, SLICES } from "../views";

describe("Tasks, the view that replaced Today and Upcoming", () => {
  it("means today when nothing was chosen", () => {
    expect(asView({ named: "tasks" })).toEqual({ window: "today", most: A_COLUMN });
  });

  it.each([
    ["today", { window: "today", most: A_COLUMN }],
    ["upcoming", { window: "upcoming", most: A_COLUMN }],
    ["repeating", { repeating: true, most: A_COLUMN }],
    ["all", { most: A_COLUMN }],
  ] as const)("asks the core for %s", (slice, view) => {
    expect(asView({ named: "tasks", slice })).toEqual(view);
  });

  it("offers the four the author settled on", () => {
    expect(SLICES).toEqual(["today", "upcoming", "repeating", "all"]);
  });

  it("asks for habits without asking for a day", () => {
    expect(asView({ named: "tasks", slice: "repeating" })).not.toHaveProperty("window");
  });

  it("asks only for what an agent said, narrowed to the picked lists", () => {
    expect(asView({ named: "tasks", slice: "said" })).toEqual({ said: true, most: A_COLUMN });
    expect(asView({ named: "tasks", slice: "said", lists: ["01A"] })).toEqual({
      said: true,
      lists: ["01A"],
      most: A_COLUMN,
    });
  });

  it("lets a chosen list win over the slice", () => {
    expect(asView({ named: "tasks", slice: "all", list: "01L" })).toEqual({
      list: "01L",
      most: A_COLUMN,
    });
  });
});

describe("capturing from a slice", () => {
  it.each(["today", "upcoming", "repeating", "all"] as const)("is offered on %s", (slice) => {
    expect(accepts({ named: "tasks", slice })).toBe(true);
  });

  it("says the task lands on today only where that is true", () => {
    expect(invite({ named: "tasks", slice: "today" }, [])).toMatch(/today/i);
    expect(invite({ named: "tasks", slice: "upcoming" }, [])).not.toMatch(/today/i);
  });
});

describe("what an empty slice says", () => {
  it("says something different for each one", () => {
    const said = (["today", "upcoming", "repeating", "all"] as const).map((slice) =>
      nothing({ named: "tasks", slice }, false),
    );

    expect(new Set(said).size).toBe(said.length);
  });

  it("says no agent is waiting when nothing is left to confirm", () => {
    expect(nothing({ named: "tasks", slice: "said" }, false)).toBe(t("saidEmpty"));
  });

  it("still teaches the syntax where a new reader lands", () => {
    expect(nothing({ named: "tasks", slice: "today" }, false)).toMatch(/«[^»]+»/);
  });
});
