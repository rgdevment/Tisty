import type { EditorView } from "@codemirror/view";
import type { Editing } from "tisty-editor/edit";
import { afterEach, describe, expect, it, vi } from "vitest";
import { edited } from "../ui/hosting";
import { livelyChoices } from "../ui/livelyMenu";

const clipboard = vi.hoisted(() => ({ written: [] as string[] }));

vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  copied: (text: string) => {
    clipboard.written.push(text);
    return Promise.resolve();
  },
}));

let opened: Editing[] = [];

afterEach(() => {
  for (const one of opened) one.leave();
  opened = [];
  clipboard.written = [];
  document.body.innerHTML = "";
});

const viewOf = (doc: string, anchor: number, head = anchor) => {
  const page = document.createElement("article");
  page.className = "page";
  const block = document.createElement("p");
  page.append(block);
  document.body.append(page);
  const editing = edited(
    block,
    doc,
    () => {},
    () => {},
  );
  opened.push(editing);
  editing.view.dispatch({ selection: { anchor, head } });
  return editing.view;
};

const keysOf = (view: EditorView) => livelyChoices(view).map((one) => one.key);
const pick = (view: EditorView, key: string) =>
  livelyChoices(view)
    .find((one) => one.key === key)
    ?.onPick?.();

describe("the menu of the whole-document editor", () => {
  it("offers only the blocks' acts with no words chosen, each with its keys", () => {
    const view = viewOf("one\n\ntwo", 0);
    expect(keysOf(view)).toEqual([
      "block-choose",
      "block-double",
      "block-up",
      "block-down",
      "block-drop",
    ]);
    expect(livelyChoices(view).every((one) => one.key === "block-drop" || one.keys)).toBe(true);
  });

  it("adds cutting, copying and the formats when words are chosen", () => {
    const view = viewOf("one two\n\nthree", 0, 3);
    expect(keysOf(view).slice(0, 3)).toEqual(["cut", "copy", "format-bold"]);
  });

  it("copies the words chosen, and cuts them out", async () => {
    const view = viewOf("one two", 0, 3);
    pick(view, "copy");
    pick(view, "cut");
    await new Promise((done) => setTimeout(done, 0));
    expect(clipboard.written).toEqual(["one", "one"]);
    expect(view.state.doc.toString()).toBe(" two");
  });

  it("wraps the words chosen in the format picked", () => {
    const view = viewOf("one two", 0, 3);
    pick(view, "format-bold");
    expect(view.state.doc.toString()).toBe("**one** two");
  });

  it("chooses, doubles, moves and drops the block under the caret", () => {
    const view = viewOf("one\n\ntwo", 0);
    pick(view, "block-double");
    expect(view.state.doc.toString()).toBe("one\n\none\n\ntwo");
    pick(view, "block-down");
    expect(view.state.doc.toString()).toBe("one\n\ntwo\n\none");
    pick(view, "block-drop");
    expect(view.state.doc.toString()).toBe("one\n\ntwo");
    view.dispatch({ selection: { anchor: 0 } });
    pick(view, "block-choose");
    expect(view.state.selection.main.to).toBe(3);
    pick(view, "block-up");
    expect(view.state.doc.toString()).toBe("one\n\ntwo");
  });
});
