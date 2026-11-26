import type { Asked } from "tisty-editor/edit";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { type InsertAsked, insertChoices } from "../ui/InsertMenu";
import type { Choice } from "../ui/Menu";

const ALL: Asked[] = [
  "newPage",
  "page",
  "newDoc",
  "doc",
  "newTask",
  "task",
  "icon",
  "file",
  "diagram",
  "diagramFrom",
];

const asked = (more: Partial<InsertAsked> = {}): InsertAsked => ({
  kinds: ALL,
  steps: [],
  sections: false,
  pick: vi.fn(),
  ...more,
});

const rows = (more: Partial<InsertAsked> = {}) => insertChoices(asked(more)).choices;
const keys = (list: Choice[]) => list.map((one) => one.key);
const every = (list: Choice[]): Choice[] =>
  list.flatMap((one) => [one, ...every(one.into?.choices ?? [])]);

describe("the rows of the Insert menu", () => {
  it("come in four groups, in order, each one a single run", () => {
    const groups = rows().map((one) => one.group);

    expect([...new Set(groups)]).toEqual([
      t("insertGroupText"),
      t("insertGroupLists"),
      t("insertGroupInsert"),
      t("insertGroupLink"),
    ]);
    expect(groups.filter((one, at) => one !== groups[at - 1])).toHaveLength(4);
  });

  it("are 21 with everything allowed and no steps, 22 with steps", () => {
    expect(rows()).toHaveLength(21);
    expect(rows({ steps: ["Pan"] })).toHaveLength(22);
  });

  it("never offer a format that wraps what is selected", () => {
    const gone = ["bold", "italic", "underline", "strike", "inline", "link", "pen"];
    const there = keys(every(rows({ steps: ["Pan"] })));

    for (const key of gone) expect(there, key).not.toContain(key);
  });

  it("open a list of six for the callout", () => {
    const callout = rows().find((one) => one.key === "callout");

    expect(callout?.into?.choices).toHaveLength(6);
    expect(keys(callout?.into?.choices ?? [])).toContain("callout-idea");
  });

  it("open the diagram list with its three kinds", () => {
    const diagram = rows().find((one) => one.key === "diagram");

    expect(keys(diagram?.into?.choices ?? [])).toEqual(["diagram", "diagramFrom", "mermaid"]);
  });

  it("open the steps of the page, each picked as a reference to its number", () => {
    const pick = vi.fn();
    const step = rows({ steps: ["Comprar pan", "Hornear"], pick }).find(
      (one) => one.key === "step",
    );
    const inside = step?.into?.choices ?? [];

    expect(inside.map((one) => one.label)).toEqual(["1. Comprar pan", "2. Hornear"]);
    inside[1].onPick?.();
    expect(pick).toHaveBeenCalledWith(
      expect.objectContaining({ put: { inline: "[[#2]]", at: 6 } }),
    );
  });

  it("leave out the step row when the page has no steps", () => {
    expect(keys(rows())).not.toContain("step");
  });

  it("leave out the three headings on a task page, whose sections have their own buttons", () => {
    const plain = keys(rows());
    const kept = keys(rows({ sections: true }));

    expect(plain).toEqual(expect.arrayContaining(["h1", "h2", "h3"]));
    for (const key of ["h1", "h2", "h3"]) expect(kept).not.toContain(key);
    expect(kept).toContain("paragraph");
  });

  it("leave out what the folder does not let be asked for", () => {
    const kept = keys(rows({ kinds: [] }));

    for (const key of ["file", "icon", "newpage", "pageOrDoc", "task"])
      expect(kept, key).not.toContain(key);
    expect(kept).toEqual(expect.arrayContaining(["paragraph", "table", "code", "callout"]));
    // Only the Mermaid text is left of the diagrams, so that row is it alone.
    const diagram = rows({ kinds: [] }).find((one) => one.key === "diagram");
    expect(diagram?.into).toBeUndefined();
  });

  it("collapse a list left with one row into that row, keeping the words of the list", () => {
    const one = rows({ kinds: ["doc"] }).find((row) => row.key === "pageOrDoc");

    expect(one?.into).toBeUndefined();
    expect(one?.group).toBe(t("insertGroupLink"));
    expect(one?.words).toContain("page");
  });

  it("drop a list left with no row at all", () => {
    const list = keys(rows({ kinds: ["file"] }));

    expect(list).not.toContain("pageOrDoc");
    expect(list).not.toContain("task");
  });

  it("pick through the host with the block they stand for", () => {
    const pick = vi.fn();
    rows({ pick })
      .find((one) => one.key === "table")
      ?.onPick?.();

    expect(pick).toHaveBeenCalledWith(expect.objectContaining({ key: "table" }));
  });
});

describe("the shortcut tiles of the Insert menu", () => {
  it("are six, the blocks most often wanted", () => {
    const { row } = insertChoices(asked());

    expect(keys(row)).toEqual(["h1", "bullets", "todo", "table", "file", "code"]);
    expect(row.every((one) => one.glyph)).toBe(true);
  });

  it("follow what the page allows", () => {
    expect(keys(insertChoices(asked({ sections: true })).row)).not.toContain("h1");
    expect(keys(insertChoices(asked({ kinds: [] })).row)).not.toContain("file");
  });

  it("pick the block they stand for", () => {
    const pick = vi.fn();
    insertChoices(asked({ pick })).row[0].onPick?.();

    expect(pick).toHaveBeenCalledWith(expect.objectContaining({ key: "h1" }));
  });
});

describe("the labels of the Insert menu rows", () => {
  it("show the entry's own label on the picture row, not the block's", () => {
    const file = rows().find((one) => one.key === "file");

    expect(file?.label).toBe(t("insertPicture"));
  });

  it("keep the block's label on a row whose entry has none", () => {
    const table = rows().find((one) => one.key === "table");

    expect(table?.label).toBe(t("table"));
  });
});

describe("the search words of a renamed row", () => {
  it("keep the name the block goes by, beside the row's own label", () => {
    const file = rows().find((one) => one.key === "file");

    expect(file?.label).toBe(t("insertPicture"));
    expect(file?.words).toContain(t("attachFile"));
  });
});
