import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView, keymap, runScopeHandlers } from "@codemirror/view";
import { retable, retabled, tableAt, tableKeys, widened } from "tisty-editor/edit";
import { describe, expect, it } from "vitest";
import { t } from "../locales";
import { barred } from "../ui/barring";
import { gripped, widening } from "../ui/widening";

const text = "Intro\n\n| Name | Age |\n|------|:---:|\n| Rina | 4 |\n| Toto | 7 |\n\nAfter";
const at = (said: string) => text.indexOf(said) + 1;
const rows = (said: string) => said.split("\n").slice(2, -2);

const view = (doc: string, from: number, separator = "\n") =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc,
      selection: EditorSelection.single(from),
      extensions: [
        EditorState.lineSeparator.of(separator),
        markdown({ base: markdownLanguage }),
        keymap.of(tableKeys),
      ],
    }),
  });

const chosen = (editor: EditorView) =>
  editor.state.sliceDoc(editor.state.selection.main.from, editor.state.selection.main.to);

const press = (editor: EditorView, key: string, shiftKey = false) =>
  runScopeHandlers(editor, new KeyboardEvent("keydown", { key, shiftKey }), "editor");

describe("a table written in the block", () => {
  it("finds the table the caret is in, and nothing outside one", () => {
    expect(tableAt(text, at("Rina"))?.rows).toHaveLength(4);
    expect(tableAt(text, 2)).toBeNull();
    expect(tableAt("a | b\nnot a rule\n1 | 2", 1)).toBeNull();
    expect(retabled("no table", 2, "rowBelow")).toBeNull();
  });

  it("adds an empty row above or below, never above the head", () => {
    expect(rows(retabled(text, at("Rina"), "rowAbove")?.text ?? "")).toEqual([
      "| Name | Age |",
      "|------|:---:|",
      "|  |  |",
      "| Rina | 4 |",
      "| Toto | 7 |",
    ]);
    expect(rows(retabled(text, at("Name"), "rowBelow")?.text ?? "")[2]).toBe("|  |  |");
    expect(retabled(text, at("Name"), "rowAbove")).toBeNull();
  });

  it("adds a column on either side, with a rule of its own and every other dash kept", () => {
    expect(rows(retabled(text, at("Rina"), "columnRight")?.text ?? "")).toEqual([
      "| Name |  | Age |",
      "|------| --- |:---:|",
      "| Rina |  | 4 |",
      "| Toto |  | 7 |",
    ]);
    expect(rows(retabled(text, at("Rina"), "columnLeft")?.text ?? "")[1]).toBe(
      "| --- |------|:---:|",
    );
  });

  it("aligns a column with colons around the same dashes, and the same choice again clears it", () => {
    expect(rows(retabled(text, at("Rina"), "alignRight")?.text ?? "")[1]).toBe("|------:|:---:|");
    expect(rows(retabled(text, at("Rina"), "alignLeft")?.text ?? "")[1]).toBe("|:------|:---:|");
    expect(rows(retabled(text, text.indexOf("4"), "alignCenter")?.text ?? "")[1]).toBe(
      "|------|---|",
    );
  });

  it("removes a row or a column, but never the head, the rule or the last column", () => {
    expect(rows(retabled(text, at("Rina"), "rowRemove")?.text ?? "")).toEqual([
      "| Name | Age |",
      "|------|:---:|",
      "| Toto | 7 |",
    ]);
    expect(retabled(text, at("Name"), "rowRemove")).toBeNull();
    expect(rows(retabled(text, at("Rina"), "columnRemove")?.text ?? "")).toEqual([
      "| Age |",
      "|:---:|",
      "| 4 |",
      "| 7 |",
    ]);
    expect(retabled("| a |\n|---|\n| 1 |", 2, "columnRemove")).toBeNull();
  });

  it("keeps a table without outer pipes a table when one column is left", () => {
    expect(retabled("a | b\n--- | ---\n1 | 2", 1, "columnRemove")?.text).toBe(
      "| b |\n| --- |\n| 2 |",
    );
    expect(retabled("a | b\n--- | ---\n1 | 2", 1, "columnRight")?.text).toBe(
      "a |  | b\n--- | --- | ---\n1 |  | 2",
    );
  });

  it("walks the cells with Tab and Shift Tab, skipping the rule, and starts a row past the last", () => {
    const editor = view(text, at("Name"));
    press(editor, "Tab");
    expect(
      editor.state.sliceDoc(editor.state.selection.main.from, editor.state.selection.main.to),
    ).toBe("Age");
    press(editor, "Tab");
    expect(
      editor.state.sliceDoc(editor.state.selection.main.from, editor.state.selection.main.to),
    ).toBe("Rina");
    press(editor, "Tab", true);
    expect(
      editor.state.sliceDoc(editor.state.selection.main.from, editor.state.selection.main.to),
    ).toBe("Age");
    editor.dispatch({ selection: { anchor: editor.state.sliceDoc().indexOf("7") } });
    press(editor, "Tab");
    expect(rows(editor.state.sliceDoc()).pop()).toBe("|  |  |");
    editor.destroy();
  });

  it("leaves Tab alone outside a table", () => {
    const editor = view("words", 2);
    expect(press(editor, "Tab")).toBe(false);
    editor.destroy();
  });

  it("walks the right cells in a document with Windows line ends", () => {
    const editor = view(text.split("\n").join("\r\n"), at("Rina"), "\r\n");
    press(editor, "Tab");
    expect(chosen(editor)).toBe("4");
    press(editor, "Tab");
    expect(chosen(editor)).toBe("Toto");
    editor.destroy();
  });

  it("leaves a table shown inside a code block as code", () => {
    const fenced = "```md\n| a | b |\n|---|---|\n| 1 | 2 |\n```";
    const editor = view(fenced, fenced.indexOf("1"));
    expect(press(editor, "Tab")).toBe(false);
    expect(retable(editor, "rowBelow")).toBe(false);
    editor.destroy();
  });

  it("gives a new row the indent of a table that sits in a list", () => {
    const listed = "- item\n\n  | a | b |\n  |---|---|\n  | 1 | 2 |";
    expect(retabled(listed, listed.indexOf("1"), "rowBelow")?.text.split("\n").pop()).toBe(
      "  |  |  |",
    );
  });

  it("changes the table from the bar's menu, which says it is in a table", () => {
    const editor = view(text, at("Rina"));
    const { bar, follow } = barred(editor);
    follow(editor);
    const kind = bar.querySelector<HTMLButtonElement>(".cm-bar-kind");
    expect(kind?.textContent).toContain(t("table"));
    kind?.click();
    const shown = Array.from(bar.querySelectorAll<HTMLElement>(".cm-bar-menu")).find(
      (one) => !one.hidden,
    );
    const below = Array.from(
      shown?.querySelectorAll<HTMLButtonElement>("[role=menuitem]") ?? [],
    ).find((one) => one.textContent === t("tableRowBelow"));
    below?.click();
    expect(rows(editor.state.sliceDoc())[3]).toBe("|  |  |");
    expect(retable(editor, "rowRemove")).toBe(true);
    expect(rows(editor.state.sliceDoc())).toHaveLength(4);
    editor.destroy();
  });

  it("writes a column's width as its dashes, keeping its colons and every other column", () => {
    const span: [number, number] = [text.indexOf("| Name"), text.indexOf("\n\nAfter")];
    expect(rows(widened(text, span, 1, 8) ?? "")[1]).toBe("|------|:--------:|");
    expect(rows(widened(text, span, 0, 4) ?? "")[1]).toBe("|----|:---:|");
    expect(widened(text, span, 0, 6)).toBeNull();
    expect(widened(text, span, 5, 6)).toBeNull();
    expect(widened("just words\nand more", [0, 19], 0, 6)).toBeNull();
  });

  it("drags a head's edge into a width said back in dashes, ten pixels each", () => {
    const page = document.implementation.createHTMLDocument("");
    page.body.innerHTML =
      '<div class="page"><div class="table" data-at="7-80"><table><colgroup><col><col></colgroup><thead><tr><th>Name</th><th>Age</th></tr></thead></table></div></div>';
    gripped(page);
    const asked: [number, number, number][] = [];
    widening(page, () => (at, column, dashes) => asked.push([at[0], column, dashes]));
    const grip = page.querySelectorAll(".col-grip")[1];
    expect(page.querySelectorAll(".col-grip")).toHaveLength(2);
    grip.dispatchEvent(new MouseEvent("pointerdown", { bubbles: true, clientX: 100, button: 0 }));
    expect(page.documentElement.classList.contains("widening")).toBe(true);
    page.dispatchEvent(new MouseEvent("pointermove", { clientX: 220 }));
    expect(page.querySelectorAll("col")[1].style.width).toBe("120px");
    page.dispatchEvent(new MouseEvent("pointerup", { clientX: 220 }));
    expect(asked).toEqual([[7, 1, 12]]);
    expect(page.documentElement.classList.contains("widening")).toBe(false);
  });
});

