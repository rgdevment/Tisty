import { describe, expect, it } from "vitest";
import type { List } from "../core";
import { accepts, asView, invite, title } from "../views";

const lists: List[] = [{ id: "01L", name: "work", order: "a0" }];

describe("asView", () => {
  it("asks for the hidden ones only while the drawer is open", () => {
    expect(asView({ named: "archive" })).toEqual({ archive: true });
    expect(asView({ named: "archive", folded: true })).toEqual({ archive: true, hidden: true });
  });

  it("lets a chosen list outrank whatever else was selected", () => {
    expect(asView({ named: "tasks", list: "01L" })).toEqual({ list: "01L" });
  });

  it("reaches into the archive for tags, unlike every other view", () => {
    expect(asView({ tags: ["home"] })).toEqual({ tags: ["home"], everything: true });
    expect(asView({ named: "tags" })).toEqual({ tagged: true, everything: true });
  });

  it("falls back to today", () => {
    expect(asView({})).toEqual({});
    expect(asView({ named: "tasks" })).toEqual({ window: "today" });
  });
});

describe("accepts", () => {
  it("refuses the views with nothing to add to", () => {
    expect(accepts({ named: "archive" })).toBe(false);
    expect(accepts({ named: "search" })).toBe(false);
    expect(accepts({ named: "tasks", slice: "upcoming" })).toBe(false);
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
    expect(invite({ named: "inbox" }, lists)).toBe("Add to the inbox");
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
