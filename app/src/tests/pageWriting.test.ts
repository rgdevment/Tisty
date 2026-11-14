import { SLASHED, type Slashed } from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { sourceAt, spotOf } from "../ui/pageSpots";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nUno **dos** tres\n\n- primero\n- segundo\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-24"><span data-src="8-12">Uno </span><strong><span data-src="14-17">dos</span></strong><span data-src="19-24"> tres</span></p>' +
  '<ul data-at="26-46"><li data-at="26-35"><span data-src="28-35">primero</span></li>' +
  '<li data-at="36-45"><span data-src="38-45">segundo</span></li></ul>';

let writing: PageWriting | null = null;

// The core draws the page again after each change, every place past it moved along.
const moved = (html: string, past: number, by: number) =>
  html.replace(/data-(at|src)="(\d+)-(\d+)"/g, (_, kind: string, from: string, to: string) => {
    const shift = (at: number) => (at > past ? at + by : at);
    return `data-${kind}="${shift(Number(from))}-${shift(Number(to))}"`;
  });

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const pageOf = (html = PLACED) => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
  return document;
};

const piece = (said: string) =>
  Array.from(document.querySelectorAll("[data-src]")).find((one) => one.textContent === said)
    ?.firstChild as Text;

const caretIn = (said: string, at: number) => document.getSelection()?.collapse(piece(said), at);

const asked = (inputType: string, data: string | null = null) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

const menus: boolean[] = [];
const answers: string[] = [];

const writer = (body = BODY) => {
  const said: string[] = [];
  let text = body;
  menus.length = 0;
  writing = writtenOn(pageOf(), {
    text: () => text,
    write: (next) => {
      text = next;
      said.push(next);
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: (_kind, _at, put) => {
      const answer = answers.shift();
      if (answer !== undefined) put(answer);
    },
    blocks: (_at, slash) => menus.push(slash),
    acts: () => {},
  });
  return said;
};

const block = (key: string) => SLASHED.find((one) => one.key === key) as Slashed;

describe("the places of the page the core draws to be written on", () => {
  it("finds the Markdown under a point in the words, past the marks the page hides", () => {
    pageOf();
    expect(sourceAt(BODY, piece("dos"), 1)).toBe(BODY.indexOf("dos") + 1);
    expect(sourceAt(BODY, piece(" tres"), 0)).toBe(BODY.indexOf(" tres"));
  });

  it("takes the start of the words for a point at the start of their block", () => {
    pageOf();
    const block = document.querySelector('p[data-at="8-24"]') as Element;
    expect(sourceAt(BODY, block, 0)).toBe(BODY.indexOf("Uno"));
  });

  it("lays the caret inside the piece that holds a place in the Markdown", () => {
    pageOf();
    const spot = spotOf(document, BODY, BODY.indexOf("dos") + 2);
    expect(spot?.[0]).toBe(piece("dos"));
    expect(spot?.[1]).toBe(2);
  });
});

describe("writing on the page the core draws", () => {
  it("writes a key into the Markdown where the caret stands", () => {
    const said = writer();
    caretIn("dos", 3);
    asked("insertText", "!");
    expect(said[said.length - 1]).toBe("# Hola\n\nUno **dos!** tres\n\n- primero\n- segundo\n");
  });

  it("starts a paragraph with Enter, and a new item inside a list", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertParagraph");
    expect(said[said.length - 1]?.startsWith("# Hola\n\n\n\nUno")).toBe(true);
    writing?.stop();
    const items = writer();
    caretIn("primero", 7);
    asked("insertParagraph");
    expect(items[items.length - 1]).toContain("- primero\n- \n- segundo");
  });

  it("stands a paragraph in for one just begun, and writes into it", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertParagraph");
    pageOf(moved(PLACED, 7, 2));
    writing?.drawn();
    expect(document.querySelector("[data-ghost]")).not.toBeNull();
    asked("insertText", "x");
    expect(said[said.length - 1]?.startsWith("# Hola\n\nx\n\nUno")).toBe(true);
  });

  it("wraps the words chosen in a format's marks", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("primero"), 0);
    range.setEnd(piece("primero"), 7);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    asked("formatBold");
    expect(said[said.length - 1]).toContain("- **primero**");
  });

  it("writes what a composition ends with, as an accent typed with a dead key", () => {
    const said = writer();
    caretIn("Hola", 4);
    const page = document.querySelector(".page") as Element;
    page.dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true }));
    page.dispatchEvent(new CompositionEvent("compositionend", { bubbles: true, data: "é" }));
    expect(said[said.length - 1]?.startsWith("# Holaé\n")).toBe(true);
  });

  it("takes a change back with undo, and again with redo", () => {
    const said = writer();
    caretIn("dos", 0);
    asked("insertText", "y ");
    asked("historyUndo");
    expect(said[said.length - 1]).toBe(BODY);
    asked("historyRedo");
    expect(said[said.length - 1]).toContain("**y dos**");
  });

  it("takes a format off words already wearing it, and keeps the words chosen before a button", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("dos"), 0);
    range.setEnd(piece("dos"), 3);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    document.dispatchEvent(new Event("selectionchange"));
    document.getSelection()?.removeAllRanges();
    writing?.format("**", "**");
    expect(said[said.length - 1]).toBe("# Hola\n\nUno dos tres\n\n- primero\n- segundo\n");
  });

  it("cuts the words chosen and hands their Markdown", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("Uno "), 0);
    range.setEnd(piece("dos"), 3);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    expect(writing?.chosen()).toBe("Uno **dos");
    writing?.cut();
    expect(said[said.length - 1]).toBe("# Hola\n\n** tres\n\n- primero\n- segundo\n");
  });

  it("keeps the caret it lays through a key, and lets it go with an arrow", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertText", "a");
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "b" }));
    asked("insertText", "b");
    expect(said[said.length - 1]?.startsWith("# Holaab\n")).toBe(true);
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft" }));
    caretIn("Hola", 0);
    asked("insertText", "¡");
    expect(said[said.length - 1]?.startsWith("# ¡Holaab\n")).toBe(true);
  });

  it("leaves tables, drawings and rules to their own hands", () => {
    writer();
    pageOf(`${PLACED}<div class="table" data-at="47-60"></div><hr data-at="61-64">`);
    writing?.drawn();
    expect(document.querySelector(".table")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector("hr")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector("p")?.hasAttribute("contenteditable")).toBe(false);
  });
});

