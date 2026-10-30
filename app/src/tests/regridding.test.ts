import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView, keymap, runScopeHandlers } from "@codemirror/view";
import { act } from "@testing-library/react";
import { gridFailed, gridKeys, hosted, regrid } from "tisty-editor/edit";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { HOST } from "../ui/hosting";

const core = vi.hoisted(() => ({
  near: vi.fn<(body: string, at: number, ...edits: unknown[]) => Promise<string>>(),
}));

vi.mock("../tables", () => ({
  tableEditNear: core.near,
  refused: (why: unknown) => new Error(`refused ${String(why)}`),
}));

// A word with an accent sits before the table, so its place in bytes and in characters differ.
const TEXT = "Señal\n\n| Nombre | Edad |\n| --- | --- |\n| Rina | 4 |\n| Toto | 7 |\n\nFin";
const MOVED = "Señal\n\n| Nombre | Edad |\n| --- | --- |\n| Toto | 7 |\n| Rina | 4 |\n\nFin";

const bytes = (said: string) => new TextEncoder().encode(said).length;

const view = (head: number, doc = TEXT, failed = vi.fn()) =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc,
      selection: EditorSelection.single(head),
      extensions: [
        hosted.of(HOST),
        markdown({ base: markdownLanguage }),
        keymap.of(gridKeys),
        gridFailed.of(failed),
      ],
    }),
  });

const press = (editor: EditorView, key: string) =>
  runScopeHandlers(editor, new KeyboardEvent("keydown", { key, altKey: true }), "editor");

const settle = () => act(async () => {});

beforeEach(() => {
  core.near.mockReset().mockResolvedValue(MOVED);
});

