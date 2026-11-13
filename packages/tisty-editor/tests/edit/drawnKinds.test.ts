import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";
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
  it("draws ideas, code and live widgets, and leaves plain words to be written", () => {
    const doc =
      "> [!IDEA] one\n> words\n\n```js\nlet a = 1;\n```\n\n```widget\n<p>hi</p>\n```\n\nplain";
    const view = viewOf(
      doc,
      hostWith(() => undefined),
    );
    expect(sources(view)).toEqual([
      "> [!IDEA] one\n> words",
      "```js\nlet a = 1;\n```",
      "```widget\n<p>hi</p>\n```",
    ]);
  });

  it("opens a drawn block to its Markdown with Enter once it is taken whole", () => {
    const doc = "```js\nlet a = 1;\n```\n\nafter";
    const view = viewOf(
      doc,
      hostWith(() => undefined),
    );
    view.dispatch({ selection: { anchor: 0, head: doc.indexOf("\n\nafter") } });
    const enter = livelyKeys.filter((one) => one.key === "Enter");
    expect(enter.some((one) => one.run?.(view))).toBe(true);
    expect(view.dom.querySelector(".cm-rich")).toBeNull();
  });

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
