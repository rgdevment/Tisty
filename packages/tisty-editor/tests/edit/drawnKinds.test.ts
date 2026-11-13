import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { movable } from "../../src/edit/barMoving";
import { barred } from "../../src/edit/barring";
import type { Hosted } from "../../src/edit/hosted";
import { hosted } from "../../src/edit/hosted";
import { lively, livelyKeys } from "../../src/edit/living";
import { GRAMMAR } from "../../src/edit/pairing";
import { visual } from "../../src/edit/seeing";

let opened: EditorView[] = [];

afterEach(() => {
  for (const one of opened) one.destroy();
  opened = [];
  document.body.innerHTML = "";
});

const hostWith = (drawBlock: Hosted["drawBlock"]): Hosted => ({
  say: (word) => word,
  tableEditNear: () => Promise.reject(new Error("no table here")),
  fromHtml: () => Promise.resolve(""),
  fromText: () => Promise.resolve(null),
  drawBlock,
});

const viewOf = (doc: string, host: Hosted, at = 0) => {
  const parent = document.createElement("div");
  document.body.append(parent);
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc,
      selection: { anchor: at },
      extensions: [GRAMMAR, hosted.of(host), visual, lively],
    }),
  });
  opened.push(view);
  return view;
};

const sources = (view: EditorView) =>
  Array.from(view.dom.querySelectorAll<HTMLElement>(".cm-rich"), (one) => one.dataset.source);

describe("the blocks the whole-document editor draws as the page does", () => {
  it("draws ideas and live widgets, and leaves code and plain words to be written", () => {
    const doc =
      '> [!IDEA] one\n> words\n\n```js title="a.js"\nlet a = 1;\n```\n\n```widget\n<p>hi</p>\n```\n\nplain';
    const view = viewOf(
      doc,
      hostWith(() => undefined),
    );
    expect(sources(view)).toEqual(["> [!IDEA] one\n> words", "```widget\n<p>hi</p>\n```"]);
    const code = Array.from(
      view.dom.querySelectorAll(".cm-line.cm-code"),
      (one) => one.textContent,
    );
    expect(code).toEqual(["let a = 1;"]);
    expect(view.dom.querySelector(".cm-code-head")?.textContent).toBe("a.js · js");
    expect(view.dom.querySelector(".cm-code-foot")).not.toBeNull();
  });

  it("shows a page break as a turn of the sheet and hides a comment", () => {
    const view = viewOf(
      "one\n\n---\n\n<!-- note -->\n\ntwo",
      hostWith(() => undefined),
    );
    expect(view.dom.querySelector(".cm-turn .cm-turn-cut")).not.toBeNull();
    expect(view.dom.querySelector(".cm-turn")?.getAttribute("title")).toBe("leafBreak");
    expect(view.dom.textContent).not.toContain("note");
    expect(view.dom.textContent).not.toContain("---");
  });

  it("takes a page break whole on a press", () => {
    const view = viewOf(
      "one\n\n---\n\ntwo",
      hostWith(() => undefined),
    );
    const turn = view.dom.querySelector(".cm-turn");
    turn?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0 }));
    const { from, to } = view.state.selection.main;
    expect(view.state.sliceDoc(from, to)).toBe("---");
  });

  it("never opens a drawn block's Markdown with Enter", () => {
    const doc = "![x](a.png)\n\nafter";
    const view = viewOf(
      doc,
      hostWith(() => undefined),
    );
    view.dispatch({ selection: { anchor: 0, head: doc.indexOf("\n\nafter") } });
    const enter = livelyKeys.filter((one) => one.key === "Enter");
    expect(enter.some((one) => one.run?.(view))).toBe(false);
    expect(view.dom.querySelector(".cm-rich")).not.toBeNull();
  });
});

const DIAGRAMS = "```mermaid\nflowchart LR\n  A --> B\n```\n\n```mmd\npie\n```\n\nafter";

const diagramDrawn: Hosted["drawBlock"] = (source, into) => {
  into.innerHTML = source.includes("pie")
    ? '<figure class="diagram drawn"><svg></svg><div class="diagram-bar"><button data-asked="diagramExport"></button><button data-asked="diagram"></button></div></figure>'
    : '<figure class="diagram drawn"><svg></svg></figure>';
  return undefined;
};

const pressOn = (target: Element | null | undefined, detail = 1) =>
  target?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0, detail }));

