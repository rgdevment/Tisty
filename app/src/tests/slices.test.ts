import { describe, expect, it } from "vitest";
import { accepts, asView, invite, nothing, SLICES } from "../views";

describe("Tasks, the view that replaced Today and Upcoming", () => {
  /// They were two names for one thing: a date filter with a fixed seat in the
  /// sidebar. Opening on «all» would be forty rows the moment you arrive.
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

  /// «Today» already carries what has no date, so there is no «undated» chip.
  it("offers the four the author settled on", () => {
    expect(SLICES).toEqual(["today", "upcoming", "repeating", "all"]);
  });

  /// It cuts across the calendar, not along it, which is exactly why «all»
  /// earns its place: the other three no longer cover the whole.
  it("asks for habits without asking for a day", () => {
    expect(asView({ named: "tasks", slice: "repeating" })).not.toHaveProperty("window");
  });

  /// A list still outranks it, as it always did.
  it("lets a chosen list win over the slice", () => {
    expect(asView({ named: "tasks", slice: "all", list: "01L" })).toEqual({ list: "01L" });
  });
});

describe("capturing from a slice", () => {
  /// It used to be refused anywhere but «today», reasoning that a task written
  /// with no day would vanish. It does not: the notice after a capture shows
  /// what was filed and opens it, wherever the list happens to be looking.
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

  it("keeps the inbox saying its own thing", () => {
    expect(nothing({ named: "inbox" }, false)).toMatch(/inbox/i);
  });
});

describe("Tasks and the Inbox are not the same list", () => {
  /// The question each one answers is different: the inbox is «what have I not
  /// filed yet», which is about LISTS; the slices are about DAYS.
  it("asks the core for different things", () => {
    expect(asView({ named: "inbox" })).toEqual({ inbox: true });
    expect(asView({ named: "tasks", slice: "today" })).toEqual({ window: "today" });
  });
});
