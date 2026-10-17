import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState, Prec } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { blockKeys } from "../ui/barring";
import { linking, wrapped } from "../ui/formatting";
import { pending, pendingFormat, pendingKeys } from "../ui/pending";
import { visual } from "../ui/seeing";

const view = (doc: string, from = doc.length, to = from) =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc,
      selection: EditorSelection.single(from, to),
      extensions: [
        markdown({ base: markdownLanguage }),
        visual,
        pendingFormat,
        Prec.highest(keymap.of([...pendingKeys, ...blockKeys, linking])),
      ],
    }),
  });

const formatted = (doc: string, from: number, to: number, mark: string, close = mark) => {
  const editor = view(doc, from, to);
  wrapped(editor, mark, close);
  const text = editor.state.sliceDoc();
  editor.destroy();
  return text;
};

const press = (editor: EditorView, key: string) =>
  editor.contentDOM.dispatchEvent(
    new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }),
  );

const type = (editor: EditorView, text: string) => {
  const at = editor.state.selection.main.head;
  editor.dispatch({ changes: { from: at, insert: text }, selection: { anchor: at + text.length } });
};

describe("formats over what is selected", () => {
  it("never wraps a title's, quote's or item's mark, and formats each item on its own", () => {
    expect(formatted("## uno dos", 0, 10, "**")).toBe("## **uno dos**");
    expect(formatted("> uno dos", 0, 9, "**")).toBe("> **uno dos**");
    expect(formatted("- uno\n- dos", 0, 11, "**")).toBe("- **uno**\n- **dos**");
    expect(formatted("1. uno\n2. dos", 0, 13, "*")).toBe("1. *uno*\n2. *dos*");
    expect(formatted("> [!TIP]\n> uno dos", 0, 18, "**")).toBe("> [!TIP]\n> **uno dos**");
    expect(formatted("- uno\n- dos\n- tres", 3, 15, "**")).toBe("- u**no**\n- **dos**\n- **t**res");
  });

  it("joins a format it crosses instead of crossing its marks", () => {
    expect(formatted("uno **dos** tres", 7, 13, "**")).toBe("uno **dos t**res");
    expect(formatted("**a** b **c**", 0, 13, "**")).toBe("**a b c**");
    expect(formatted("un ==dato== y más", 6, 14, "==", "=={green}")).toBe(
      "un ==dato y=={green} más",
    );
  });

  it("takes a format off when all of the selection already has it, item by item", () => {
    expect(formatted("- **uno**\n- **dos**", 0, 19, "**")).toBe("- uno\n- dos");
    expect(formatted("uno `dos` tres", 5, 8, "**")).toBe("uno `dos` tres");
  });

  it("takes a link off when pressed again, and leaves the caret where its address goes", () => {
    expect(formatted("ver [dos](http://x.cl) ya", 5, 8, "[", "]()")).toBe("ver dos ya");
    const editor = view("ver dos", 4, 7);
    wrapped(editor, "[", "]()");
    expect(editor.state.sliceDoc()).toBe("ver [dos]()");
    expect(editor.state.selection.main.head).toBe(10);
    editor.destroy();
  });

  it("leaves no empty link behind when nothing was typed in it", () => {
    const editor = view("ver ");
    wrapped(editor, "[", "]()");
    editor.dispatch({ selection: { anchor: 0 } });
    expect(editor.state.sliceDoc()).toBe("ver ");
    editor.destroy();
  });

  it("steps from a link's words into its address with the right arrow", () => {
    const editor = view("ver ");
    wrapped(editor, "[", "]()");
    type(editor, "dos");
    press(editor, "ArrowRight");
    type(editor, "http://x.cl");
    expect(editor.state.sliceDoc()).toBe("ver [dos](http://x.cl)");
    editor.destroy();
  });
});

