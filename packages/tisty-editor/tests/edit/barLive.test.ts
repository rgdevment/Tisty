import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";
import { movable } from "../../src/edit/barMoving";
import { barred } from "../../src/edit/barring";
import type { Hosted } from "../../src/edit/hosted";
import { hosted } from "../../src/edit/hosted";
import { lively } from "../../src/edit/living";
import { GRAMMAR } from "../../src/edit/pairing";
import { visual } from "../../src/edit/seeing";

let opened: EditorView[] = [];

afterEach(() => {
  for (const one of opened) one.destroy();
  opened = [];
  document.body.innerHTML = "";
});

const HOST: Hosted = {
  say: (word) => word,
  tableEditNear: () => Promise.reject(new Error("no table here")),
  fromHtml: () => Promise.resolve(""),
  fromText: () => Promise.resolve(null),
};

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

describe("the bar of the Markdown view", () => {
  it("opens over a word taken with a double press", () => {
    const view = viewOf("one two", HOST, 4);
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

describe("the Markdown view", () => {
  it("shows every mark even when the marks were last left hidden elsewhere", () => {
    localStorage.setItem("tisty.marksShown", "0");
    const view = viewOf("# Title", HOST);
    expect(view.contentDOM.textContent).toBe("# Title");
    localStorage.removeItem("tisty.marksShown");
  });

  it("shows every mark as it is written", () => {
    const view = viewOf("# Title\n\n**bold** and [a link](https://x.y)", HOST);
    expect(view.dom.classList.contains("cm-plain")).toBe(true);
    expect(view.contentDOM.textContent).toContain("**bold**");
    expect(view.contentDOM.textContent).toContain("](https://x.y)");
  });
});
