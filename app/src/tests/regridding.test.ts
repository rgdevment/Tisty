import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView, keymap, runScopeHandlers } from "@codemirror/view";
import { act } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { gridKeys, regrid } from "../ui/regridding";

const core = vi.hoisted(() => ({
  around: vi.fn<(body: string, at: number) => Promise<[number, number] | null>>(),
  edit: vi.fn<(body: string, at: [number, number], ...edits: unknown[]) => Promise<string>>(),
}));

vi.mock("../tables", () => ({ tableAround: core.around, tableEdit: core.edit }));

// A word with an accent sits before the table, so its place in bytes and in characters differ.
const TEXT = "Señal\n\n| Nombre | Edad |\n| --- | --- |\n| Rina | 4 |\n| Toto | 7 |\n\nFin";
const MOVED = "Señal\n\n| Nombre | Edad |\n| --- | --- |\n| Toto | 7 |\n| Rina | 4 |\n\nFin";

const view = (head: number) =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc: TEXT,
      selection: EditorSelection.single(head),
      extensions: [markdown({ base: markdownLanguage }), keymap.of(gridKeys)],
    }),
  });

const press = (editor: EditorView, key: string) =>
  runScopeHandlers(editor, new KeyboardEvent("keydown", { key, altKey: true }), "editor");

const settle = () => act(async () => {});

beforeEach(() => {
  core.around.mockReset().mockResolvedValue([8, 72]);
  core.edit.mockReset().mockResolvedValue(MOVED);
});

describe("a table changed from the text editor", () => {
  it("moves the row under the caret through the core, and the caret goes with it", async () => {
    const editor = view(TEXT.indexOf("Rina") + 1);
    expect(press(editor, "ArrowDown")).toBe(true);
    await settle();
    expect(core.around).toHaveBeenCalledWith(
      TEXT,
      new TextEncoder().encode(TEXT.slice(0, 7)).length,
    );
    expect(core.edit).toHaveBeenCalledWith(TEXT, [8, 72], { kind: "rowMove", from: 0, to: 1 });
    expect(editor.state.doc.toString()).toBe(MOVED);
    const head = editor.state.selection.main.head;
    expect(editor.state.sliceDoc(head, head + 4)).toBe("Rina");
  });

  it("moves a column with Alt and a side arrow, and sorts and retypes by the column of the caret", async () => {
    const editor = view(TEXT.indexOf("Edad") + 1);
    press(editor, "ArrowLeft");
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(TEXT, [8, 72], {
      kind: "columnMove",
      from: 1,
      to: 0,
    });

    core.edit.mockResolvedValue(TEXT);
    const other = view(TEXT.indexOf("Edad") + 1);
    regrid(other, { kind: "sort", down: true });
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(TEXT, [8, 72], {
      kind: "sort",
      column: 1,
      down: true,
    });
    regrid(other, { kind: "retype", words: "number sum" });
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(TEXT, [8, 72], {
      kind: "retype",
      column: 1,
      words: "number sum",
    });
  });

  it("keeps Alt and an arrow inside a table, even on the head, and leaves them alone outside it", async () => {
    const head = view(TEXT.indexOf("Nombre") + 1);
    expect(press(head, "ArrowDown")).toBe(true);
    const last = view(TEXT.indexOf("Toto") + 1);
    expect(press(last, "ArrowDown")).toBe(true);
    const away = view(1);
    expect(press(away, "ArrowDown")).toBe(false);
    await settle();
    expect(core.around).not.toHaveBeenCalled();
  });

  it("puts nothing back when the text changed while the core was asked", async () => {
    const editor = view(TEXT.indexOf("Rina") + 1);
    press(editor, "ArrowDown");
    editor.dispatch({ changes: { from: 0, insert: "¡" } });
    await settle();
    expect(editor.state.doc.toString()).toBe(`¡${TEXT}`);
  });

  it("does nothing when the core finds no table there", async () => {
    core.around.mockResolvedValue(null);
    const editor = view(TEXT.indexOf("Rina") + 1);
    press(editor, "ArrowDown");
    await settle();
    expect(core.edit).not.toHaveBeenCalled();
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
    const { barred } = await import("../ui/barring");
    core.edit.mockResolvedValue(TEXT);
    const editor = view(TEXT.indexOf("Edad") + 1);
    const { bar, follow } = barred(editor);
    follow(editor);
    bar.querySelector<HTMLButtonElement>(".cm-bar-kind")?.click();
    item(menuShown(bar), "Sort descending")?.click();
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(TEXT, [8, 72], {
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
    expect(core.edit).toHaveBeenLastCalledWith(TEXT, [8, 72], {
      kind: "retype",
      column: 1,
      words: "number",
    });
    editor.destroy();
  });
});