describe("the blocks put on the page written in place", () => {
  it("offers the blocks at a slash on a line just begun, and writes it anywhere else", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertParagraph");
    pageOf(moved(PLACED, 7, 2));
    writing?.drawn();
    asked("insertText", "/");
    expect(menus).toEqual([true]);
    caretIn("dos", 1);
    asked("insertText", "/");
    expect(said[said.length - 1]).toContain("**d/os**");
  });

  it("turns the line the caret stands in into a title, its list mark gone", () => {
    const said = writer();
    caretIn("primero", 3);
    writing?.slashed(block("h2"));
    expect(said[said.length - 1]).toContain("## primero\n- segundo");
  });

  it("puts a table after the block the caret stands in, a blank line apart", () => {
    const said = writer();
    caretIn("dos", 1);
    writing?.slashed(block("table"));
    expect(said[said.length - 1]).toContain("Uno **dos** tres\n\n|  |  |\n| --- | --- |");
  });

  it("puts what the window answers for a diagram as a block of its own", () => {
    const said = writer();
    answers.push("```mermaid\nflowchart LR\n```");
    caretIn("Hola", 4);
    writing?.slashed(block("diagram"));
    const now = said[said.length - 1] ?? "";
    expect(now.startsWith("# Hola\n\n```mermaid\nflowchart LR\n```\n\nUno")).toBe(true);
  });
});

describe("blocks moved with the keys on the page written in place", () => {
  it("moves the item the caret stands in up with Alt and an arrow", () => {
    const said = writer();
    caretIn("segundo", 2);
    document
      .querySelector(".page")
      ?.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true, bubbles: true }),
      );
    expect(said[said.length - 1]?.endsWith("- segundo\n- primero\n")).toBe(true);
  });
});
