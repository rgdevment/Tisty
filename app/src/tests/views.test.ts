import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { List } from "../core";
import {
  A_COLUMN,
  accepts,
  asView,
  type Chosen,
  headerCount,
  invite,
  title,
  useReach,
} from "../views";

const lists: List[] = [{ id: "01L", name: "work", order: "a0" }];

describe("asView", () => {
  it("asks for the hidden ones only while the drawer is open", () => {
    expect(asView({ named: "archive", folded: true })).toEqual({
      archive: true,
      hidden: true,
      most: A_COLUMN,
    });
  });

  it("opens the archive on the layer that has something to tell", () => {
    expect(asView({ named: "archive" })).toEqual({
      archive: true,
      reading: "story",
      most: A_COLUMN,
    });
    expect(asView({ named: "archive", layer: "trace" })).toEqual({
      archive: true,
      reading: "trace",
      most: A_COLUMN,
    });
  });

  it("drops the layer while the hidden drawer is open, so nothing goes missing there", () => {
    expect(asView({ named: "archive", layer: "story", folded: true })).toEqual({
      archive: true,
      hidden: true,
      most: A_COLUMN,
    });
  });

  it("lets a chosen list outrank whatever else was selected", () => {
    expect(asView({ named: "tasks", list: "01L" })).toEqual({ list: "01L", most: A_COLUMN });
  });

  it("reaches into the archive for tags, unlike every other view", () => {
    expect(asView({ tags: ["home"] })).toEqual({
      tags: ["home"],
      everything: true,
      most: A_COLUMN,
    });
    expect(asView({ named: "tags" })).toEqual({
      tagged: true,
      everything: true,
      most: A_COLUMN,
    });
  });

  it("asks for a wider window when the column has been scrolled to its end", () => {
    expect(asView({ named: "tasks" }, A_COLUMN * 3)).toEqual({
      window: "today",
      most: A_COLUMN * 3,
    });
    expect(asView({ named: "archive" }, A_COLUMN * 2).most).toBe(A_COLUMN * 2);
    expect(asView({ named: "quadrants" }, A_COLUMN * 2)).toEqual({
      board: true,
      most: A_COLUMN * 2,
    });
  });

  it("asks the lists screen for the three of each and no column of its own", () => {
    expect(asView({ named: "lists" })).toEqual({ soonest: true, most: A_COLUMN });
  });

  it("falls back to today", () => {
    expect(asView({})).toEqual({ most: A_COLUMN });
    expect(asView({ named: "tasks" })).toEqual({ window: "today", most: A_COLUMN });
  });
});

describe("accepts", () => {
  it("refuses the views with nothing to add to", () => {
    expect(accepts({ named: "archive" })).toBe(false);
    expect(accepts({ named: "search" })).toBe(false);
    expect(accepts({ named: "keeping" })).toBe(false);
  });

  it("allows tags only once one is picked, because the task inherits it", () => {
    expect(accepts({ named: "tags" })).toBe(false);
    expect(accepts({ named: "tags", tags: ["home"] })).toBe(true);
  });
});

describe("invite", () => {
  it("says where the task will land", () => {
    expect(invite({ list: "01L" }, lists)).toBe("Add to work");
    expect(invite({ tags: ["home", "urgent"] }, lists)).toBe("Add with #home #urgent");
    expect(invite({ named: "tasks" }, lists)).toBe("Add for today");
  });

  it("stays generic when the list is gone", () => {
    expect(invite({ list: "vanished" }, lists)).toBe("Add a task");
  });
});

describe("title", () => {
  it("empties rather than showing an identifier when the list is gone", () => {
    expect(title({ list: "vanished" }, lists)).toBe("");
    expect(title({ list: "01L" }, lists)).toBe("work");
  });

  it("joins the chosen tags", () => {
    expect(title({ tags: ["home", "urgent"] }, lists)).toBe("#home #urgent");
  });
});

describe("what the header counts", () => {
  it("says nothing over a search nobody has typed into yet", () => {
    expect(headerCount({ named: "search" }, null, { tasks: 4820 }, 4820)).toBeUndefined();
  });

  it("says how many were found once something was typed", () => {
    expect(headerCount({ named: "search" }, { tasks: [1, 2] }, {}, 4820)).toBe(2);
  });

  it("says how many there are, not how many travelled", () => {
    expect(headerCount({ named: "archive" }, null, {}, 4820)).toBe(4820);
  });
});

describe("how far a column reaches", () => {
  it("hands back the same further across renders, so the edge arms once", () => {
    const chosen: Chosen = { named: "tasks" };
    const seen = renderHook(({ one }) => useReach(one), { initialProps: { one: chosen } });
    const first = seen.result.current.further;

    seen.rerender({ one: chosen });

    expect(seen.result.current.further).toBe(first);
  });

  it("is back to one column the moment the view changes, before anything is asked", () => {
    const seen = renderHook(({ one }) => useReach(one), {
      initialProps: { one: { named: "tasks" } as Chosen },
    });
    act(() => seen.result.current.further());
    expect(seen.result.current.reach).toBe(A_COLUMN * 2);

    seen.rerender({ one: { named: "archive" } as Chosen });

    expect(seen.result.current.reach).toBe(A_COLUMN);
  });
});
