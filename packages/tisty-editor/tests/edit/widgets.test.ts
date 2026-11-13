import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";
import { type Hosted, hosted, type TableEdit } from "../../src/edit/hosted";
import { lively, livelyKeys, viewTurned } from "../../src/edit/living";
import { GRAMMAR } from "../../src/edit/pairing";
import { marksShown, showMarks } from "../../src/edit/seeing";

const TABLE = "| a | b |\n| --- | --- |\n| 1 | 2 |";
const DOC = `# T\n\n${TABLE}\n\npara`;

const drawnTable = (types = ["text", "check"]) => `<div class="table">
  <table><colgroup><col><col></colgroup>
    <thead><tr>
      <th data-type="${types[0]}">a<button data-act="column" data-column="0">⋯</button><span class="col-grip"></span></th>
      <th data-type="${types[1]}">b<button data-act="column" data-column="1">⋯</button></th>
    </tr></thead>
    <tbody><tr data-row="0">
      <td><button data-act="row" data-row="0"></button>1</td>
      <td><button data-act="toggle" data-row="0" data-column="1" data-to="1"></button><button data-act="star" data-row="0" data-column="1" data-to="3"></button></td>
    </tr></tbody>
  </table>
  <button data-act="rowAdd">+</button>
</div>`;

let opened: EditorView[] = [];

afterEach(() => {
  for (const one of opened) one.destroy();
  opened = [];
  document.body.innerHTML = "";
});

const hostOf = (types?: string[]) => {
  const edits: TableEdit[] = [];
  const host: Hosted = {
    say: (word) => word,
    tableEditNear: vi.fn((body: string, _at: number, ...made: TableEdit[]) => {
      edits.push(...made);
      return Promise.resolve(`${body}\n`);
    }),
    fromHtml: () => Promise.resolve(""),
    fromText: () => Promise.resolve(null),
    drawBlock: (source, into) => {
      into.innerHTML = source.startsWith("|") ? drawnTable(types) : '<p><img alt="x"></p>';
    },
  };
  return { host, edits };
};

const viewOf = (doc: string, host: Hosted, at = 0) => {
  const parent = document.createElement("div");
  document.body.append(parent);
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc,
      selection: { anchor: at },
      extensions: [GRAMMAR, hosted.of(host), lively],
    }),
  });
  opened.push(view);
  return view;
};

const press = (target: Element, init: MouseEventInit = {}) =>
  target.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0, ...init }));

const settled = () => new Promise((done) => setTimeout(done, 0));

describe("a drawn block in the whole-document editor", () => {
  it("draws a table as the page does and keeps its Markdown as the source", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    const box = view.dom.querySelector<HTMLElement>(".cm-rich");
    expect(box?.querySelector("table")).not.toBeNull();
    expect(box?.dataset.source).toBe(TABLE);
  });

  it("takes the whole table with a press beside its cells", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    const box = view.dom.querySelector<HTMLElement>(".cm-rich");
    if (box) press(box);
    const { from, to } = view.state.selection.main;
    expect(view.state.sliceDoc(from, to)).toBe(TABLE);
  });

  it("takes the table first on a Backspace after it, so one key never wipes it", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host, DOC.indexOf("para"));
    const back = livelyKeys.find((one) => one.key === "Backspace" && one.run?.(view));
    expect(back).toBeDefined();
    const { from, to } = view.state.selection.main;
    expect(view.state.sliceDoc(from, to)).toBe(TABLE);
  });

  it("opens a picture's Markdown on a double press, and draws it again once the caret leaves", () => {
    const { host } = hostOf();
    const view = viewOf("![x](a.png)\n\nafter", host);
    const box = view.dom.querySelector<HTMLElement>(".cm-rich");
    if (box) press(box, { detail: 2 });
    expect(view.dom.querySelector(".cm-rich")).toBeNull();
    view.dispatch({ selection: { anchor: view.state.doc.length } });
    expect(view.dom.querySelector(".cm-rich")).not.toBeNull();
  });

  it("never opens a table's Markdown on a double press", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    const box = view.dom.querySelector<HTMLElement>(".cm-rich");
    if (box) press(box, { detail: 2 });
    expect(view.dom.querySelector(".cm-rich table")).not.toBeNull();
  });
});

