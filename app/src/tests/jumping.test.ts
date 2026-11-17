import { describe, expect, it, vi } from "vitest";
import type { Filed, Folded, Found, Task } from "../core";
import { type Going, jumps } from "../jumping";
import { latest } from "../lately";

const doc = (id: string, title: string, wrote?: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `file-${id}`,
  title,
  folder: null,
  archived: false,
  away: false,
  wrote: wrote ?? null,
  ...more,
});

const folder = (id: string, name: string, parent: string | null = null): Folded => ({
  id,
  name,
  parent,
  icon: null,
  holds: 0,
  archived: false,
  away: false,
});

const task = (id: string, title: string, status = "open", list?: string): Task =>
  ({ id, title, status, priority: "unset", order: "a0", list }) as unknown as Task;

const going = (): Going => ({ doc: vi.fn(), folder: vi.fn(), task: vi.fn() });

const labels = (rows: { label: string }[]) => rows.map((one) => one.label);

describe("the documents written last", () => {
  it("come newest first, past the archive, the pages and what has no date", () => {
    const papers = {
      folders: [],
      docs: [
        doc("a", "Old", "2026-11-01T10:00:00Z"),
        doc("b", "New", "2026-11-16T09:00:00.5Z"),
        doc("c", "Middle", "2026-11-10T10:00:00Z"),
        doc("d", "Away", "2026-11-16T10:00:00Z", { away: true }),
        doc("e", "Page", "2026-11-16T10:00:00Z", { pageOf: "a" }),
        doc("f", "Never"),
      ],
    };

    expect(labels(latest(papers, 2).map((one) => ({ label: one.title })))).toEqual([
      "New",
      "Middle",
    ]);
  });
});

describe("a date that cannot be read", () => {
  it("counts as the oldest, so the order of the rest holds", () => {
    const papers = {
      folders: [],
      docs: [
        doc("a", "Old", "2026-11-01T10:00:00Z"),
        doc("b", "Broken", "not a date"),
        doc("c", "New", "2026-11-16T10:00:00Z"),
      ],
    };

    expect(latest(papers, 3).map((one) => one.title)).toEqual(["New", "Old", "Broken"]);
  });
});

describe("where the palette can jump", () => {
  const papers = {
    folders: [folder("F1", "Recipes"), folder("F2", "Soups", "F1"), folder("F3", "Gone")],
    docs: [
      doc("a", "Bread recipe", "2026-11-01T10:00:00Z"),
      doc("b", "A recipe for soup", "2026-11-15T10:00:00Z"),
      doc("c", "Recipes I love", "2026-11-02T10:00:00Z"),
      doc("d", "January", "2026-11-03T10:00:00Z", { pageOf: "c" }),
    ],
  };

  it("offers the documents written last when nothing is asked", () => {
    const rows = jumps(papers, [], "", null, going());

    expect(labels(rows)).toEqual(["A recipe for soup", "Recipes I love", "Bread recipe"]);
    expect(rows.every((one) => one.group === "Written lately")).toBe(true);
  });

  it("puts the titles that begin with the words first, then the newest", () => {
    const rows = jumps(papers, [], "recipe", null, going());

    expect(labels(rows.filter((one) => one.group === "Documents"))).toEqual([
      "Recipes I love",
      "A recipe for soup",
      "Bread recipe",
    ]);
  });

  it("reads the words without minding accents or case", () => {
    const rows = jumps(
      { folders: [], docs: [doc("a", "Canción de cuna")] },
      [],
      "CANCION",
      null,
      going(),
    );

    expect(labels(rows)).toEqual(["Canción de cuna"]);
  });

  it("names a page by the document it belongs to", () => {
    const rows = jumps(papers, [], "january", null, going());

    expect(rows[0]).toMatchObject({ label: "January", detail: "Recipes I love" });
  });

  it("finds folders by name and says where they sit", () => {
    const rows = jumps(papers, [], "soups", null, going()).filter((one) => one.group === "Folders");

    expect(rows).toEqual([expect.objectContaining({ label: "Soups", detail: "Recipes" })]);
  });

  it("brings the tasks the store found, the open ones first, with their list", () => {
    const found: Found = {
      tasks: [task("t1", "Bake bread", "done", "L1"), task("t2", "Buy flour", "open", "L1")],
      papers: [],
      total: 2,
    };
    const lists = [{ id: "L1", name: "Kitchen", order: "a0" }];

    const rows = jumps(papers, lists, "bread", found, going()).filter(
      (one) => one.group === "Tasks",
    );

    expect(rows.map((one) => [one.label, one.detail])).toEqual([
      ["Buy flour", "Kitchen"],
      ["Bake bread", "Kitchen · archived"],
    ]);
  });

  it("adds what the text holds, without a document already found by its title", () => {
    const found: Found = {
      tasks: [],
      papers: [
        { id: "file-a", title: "Bread recipe", line: "knead the dough", archived: false },
        { id: "file-z", title: "Notes", line: "bread on Sunday", archived: false },
      ],
      total: 0,
    };

    const rows = jumps(papers, [], "bread", found, going()).filter(
      (one) => one.group === "In the text",
    );

    expect(rows.map((one) => [one.label, one.detail])).toEqual([["Notes", "bread on Sunday"]]);
  });

  it("says when what the text holds was put away", () => {
    const found: Found = {
      tasks: [],
      papers: [{ id: "file-z", title: "Old notes", line: "bread long ago", archived: true }],
      total: 0,
    };

    const rows = jumps(papers, [], "bread", found, going()).filter(
      (one) => one.group === "In the text",
    );

    expect(rows.map((one) => one.detail)).toEqual(["bread long ago · archived"]);
  });

  it("opens each kind with its own hand", () => {
    const go = going();
    const found: Found = { tasks: [task("t1", "Soup night")], papers: [], total: 1 };
    const rows = jumps(papers, [], "soup", found, go);

    for (const one of rows) one.onPick();

    expect(go.doc).toHaveBeenCalledWith("file-b");
    expect(go.folder).toHaveBeenCalledWith("F2");
    expect(go.task).toHaveBeenCalledWith(expect.objectContaining({ id: "t1" }));
  });
});
