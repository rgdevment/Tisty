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
    ["undated", { window: "undated" }],
  ] as const)("asks the core for %s", (slice, view) => {
    expect(asView({ named: "tasks", slice })).toEqual(view);
  });

  /// There is no «all» chip on purpose: `today` already carries what has no
  /// date, so today ∪ upcoming is every open task. A fourth chip would show
  /// the union of the other two under a name answering no question of its own.
  it("offers three slices, because a fourth would be the sum of two", () => {
    expect(SLICES).toEqual(["today", "upcoming", "undated"]);
  });

  /// A list still outranks it, as it always did.
  it("lets a chosen list win over the slice", () => {
    expect(asView({ named: "tasks", slice: "undated", list: "01L" })).toEqual({ list: "01L" });
  });
});

describe("capturing from a slice", () => {
  /// A task written with no day lands on today. In any other slice it would
  /// disappear the instant it was typed.
  it("is offered on today, where what you write stays visible", () => {
    expect(accepts({ named: "tasks", slice: "today" })).toBe(true);
    expect(accepts({ named: "tasks" })).toBe(true);
  });

  it.each(["upcoming", "undated"] as const)("is refused on %s", (slice) => {
    expect(accepts({ named: "tasks", slice })).toBe(false);
  });

  it("still says the task will land on today", () => {
    expect(invite({ named: "tasks", slice: "today" }, [])).toMatch(/today/i);
  });
});

describe("what an empty slice says", () => {
  it("says something different for each one", () => {
    const said = (["today", "upcoming", "undated"] as const).map((slice) =>
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
