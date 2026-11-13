import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { blocking, chooseOrLet, livingKeys, moved, movedTo, unitAt } from "../../src/edit/blocks";
import { GRAMMAR } from "../../src/edit/pairing";

const stateOf = (doc: string, at = 0) =>
  EditorState.create({ doc, selection: { anchor: at }, extensions: [GRAMMAR, blocking] });

const after = (doc: string, at: number, way: -1 | 1) => {
  const state = stateOf(doc, at);
  const spec = moved(state, way);
  return spec ? state.update(spec).state.doc.toString() : doc;
};

const viewOf = (doc: string, at = 0) => {
  const parent = document.createElement("div");
  document.body.append(parent);
  return new EditorView({ parent, state: stateOf(doc, at) });
};

const pressed = (view: EditorView, key: string) => {
  const binding = livingKeys.find((one) => one.key === key);
  return binding?.run?.(view) ?? false;
};

describe("a block moves among its neighbours", () => {
  it("a paragraph trades places with the one below and keeps the gap", () => {
    expect(after("one\n\ntwo\n\nthree", 0, 1)).toBe("two\n\none\n\nthree");
    expect(after("one\n\ntwo\n\nthree", 12, -1)).toBe("one\n\nthree\n\ntwo");
  });

  it("the first block goes no higher and the last no lower", () => {
    expect(after("one\n\ntwo", 0, -1)).toBe("one\n\ntwo");
    expect(after("one\n\ntwo", 6, 1)).toBe("one\n\ntwo");
  });

  it("a title takes its section past the next section of its level", () => {
    const doc = "## A\n\na\n\n## B\n\nb";
    expect(after(doc, 0, 1)).toBe("## B\n\nb\n\n## A\n\na");
  });

  it("a title stays inside the section above it", () => {
    const doc = "# Top\n\n## A\n\na\n\n# Next";
    expect(after(doc, 8, 1)).toBe(doc);
  });

  it("a list item trades places with its sibling", () => {
    expect(after("- one\n- two\n- three", 0, 1)).toBe("- two\n- one\n- three");
  });

  it("the caret moves along with the block", () => {
    const state = stateOf("one\n\ntwo", 1);
    const spec = moved(state, 1);
    expect(spec && state.update(spec).state.selection.main.head).toBe(6);
  });
});

describe("a block dragged elsewhere", () => {
  it("lands before the block it was let go on", () => {
    const state = stateOf("one\n\ntwo\n\nthree");
    const unit = unitAt(state, 12);
    const spec = unit && movedTo(state, unit, 0);
    expect(spec && state.update(spec).state.doc.toString()).toBe("three\n\none\n\ntwo");
  });

  it("lands at the end", () => {
    const state = stateOf("one\n\ntwo\n\nthree");
    const unit = unitAt(state, 0);
    const spec = unit && movedTo(state, unit, state.doc.length);
    expect(spec && state.update(spec).state.doc.toString()).toBe("two\n\nthree\n\none");
  });

  it("let go on itself, stays", () => {
    const state = stateOf("one\n\ntwo");
    const unit = unitAt(state, 0);
    expect(unit && movedTo(state, unit, 1)).toBeNull();
  });
});

describe("a block chosen with Escape", () => {
  it("is taken whole, grows by blocks and goes with its gap", () => {
    const view = viewOf("one\n\ntwo\n\nthree", 1);
    expect(chooseOrLet(view)).toBe(true);
    expect(view.state.selection.main).toMatchObject({ from: 0, to: 3 });
    expect(pressed(view, "Shift-ArrowDown")).toBe(true);
    expect(view.state.selection.main).toMatchObject({ from: 0, to: 8 });
    expect(pressed(view, "Backspace")).toBe(true);
    expect(view.state.doc.toString()).toBe("three");
    view.destroy();
  });

  it("is doubled under itself", () => {
    const view = viewOf("one\n\ntwo", 0);
    expect(pressed(view, "Mod-d")).toBe(true);
    expect(view.state.doc.toString()).toBe("one\n\none\n\ntwo");
    view.destroy();
  });

  it("a second Escape lets it go, and keys go back to writing", () => {
    const view = viewOf("one\n\ntwo", 1);
    chooseOrLet(view);
    chooseOrLet(view);
    expect(view.state.selection.main.empty).toBe(true);
    expect(pressed(view, "Backspace")).toBe(false);
    view.destroy();
  });
});
