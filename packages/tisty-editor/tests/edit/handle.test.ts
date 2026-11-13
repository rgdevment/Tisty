import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it } from "vitest";
import { blocking, chosen } from "../../src/edit/blocks";
import { gripping } from "../../src/edit/handle";
import { GRAMMAR } from "../../src/edit/pairing";

const DOC = "one\n\ntwo\n\nthree";
const LINE_PX = 20;

let opened: EditorView[] = [];

afterEach(() => {
  for (const one of opened) one.destroy();
  opened = [];
  document.body.innerHTML = "";
});

// jsdom lays nothing out, so each line stands where its number puts it.
const laidOut = (view: EditorView) => {
  const lineTop = (at: number) => (view.state.doc.lineAt(at).number - 1) * LINE_PX;
  view.posAtCoords = ((coords: { y: number }) => {
    const number = Math.min(view.state.doc.lines, Math.max(1, Math.floor(coords.y / LINE_PX) + 1));
    return view.state.doc.line(number).from;
  }) as EditorView["posAtCoords"];
  view.coordsAtPos = ((at: number) => {
    const top = lineTop(at);
    return { top, bottom: top + LINE_PX, left: 0, right: 0 };
  }) as EditorView["coordsAtPos"];
  return view;
};

const viewOf = (doc = DOC) => {
  const parent = document.createElement("div");
  document.body.append(parent);
  const view = new EditorView({
    parent,
    state: EditorState.create({ doc, extensions: [GRAMMAR, blocking, gripping] }),
  });
  opened.push(view);
  return laidOut(view);
};

const mouse = (target: EventTarget, type: string, y: number) =>
  target.dispatchEvent(new MouseEvent(type, { bubbles: true, button: 0, clientX: 4, clientY: y }));

const gripOf = (view: EditorView) => {
  const grip = view.dom.querySelector<HTMLElement>(".cm-grip");
  if (!grip) throw new Error("no grip");
  return grip;
};

describe("the grip beside a block", () => {
  it("waits hidden and shows beside the block under the pointer", () => {
    const view = viewOf();
    const grip = gripOf(view);
    expect(grip.hidden).toBe(true);
    mouse(view.dom, "mousemove", 2 * LINE_PX + 5);
    expect(grip.hidden).toBe(false);
    expect(grip.style.top).toBe(`${2 * LINE_PX}px`);
  });

  it("chooses the block when pressed and let go in place", () => {
    const view = viewOf();
    mouse(view.dom, "mousemove", 2 * LINE_PX + 5);
    mouse(gripOf(view), "mousedown", 2 * LINE_PX + 5);
    mouse(document, "mouseup", 2 * LINE_PX + 5);
    const { from, to } = view.state.selection.main;
    expect(view.state.sliceDoc(from, to)).toBe("two");
    expect(view.state.field(chosen)).toBe(true);
  });

  it("drags the block to where it is let go, past the one below", () => {
    const view = viewOf();
    mouse(view.dom, "mousemove", 5);
    mouse(gripOf(view), "mousedown", 5);
    mouse(document, "mousemove", 4 * LINE_PX + 15);
    const drop = view.dom.querySelector<HTMLElement>(".cm-drop");
    expect(drop?.hidden).toBe(false);
    mouse(document, "mouseup", 4 * LINE_PX + 15);
    expect(view.state.doc.toString()).toBe("two\n\nthree\n\none");
  });

  it("hides again when the pointer leaves the editor, and when the words change", () => {
    const view = viewOf();
    const grip = gripOf(view);
    mouse(view.dom, "mousemove", 5);
    view.dom.dispatchEvent(new MouseEvent("mouseleave"));
    expect(grip.hidden).toBe(true);
    mouse(view.dom, "mousemove", 5);
    view.dispatch({ changes: { from: 0, insert: "x" } });
    expect(grip.hidden).toBe(true);
  });
});