describe("a drawn diagram in the whole-document editor", () => {
  it("opens its window on a double press, named by its place among the diagrams", () => {
    const drawnAsked = vi.fn();
    const view = viewOf(DIAGRAMS, { ...hostWith(diagramDrawn), drawnAsked });
    const second = view.dom.querySelectorAll("figure.diagram")[1];
    pressOn(second, 2);
    expect(drawnAsked).toHaveBeenCalledWith({
      kind: "diagram",
      code: "pie\n",
      from: DIAGRAMS.indexOf("```mmd"),
      nth: 1,
    });
    expect(view.dom.querySelectorAll(".cm-rich")).toHaveLength(2);
  });

  it("looks at it large on a single press, once no second press follows", () => {
    vi.useFakeTimers();
    const drawnAsked = vi.fn();
    const view = viewOf(DIAGRAMS, { ...hostWith(diagramDrawn), drawnAsked });
    const first = view.dom.querySelector("figure.diagram");
    pressOn(first);
    expect(drawnAsked).not.toHaveBeenCalled();
    vi.advanceTimersByTime(400);
    vi.useRealTimers();
    expect(drawnAsked).toHaveBeenCalledWith(
      expect.objectContaining({ kind: "zoom", target: first }),
    );
  });

  it("opens its window and its export from the buttons on its bar", () => {
    const drawnAsked = vi.fn();
    const view = viewOf(DIAGRAMS, { ...hostWith(diagramDrawn), drawnAsked });
    const out = view.dom.querySelector('[data-asked="diagramExport"]');
    pressOn(view.dom.querySelector('.diagram-bar [data-asked="diagram"]'));
    pressOn(out);
    expect(drawnAsked).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({ kind: "diagram", nth: 1 }),
    );
    expect(drawnAsked).toHaveBeenNthCalledWith(2, {
      kind: "export",
      code: "pie\n",
      nth: 1,
      by: out,
    });
  });
});

describe("a sticky note in the whole-document editor", () => {
  const NOTE = "> [!IDEA] 2026-11-12 · blue\n> one\n> two\n\nafter";
  beforeEach(() => {
    Range.prototype.getClientRects = () => [] as unknown as DOMRectList;
    Range.prototype.getBoundingClientRect = () => new DOMRect();
  });

  const noteDrawn: Hosted["drawBlock"] = (_, into) => {
    into.innerHTML =
      '<blockquote data-callout="idea"><span class="sticky-pin"></span><p>one two</p></blockquote>';
    return undefined;
  };

  it("is written in place and keeps its first line", () => {
    const view = viewOf(NOTE, hostWith(noteDrawn));
    pressOn(view.dom.querySelector("blockquote p"));
    const words = view.dom.querySelector<HTMLElement>(".cm-idea-words");
    expect(words?.textContent).toBe("one\ntwo");
    expect(view.dom.querySelector(".sticky-pin")).not.toBeNull();
    if (words) words.textContent = "three\n\nfour";
    words?.dispatchEvent(new FocusEvent("blur"));
    expect(view.state.doc.toString()).toBe(
      "> [!IDEA] 2026-11-12 · blue\n> three\n>\n> four\n\nafter",
    );
  });

  it("is left as it was on Escape", () => {
    const view = viewOf(NOTE, hostWith(noteDrawn));
    pressOn(view.dom.querySelector("blockquote p"));
    const words = view.dom.querySelector<HTMLElement>(".cm-idea-words");
    if (words) words.textContent = "gone";
    words?.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(view.state.doc.toString()).toBe(NOTE);
    expect(view.dom.querySelector(".cm-idea-words")).toBeNull();
  });
});

describe("the blocks the whole-document editor lends", () => {
  it("hands back what a drawn block lent once it leaves the screen", () => {
    const given = vi.fn();
    const view = viewOf(
      "```widget\n<p>hi</p>\n```\n\nafter",
      hostWith(() => given),
    );
    view.dispatch({ changes: { from: 0, to: 23, insert: "gone" } });
    expect(given).toHaveBeenCalled();
  });
});

describe("the bar of the whole-document editor", () => {
  it("opens over a word taken with a double press", () => {
    const view = viewOf(
      "one two",
      hostWith(() => undefined),
      4,
    );
    const { bar } = barred(view, false, undefined, undefined, undefined, true);
    expect(bar.classList.contains("folded")).toBe(true);
    view.dispatch({ selection: { anchor: 4, head: 7 } });
    view.contentDOM.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    expect(bar.classList.contains("folded")).toBe(false);
    view.contentDOM.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(bar.classList.contains("folded")).toBe(true);
  });

  it("moves where it is dragged, and the drag is no press on its button", () => {
    const bar = document.createElement("div");
    const handle = document.createElement("button");
    handle.className = "cm-bar-fold";
    const pressed = vi.fn();
    handle.addEventListener("click", pressed);
    bar.append(handle);
    document.body.append(bar);
    const stop = movable(bar);
    handle.dispatchEvent(
      new MouseEvent("mousedown", { bubbles: true, button: 0, clientX: 10, clientY: 10 }),
    );
    document.dispatchEvent(new MouseEvent("mousemove", { clientX: 60, clientY: 40 }));
    document.dispatchEvent(new MouseEvent("mouseup"));
    handle.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(bar.style.position).toBe("fixed");
    expect(bar.style.left).toBe("50px");
    expect(bar.style.top).toBe("30px");
    expect(bar.dataset.moved).toBe("");
    expect(pressed).not.toHaveBeenCalled();
    stop();
  });
});

describe("the preview of the whole document", () => {
  it("never shows a link's address, even with the caret inside it", () => {
    const doc = "see [the vet](https://vet.example) today";
    const view = viewOf(
      doc,
      hostWith(() => undefined),
      doc.indexOf("vet]"),
    );
    expect(view.contentDOM.textContent).toBe("see the vet today");
  });
});
