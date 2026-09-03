import { describe, expect, it } from "vitest";
import { LOOSE, marked, settled, type Spot, zoneIn } from "../ui/dragging";

const folder = (over: Partial<Spot> = {}): Spot => ({
  id: "f1",
  kind: "folder",
  holds: true,
  ...over,
});

const doc = (over: Partial<Spot> = {}): Spot => ({
  id: "d1",
  kind: "doc",
  holds: true,
  ...over,
});

describe("which third of a row the pointer is in", () => {
  it("splits a row that can hold something into three", () => {
    expect(zoneIn(100, 30, 104, true)).toBe("before");
    expect(zoneIn(100, 30, 115, true)).toBe("in");
    expect(zoneIn(100, 30, 126, true)).toBe("after");
  });

  it("splits a row that holds nothing in half, so there is no dead middle", () => {
    expect(zoneIn(100, 30, 112, false)).toBe("before");
    expect(zoneIn(100, 30, 118, false)).toBe("after");
  });

  it("calls a row with no height 'in', which is what an unlaid page can honestly say", () => {
    expect(zoneIn(0, 0, 0, true)).toBe("in");
  });

  it("names the spot it lit so the row and its edges never share a mark", () => {
    expect(marked(folder(), "in")).toBe("f1");
    expect(marked(folder(), "before")).toBe("f1:before");
    expect(marked(folder(), "after")).toBe("f1:after");
  });
});

describe("where a dragged row lands", () => {
  it("puts a folder inside the folder it was dropped into", () => {
    expect(settled({ id: "f2", kind: "folder" }, folder(), "in")).toEqual({
      kind: "folder",
      moved: "f2",
      folder: "f1",
    });
  });

  it("orders a folder against its sister, before and after", () => {
    const beside = folder({ parent: "top", next: "f9" });
    expect(settled({ id: "f2", kind: "folder" }, beside, "before")).toEqual({
      kind: "folder",
      moved: "f2",
      folder: "top",
      before: "f1",
    });
    expect(settled({ id: "f2", kind: "folder" }, beside, "after")).toEqual({
      kind: "folder",
      moved: "f2",
      folder: "top",
      before: "f9",
    });
  });

  it("sends a folder dropped past the last sister to the end, with nothing to go before", () => {
    const last = folder({ parent: "top", next: undefined });
    expect(settled({ id: "f2", kind: "folder" }, last, "after")).toEqual({
      kind: "folder",
      moved: "f2",
      folder: "top",
      before: undefined,
    });
  });

  it("takes a folder dropped on a document into the folder that document sits in", () => {
    expect(settled({ id: "f2", kind: "folder" }, doc({ parent: "f7" }), "in")).toEqual({
      kind: "folder",
      moved: "f2",
      folder: "f7",
    });
  });

  it("files a document into the folder it was dropped into", () => {
    expect(settled({ id: "d2", kind: "doc" }, folder(), "in")).toEqual({
      kind: "doc",
      moved: "d2",
      folder: "f1",
    });
  });

  it("puts a document dropped at a folder's edge beside the folder, not inside it", () => {
    expect(settled({ id: "d2", kind: "doc" }, folder({ parent: "top" }), "before")).toEqual({
      kind: "doc",
      moved: "d2",
      folder: "top",
    });
  });

  it("orders a document against another in the same folder", () => {
    const beside = doc({ parent: "f1", next: "d9" });
    expect(settled({ id: "d2", kind: "doc" }, beside, "before")).toEqual({
      kind: "doc",
      moved: "d2",
      folder: "f1",
      before: "d1",
    });
    expect(settled({ id: "d2", kind: "doc" }, beside, "after")).toEqual({
      kind: "doc",
      moved: "d2",
      folder: "f1",
      before: "d9",
    });
  });

  it("makes a page of a document dropped on the middle of another", () => {
    expect(settled({ id: "d2", kind: "doc" }, doc(), "in")).toEqual({
      kind: "doc",
      moved: "d2",
      pageOf: "d1",
    });
  });

  it("refuses the middle of a document that holds no pages", () => {
    expect(settled({ id: "d2", kind: "doc" }, doc({ holds: false }), "in")).toBeNull();
  });

  it("takes anything dropped on the loose list out of every folder", () => {
    expect(settled({ id: "d2", kind: "doc" }, folder({ id: LOOSE }), "in")).toEqual({
      kind: "doc",
      moved: "d2",
      folder: undefined,
    });
    expect(settled({ id: "f2", kind: "folder" }, folder({ id: LOOSE }), "in")).toEqual({
      kind: "folder",
      moved: "f2",
      folder: undefined,
    });
  });

  it("does nothing for a row dropped on itself, at any edge", () => {
    for (const where of ["before", "in", "after"] as const) {
      expect(settled({ id: "f1", kind: "folder" }, folder(), where)).toBeNull();
      expect(settled({ id: "d1", kind: "doc" }, doc(), where)).toBeNull();
    }
  });

  it("does nothing when the pointer is over no row at all", () => {
    expect(settled({ id: "d2", kind: "doc" }, null, "in")).toBeNull();
  });
});