describe("a table written in place", () => {
  it("writes a cell from its words, as the core changes it", async () => {
    const { host, edits } = hostOf();
    const view = viewOf(DOC, host);
    const cell = view.dom.querySelector<HTMLTableCellElement>(".cm-rich td");
    if (!cell) throw new Error("no cell");
    press(cell);
    expect(cell.isContentEditable || cell.contentEditable === "plaintext-only").toBe(true);
    expect(cell.textContent).toBe("1");
    cell.textContent = "9";
    cell.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settled();
    expect(edits).toEqual([{ kind: "cell", row: 0, column: 0, said: "9" }]);
    expect(view.state.doc.toString()).toBe(`${DOC}\n`);
  });

  it("leaves the cell as it was on Escape", () => {
    const { host, edits } = hostOf();
    const view = viewOf(DOC, host);
    const cell = view.dom.querySelector<HTMLTableCellElement>(".cm-rich td");
    if (!cell) throw new Error("no cell");
    press(cell);
    cell.textContent = "9";
    cell.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(edits).toEqual([]);
    expect(cell.textContent).toBe("1");
  });

  it("renames a column from its head", async () => {
    const { host, edits } = hostOf();
    const view = viewOf(DOC, host);
    const head = view.dom.querySelector<HTMLTableCellElement>(".cm-rich th");
    if (!head) throw new Error("no head");
    press(head);
    head.textContent = "name";
    head.dispatchEvent(new Event("blur"));
    await settled();
    expect(edits).toEqual([{ kind: "rename", column: 0, said: "name" }]);
  });

  it("switches a check and lights a star with one press", async () => {
    const { host, edits } = hostOf();
    const view = viewOf(DOC, host);
    const toggle = view.dom.querySelector('.cm-rich [data-act="toggle"]');
    const star = view.dom.querySelector('.cm-rich [data-act="star"]');
    if (toggle) press(toggle);
    if (star) press(star);
    await settled();
    expect(edits).toEqual([
      { kind: "cell", row: 0, column: 1, said: "[x]" },
      { kind: "cell", row: 0, column: 1, said: "3" },
    ]);
  });

  it("picks a day in a date column with the system's calendar", async () => {
    const { host, edits } = hostOf(["date", "text"]);
    const view = viewOf(DOC, host);
    const cell = view.dom.querySelector<HTMLTableCellElement>(".cm-rich td");
    if (!cell) throw new Error("no cell");
    press(cell);
    const input = cell.querySelector<HTMLInputElement>("input[type=date]");
    if (!input) throw new Error("no calendar");
    input.value = "2026-11-20";
    input.dispatchEvent(new Event("change"));
    await settled();
    expect(edits).toEqual([{ kind: "cell", row: 0, column: 0, said: "2026-11-20" }]);
  });

  it("puts the cell back when the calendar is left without a day", () => {
    const { host, edits } = hostOf(["date", "text"]);
    const view = viewOf(DOC, host);
    const cell = view.dom.querySelector<HTMLTableCellElement>(".cm-rich td");
    if (!cell) throw new Error("no cell");
    press(cell);
    cell
      .querySelector("input")
      ?.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(edits).toEqual([]);
    expect(cell.querySelector("input")).toBeNull();
  });

  it("adds a row at the end", async () => {
    const { host, edits } = hostOf();
    const view = viewOf(DOC, host);
    const add = view.dom.querySelector('.cm-rich [data-act="rowAdd"]');
    if (add) press(add);
    await settled();
    expect(edits).toEqual([{ kind: "rowAdd", at: 1 }]);
  });

  it("offers a column's menu and writes what is picked in it", async () => {
    const { host, edits } = hostOf();
    const view = viewOf(DOC, host);
    const more = view.dom.querySelector('.cm-rich [data-act="column"]');
    if (more) press(more);
    const menu = view.dom.querySelector(".cm-table-menu");
    const words = Array.from(menu?.querySelectorAll("button") ?? [], (one) => one.textContent);
    expect(words).toContain("tableSortAsc");
    expect(words).toContain("tableColumnMoveRight");
    expect(words).not.toContain("tableColumnMoveLeft");
    expect(words.find((one) => one?.startsWith("tableTypeText"))).toContain("✓");
    const remove = Array.from(menu?.querySelectorAll("button") ?? []).find(
      (one) => one.textContent === "tableColumnRemove",
    );
    if (remove) press(remove);
    await settled();
    expect(edits).toEqual([{ kind: "columnRemove", column: 0 }]);
    expect(view.dom.querySelector(".cm-table-menu")).toBeNull();
  });

  it("closes a table's menu on Escape without choosing the block under the caret", async () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host, DOC.indexOf("para"));
    const more = view.dom.querySelector('.cm-rich [data-act="column"]');
    if (more) press(more);
    await new Promise((done) => setTimeout(done, 1));
    const leaving = new KeyboardEvent("keydown", {
      key: "Escape",
      bubbles: true,
      cancelable: true,
    });
    view.contentDOM.dispatchEvent(leaving);
    expect(view.dom.querySelector(".cm-table-menu")).toBeNull();
    expect(leaving.defaultPrevented).toBe(true);
    expect(view.dom.querySelector(".cm-chosen")).toBeNull();
  });

  it("offers a row's menu, and a press elsewhere closes it", async () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    const grip = view.dom.querySelector('.cm-rich [data-act="row"]');
    if (grip) press(grip);
    const words = Array.from(
      view.dom.querySelectorAll(".cm-table-menu button"),
      (one) => one.textContent,
    );
    expect(words).toEqual(["tableRowAbove", "tableRowBelow", "tableRowRemove"]);
    await new Promise((done) => setTimeout(done, 1));
    document.body.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(view.dom.querySelector(".cm-table-menu")).toBeNull();
  });

  it("widens a column dragged by its edge, written as dashes in its rule", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    const grip = view.dom.querySelector(".cm-rich .col-grip");
    if (grip) press(grip, { clientX: 0 });
    document.dispatchEvent(new MouseEvent("mousemove", { clientX: 120, buttons: 1 }));
    document.dispatchEvent(new MouseEvent("mouseup"));
    expect(view.state.doc.toString()).toContain("| ------------ | --- |");
  });

  it("ends a widening let go outside the page at the next move with no button", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    const grip = view.dom.querySelector(".cm-rich .col-grip");
    if (grip) press(grip, { clientX: 0 });
    document.dispatchEvent(new MouseEvent("mousemove", { clientX: 120, buttons: 1 }));
    document.dispatchEvent(new MouseEvent("mousemove", { clientX: 300, buttons: 0 }));
    document.dispatchEvent(new MouseEvent("mousemove", { clientX: 500, buttons: 1 }));
    expect(view.state.doc.toString()).toContain("| ------------ | --- |");
    expect(document.documentElement.classList.contains("widening")).toBe(false);
  });

  it("renames a typed column by its name alone, the core keeping its type", async () => {
    const { host, edits } = hostOf();
    const view = viewOf("| a {date} | b |\n| --- | --- |\n| 1 | 2 |", host);
    const head = view.dom.querySelector<HTMLTableCellElement>(".cm-rich th");
    if (!head) throw new Error("no head");
    press(head);
    expect(head.textContent).toBe("a");
    head.textContent = "day";
    head.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settled();
    expect(edits).toEqual([{ kind: "rename", column: 0, said: "day" }]);
  });

  it("keeps a day typed in the calendar when its field is left", async () => {
    const { host, edits } = hostOf(["date", "text"]);
    const view = viewOf(DOC, host);
    const cell = view.dom.querySelector<HTMLTableCellElement>(".cm-rich td");
    if (!cell) throw new Error("no cell");
    press(cell);
    const input = cell.querySelector<HTMLInputElement>("input[type=date]");
    if (!input) throw new Error("no calendar");
    input.value = "2026-12-01";
    input.dispatchEvent(new Event("blur"));
    await settled();
    expect(edits).toEqual([{ kind: "cell", row: 0, column: 0, said: "2026-12-01" }]);
  });
});