describe("a table changed from the text editor", () => {
  it("moves the row under the caret through the core, by its first cell's byte, and the caret goes with it", async () => {
    const caret = TEXT.indexOf("Rina") + 1;
    const editor = view(caret);
    expect(press(editor, "ArrowDown")).toBe(true);
    await settle();
    expect(core.near).toHaveBeenCalledWith(TEXT, bytes(TEXT.slice(0, TEXT.indexOf("Rina"))), {
      kind: "rowMove",
      from: 0,
      to: 1,
    });
    expect(editor.state.doc.toString()).toBe(MOVED);
    const head = editor.state.selection.main.head;
    expect(editor.state.sliceDoc(head, head + 4)).toBe("Rina");
  });

  it("finds a table under a list item by the caret, not by where its indented line starts", async () => {
    const listed = "- Lista\n\n  | a |\n  | --- |\n  | 1 |\n  | 2 |\n";
    core.near.mockResolvedValue(listed);
    const caret = listed.indexOf("| 1") + 2;
    const editor = view(caret, listed);
    expect(press(editor, "ArrowDown")).toBe(true);
    await settle();
    expect(core.near).toHaveBeenCalledWith(listed, caret, { kind: "rowMove", from: 0, to: 1 });
  });

  it("names the table by a byte inside it when the caret sits past the last row's end", async () => {
    const ending = "| a |\n| --- |\n| 1 |\n| 2 |";
    core.near.mockResolvedValue(ending);
    const editor = view(ending.length, ending);
    expect(press(editor, "ArrowUp")).toBe(true);
    await settle();
    expect(core.near).toHaveBeenCalledWith(ending, ending.lastIndexOf("2"), {
      kind: "rowMove",
      from: 1,
      to: 0,
    });
  });

  it("finds a table inside a quote past its marks, and the caret follows the row it moved", async () => {
    const quoted = "Dicho:\n\n> | a |\n> | --- |\n> | 1 |\n> | 2 |\n";
    const moved = "Dicho:\n\n> | a |\n> | --- |\n> | 2 |\n> | 1 |\n";
    core.near.mockResolvedValue(moved);
    const caret = quoted.indexOf("| 1") + 2;
    const editor = view(caret, quoted);
    expect(press(editor, "ArrowDown")).toBe(true);
    await settle();
    expect(core.near).toHaveBeenCalledWith(quoted, caret, { kind: "rowMove", from: 0, to: 1 });
    const head = editor.state.selection.main.head;
    expect(editor.state.sliceDoc(head, head + 1)).toBe("1");
  });

  it("moves a column with Alt and a side arrow, and sorts and recasts by the column of the caret", async () => {
    core.near.mockResolvedValue(TEXT);
    const caret = TEXT.indexOf("Edad") + 1;
    const editor = view(caret);
    press(editor, "ArrowLeft");
    await settle();
    expect(core.near).toHaveBeenLastCalledWith(TEXT, bytes(TEXT.slice(0, TEXT.indexOf("Nombre"))), {
      kind: "columnMove",
      from: 1,
      to: 0,
    });
    // The caret followed the column it moved, so the rest is asked from a fresh one in the second column.
    const other = view(caret);
    regrid(other, { kind: "sort", down: true });
    await settle();
    expect(core.near).toHaveBeenLastCalledWith(TEXT, expect.any(Number), {
      kind: "sort",
      column: 1,
      down: true,
    });
    regrid(other, { kind: "recast", word: "number" });
    await settle();
    expect(core.near).toHaveBeenLastCalledWith(TEXT, expect.any(Number), {
      kind: "recast",
      column: 1,
      word: "number",
    });
  });

  it("keeps Alt and an arrow inside a table, even on the head, and leaves them alone outside it", async () => {
    expect(press(view(TEXT.indexOf("Nombre") + 1), "ArrowDown")).toBe(true);
    expect(press(view(TEXT.indexOf("Toto") + 1), "ArrowDown")).toBe(true);
    expect(press(view(1), "ArrowDown")).toBe(false);
    await settle();
    expect(core.near).not.toHaveBeenCalled();
  });

  it("asks a Mac for Control too before a column moves, since Option and a side arrow is a word there", () => {
    const sides = gridKeys.filter((one) => one.key?.endsWith("Left") || one.key?.endsWith("Right"));
    expect(sides.map((one) => one.mac)).toEqual(["Ctrl-Alt-ArrowLeft", "Ctrl-Alt-ArrowRight"]);
  });

  it("puts nothing back when the text changed while the core was asked", async () => {
    const editor = view(TEXT.indexOf("Rina") + 1);
    press(editor, "ArrowDown");
    editor.dispatch({ changes: { from: 0, insert: "¡" } });
    await settle();
    expect(editor.state.doc.toString()).toBe(`¡${TEXT}`);
  });

  it("says why when the core refuses, and leaves the text as it was", async () => {
    core.near.mockRejectedValue("moved");
    const failed = vi.fn();
    const editor = view(TEXT.indexOf("Rina") + 1, TEXT, failed);
    press(editor, "ArrowDown");
    await settle();
    expect(failed).toHaveBeenCalledWith(new Error("refused moved"));
    expect(editor.state.doc.toString()).toBe(TEXT);
  });
});

describe("the table's own menu in the editor's bar", () => {
  const menuShown = (bar: HTMLElement) =>
    Array.from(bar.querySelectorAll<HTMLElement>(".cm-bar-menu")).find((one) => !one.hidden);

  const item = (menu: HTMLElement | undefined, said: string) =>
    Array.from(menu?.querySelectorAll<HTMLButtonElement>("[role=menuitem]") ?? []).find(
      (one) => one.textContent === said,
    );

  it("sorts by the column of the caret and opens the types, each written through the core", async () => {
    const { barred } = await import("tisty-editor/edit");
    core.near.mockResolvedValue(TEXT);
    const editor = view(TEXT.indexOf("Edad") + 1);
    const { bar, follow } = barred(editor);
    follow(editor);
    bar.querySelector<HTMLButtonElement>(".cm-bar-kind")?.click();
    item(menuShown(bar), "Sort descending")?.click();
    await settle();
    expect(core.near).toHaveBeenLastCalledWith(TEXT, expect.any(Number), {
      kind: "sort",
      column: 1,
      down: true,
    });

    bar.querySelector<HTMLButtonElement>(".cm-bar-kind")?.click();
    item(menuShown(bar), "Type ▸")?.click();
    const types = menuShown(bar);
    expect(types?.querySelectorAll("[role=menuitem]")).toHaveLength(14);
    item(types, "Number")?.click();
    await settle();
    expect(core.near).toHaveBeenLastCalledWith(TEXT, expect.any(Number), {
      kind: "recast",
      column: 1,
      word: "number",
    });
    editor.destroy();
  });
});
