import { describe, expect, it } from "vitest";
import { accepts, asView, invite, nothing, SLICES } from "../views";

describe("Tasks, the view that replaced Today and Upcoming", () => {
  it("means today when nothing was chosen", () => {
    expect(asView({ named: "tasks" })).toEqual({ window: "today" });
  });

  it.each([
    ["today", { window: "today" }],
    ["upcoming", { window: "upcoming" }],
    ["repeating", { repeating: true }],
    ["all", {}],
  ] as const)("asks the core for %s", (slice, view) => {
    expect(asView({ named: "tasks", slice })).toEqual(view);
  });

  it("offers the four the author settled on", () => {
    expect(SLICES).toEqual(["today", "upcoming", "repeating", "all"]);
  });

  it("asks for habits without asking for a day", () => {
    expect(asView({ named: "tasks", slice: "repeating" })).not.toHaveProperty("window");
  });

  it("lets a chosen list win over the slice", () => {
    expect(asView({ named: "tasks", slice: "all", list: "01L" })).toEqual({ list: "01L" });
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

  it("still teaches the syntax where a new reader lands", () => {
    expect(nothing({ named: "tasks", slice: "today" }, false)).toMatch(/tomorrow/i);
  });
});