describe("the views of the whole-document editor", () => {
  it("opens on the preview and turns to plain Markdown from its tab", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    const tabs = Array.from(view.dom.querySelectorAll<HTMLElement>(".cm-view-tab"));
    expect(tabs.map((one) => one.textContent)).toEqual(["viewPreview", "viewMarkdown"]);
    expect(tabs[0].getAttribute("aria-pressed")).toBe("true");
    tabs[1].click();
    expect(view.state.field(marksShown)).toBe(true);
    expect(view.dom.classList.contains("cm-plain")).toBe(true);
    expect(view.dom.querySelector(".cm-rich")).toBeNull();
    tabs[0].click();
    expect(view.state.field(marksShown)).toBe(false);
  });

  it("turns the view for this document only, leaving the next editor as it starts", () => {
    localStorage.setItem("tisty.marksShown", "0");
    const { host } = hostOf();
    const view = viewOf(DOC, host);
    viewTurned(view);
    expect(view.state.field(marksShown)).toBe(true);
    expect(localStorage.getItem("tisty.marksShown")).toBe("0");
  });

  it("deletes a letter next to a table in the Markdown view, never taking the table", () => {
    const { host } = hostOf();
    const view = viewOf(DOC, host, DOC.indexOf("para"));
    view.dispatch({ effects: showMarks.of(true) });
    const took = livelyKeys.filter((one) => one.key === "Backspace").some((one) => one.run?.(view));
    expect(took).toBe(false);
  });

  it("never lets the caret stand before the hidden properties", () => {
    const { host } = hostOf();
    const doc = "---\nlang: es\n---\n\n# T";
    const view = viewOf(doc, host);
    view.dispatch({ selection: { anchor: 0 } });
    expect(view.state.selection.main.head).toBe(doc.indexOf("\n\n# T") + 1);
    view.dispatch({ effects: showMarks.of(true) });
    view.dispatch({ selection: { anchor: 0 } });
    expect(view.state.selection.main.head).toBe(0);
  });

  it("keeps the properties at the top out of the preview, and shows them in Markdown", () => {
    const { host } = hostOf();
    const view = viewOf("---\nlang: es\n---\n\n# T", host);
    const lines = () => Array.from(view.dom.querySelectorAll(".cm-line"), (one) => one.textContent);
    expect(lines()).not.toContain("lang: es");
    view.dispatch({ effects: showMarks.of(true) });
    expect(lines()).toContain("lang: es");
  });
});
