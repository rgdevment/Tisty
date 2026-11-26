import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { pagesIn, remembered, standingIn, treeIndex } from "../ui/treeIndex";

const doc = (id: string, folder: string | null, pageOf?: string): Filed => ({
  id,
  file: id,
  title: id,
  folder,
  pageOf,
  archived: false,
  away: false,
});

describe("treeIndex", () => {
  const docs = [
    doc("a", "f1"),
    doc("b", null),
    doc("c", "f1"),
    doc("p2", "f1", "a"),
    doc("d", "f2"),
    doc("p1", "f1", "a"),
    doc("e", null),
  ];
  const index = treeIndex(docs);

  it("groups the documents by folder in the order of the list", () => {
    expect(standingIn(index, "f1").map((one) => one.id)).toEqual(["a", "c"]);
    expect(standingIn(index, "f2").map((one) => one.id)).toEqual(["d"]);
  });

  it("keeps the loose ones under null", () => {
    expect(standingIn(index, null).map((one) => one.id)).toEqual(["b", "e"]);
  });

  it("keeps pages out of their folder and groups them under their document, in order", () => {
    expect(pagesIn(index, "a").map((one) => one.id)).toEqual(["p2", "p1"]);
    expect(standingIn(index, "f1").some((one) => one.pageOf)).toBe(false);
  });

  it("answers an empty array for a key it never saw", () => {
    expect(standingIn(index, "nope")).toEqual([]);
    expect(pagesIn(index, "nope")).toEqual([]);
    expect(standingIn(treeIndex([]), null)).toEqual([]);
  });

  it("hands back the same documents, not copies", () => {
    expect(standingIn(index, "f2")[0]).toBe(docs[4]);
  });
});

describe("remembered", () => {
  it("reads once per key and gives the same answer again", () => {
    const read = vi.fn((key: string | null) => [key]);
    const ask = remembered(read);
    const first = ask("a");
    expect(ask("a")).toBe(first);
    ask("b");
    ask(null);
    ask(null);
    expect(read).toHaveBeenCalledTimes(3);
  });

  it("keeps a falsy answer instead of reading again", () => {
    const read = vi.fn(() => 0);
    const ask = remembered(read);
    ask("x");
    ask("x");
    expect(read).toHaveBeenCalledTimes(1);
  });
});