describe("a table written inside a quote", () => {
  const quoted = "Dicho:\n\n> | Name | Age |\n> | --- | :-: |\n> | Rina | 4 |\n\nAfter";
  const lines = (said: string) => said.split("\n").slice(2, -2);
  const near = (said: string) => quoted.indexOf(said) + 1;

  it("is found past its quote marks, and its rows keep them as they grow and shrink", () => {
    const table = tableAt(quoted, near("Rina"));
    expect(table?.rows.map((one) => one.edges.length - 1)).toEqual([2, 2, 2]);
    expect(lines(retabled(quoted, near("Rina"), "rowBelow")?.text ?? "")).toEqual([
      "> | Name | Age |",
      "> | --- | :-: |",
      "> | Rina | 4 |",
      "> |  |  |",
    ]);
    expect(lines(retabled(quoted, near("Rina"), "columnRight")?.text ?? "")[1]).toBe(
      "> | --- | --- | :-: |",
    );
    expect(lines(retabled(quoted, near("Age"), "columnRemove")?.text ?? "")).toEqual([
      "> | Name |",
      "> | --- |",
      "> | Rina |",
    ]);
    expect(lines(retabled(quoted, near("Rina"), "alignRight")?.text ?? "")[1]).toBe(
      "> | ---: | :-: |",
    );
  });

  it("is walked with Tab past its quote marks", () => {
    const editor = view(quoted, near("Name"));
    runScopeHandlers(editor, new KeyboardEvent("keydown", { key: "Tab" }), "editor");
    expect(chosen(editor)).toBe("Age");
    editor.destroy();
  });

  it("is read past its marks even with no outer pipes", () => {
    const bare = "> Name | Age\n> --- | ---\n> Rina | 4\n";
    expect(tableAt(bare, bare.indexOf("Rina") + 1)?.rows[2].edges).toEqual([1, 7, 10]);
    expect(retabled(bare, bare.indexOf("Rina") + 1, "rowBelow")?.text.split("\n")[3]).toBe(
      ">   |  ",
    );
  });
});