describe("a format pressed with nothing selected", () => {
  it("takes off the format the caret touches instead of opening a new one beside it", () => {
    expect(formatted("uno **dos** tres", 11, 11, "**")).toBe("uno dos tres");
    expect(formatted("un ==dato=={green} y", 9, 9, "==", "=={blue}")).toBe("un ==dato=={blue} y");
  });

  it("steps out when pressed again, and goes away when pressed twice with nothing typed", () => {
    const editor = view("uno ");
    wrapped(editor, "**");
    wrapped(editor, "**");
    expect(editor.state.sliceDoc()).toBe("uno ");
    wrapped(editor, "**");
    type(editor, "x ");
    wrapped(editor, "**");
    type(editor, "y");
    expect(editor.state.sliceDoc()).toBe("uno **x** y");
    expect(editor.state.field(pending)).toBeNull();
    editor.destroy();
  });

  it("swaps a waiting pen's colour", () => {
    const editor = view("");
    wrapped(editor, "==");
    type(editor, "hola");
    wrapped(editor, "==", "=={pink}");
    expect(editor.state.sliceDoc()).toBe("==hola=={pink}");
    editor.destroy();
  });

  it("ends at a new line, and Backspace with nothing typed takes both marks", () => {
    const editor = view("- ");
    wrapped(editor, "**");
    type(editor, "hola ");
    press(editor, "Enter");
    expect(editor.state.sliceDoc()).toBe("- **hola**\n- ");
    expect(editor.state.field(pending)).toBeNull();
    editor.destroy();
    const gone = view("uno x");
    gone.dispatch({ selection: { anchor: 4 } });
    wrapped(gone, "**");
    press(gone, "Backspace");
    expect(gone.state.sliceDoc()).toBe("uno x");
    gone.destroy();
  });
});

describe("the keys at the edges of a block", () => {
  it("ends a list, a check list or a quote with a blank line on an empty item", () => {
    for (const [doc, done] of [
      ["- uno\n- ", "- uno\n\n"],
      ["1. uno\n2. ", "1. uno\n\n"],
      ["- [ ] uno\n- [ ] ", "- [ ] uno\n\n"],
      ["> uno\n> ", "> uno\n\n"],
    ]) {
      const editor = view(doc);
      press(editor, "Enter");
      expect(editor.state.sliceDoc()).toBe(done);
      expect(editor.state.selection.main.head).toBe(done.length);
      editor.destroy();
    }
  });

  it("turns a title, an item or a quote back into words with Backspace after its mark", () => {
    for (const [doc, done] of [
      ["# Plan", "Plan"],
      ["- uno", "uno"],
      ["- [x] uno", "uno"],
      ["> dicho", "dicho"],
    ]) {
      const editor = view(doc, doc.length - (done as string).length);
      press(editor, "Backspace");
      expect(editor.state.sliceDoc()).toBe(done);
      editor.destroy();
    }
    const head = view("> [!TIP]", 2);
    press(head, "Backspace");
    expect(head.state.sliceDoc()).toBe("> [!TIP]");
    head.destroy();
  });
});

describe("the smaller keys", () => {
  it("takes Home past a line's mark, and Shift+Home selects up to it", () => {
    const editor = view("- [ ] uno dos");
    press(editor, "Home");
    expect(editor.state.selection.main.head).toBe(6);
    editor.dispatch({ selection: { anchor: 13 } });
    editor.contentDOM.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "Home",
        shiftKey: true,
        bubbles: true,
        cancelable: true,
      }),
    );
    expect(editor.state.selection.main.from).toBe(6);
    expect(editor.state.selection.main.to).toBe(13);
    editor.destroy();
  });

  it("still moves the caret when an arrow leaves a format with nothing typed", () => {
    const editor = view("uno dos", 4);
    wrapped(editor, "**");
    press(editor, "ArrowRight");
    expect(editor.state.sliceDoc()).toBe("uno dos");
    expect(editor.state.selection.main.head).toBe(5);
    wrapped(editor, "*");
    press(editor, "ArrowLeft");
    expect(editor.state.sliceDoc()).toBe("uno dos");
    expect(editor.state.selection.main.head).toBe(4);
    editor.destroy();
  });
});
