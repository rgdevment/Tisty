import {
  deleteCharBackward,
  deleteCharForward,
  insertNewlineAndIndent,
} from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState, type Extension } from "@codemirror/state";
import { EditorView, keymap, runScopeHandlers } from "@codemirror/view";
import {
  hosted,
  marksShown,
  outOfPairs,
  pairsKept,
  paragraphKeys,
  revealing,
  visual,
} from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { HOST } from "../ui/hosting";

const open: EditorView[] = [];

afterEach(() => {
  for (const one of open.splice(0)) one.destroy();
});

const written = (doc: string, anchor: number, head = anchor, extra: Extension = []) => {
  const view = new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc,
      selection: EditorSelection.single(anchor, head),
      extensions: [
        hosted.of(HOST),
        markdown({ base: markdownLanguage, addKeymap: false }),
        visual,
        pairsKept,
        keymap.of([...paragraphKeys, { key: "Shift-Enter", run: insertNewlineAndIndent }]),
        extra,
      ],
    }),
  });
  open.push(view);
  return view;
};

const text = (view: EditorView) => view.state.sliceDoc();
const caret = (view: EditorView) => view.state.selection.main.head;
const typed = (view: EditorView, words: string) => {
  const { from, to } = view.state.selection.main;
  view.dispatch({ changes: { from, to, insert: words }, userEvent: "input.type" });
};
const cut = (view: EditorView) => {
  const { from, to } = view.state.selection.main;
  view.dispatch({ changes: { from, to }, userEvent: "delete.selection" });
};
const enter = (view: EditorView, shiftKey = false) =>
  runScopeHandlers(view, new KeyboardEvent("keydown", { key: "Enter", shiftKey }), "editor");

describe("deleting beside marks out of sight", () => {
  it("takes the last letter, not the closing mark", () => {
    const view = written("un **dato** ya", 11);
    deleteCharBackward(view);
    expect(text(view)).toBe("un **dat** ya");
    expect(caret(view)).toBe(10);
  });

  it("takes the letter before an opening mark", () => {
    const view = written("un **dato**", 5);
    deleteCharBackward(view);
    expect(text(view)).toBe("un**dato**");
  });

  it("takes the first letter going forward, not the opening mark", () => {
    const view = written("un **dato** ya", 3);
    deleteCharForward(view);
    expect(text(view)).toBe("un **ato** ya");
    expect(caret(view)).toBe(3);
  });

  it("takes the letter after a closing mark going forward", () => {
    const view = written("un **dato** ya", 9);
    deleteCharForward(view);
    expect(text(view)).toBe("un **dato**ya");
  });

  it("removes the marks with the last letter between them", () => {
    const view = written("un **d** ya", 8);
    deleteCharBackward(view);
    expect(text(view)).toBe("un  ya");
  });

  it("does the same for Tisty's own marks", () => {
    const pen = written("el ==dato=={green} y", 18);
    deleteCharBackward(pen);
    expect(text(pen)).toBe("el ==dat=={green} y");
    const under = written("el ++dato++ y", 11);
    deleteCharBackward(under);
    expect(text(under)).toBe("el ++dat++ y");
  });

  it("walks past nested marks to the words", () => {
    const view = written("un ***k*** ya", 10);
    deleteCharBackward(view);
    expect(text(view)).toBe("un  ya");
  });

  it("does nothing at the very start", () => {
    const view = written("**dato**", 2);
    deleteCharBackward(view);
    expect(text(view)).toBe("**dato**");
  });

  it("keeps the closing mark when a selection runs out of the format", () => {
    const view = written("**dato** y pan", 4, 10);
    cut(view);
    expect(text(view)).toBe("**da** pan");
    expect(caret(view)).toBe(4);
  });

  it("keeps the opening mark when a selection runs into the format", () => {
    const view = written("pan y **dato**", 4, 9);
    cut(view);
    expect(text(view)).toBe("pan **ato**");
  });

  it("removes both marks when all between them is deleted", () => {
    const view = written("un **dato** ya", 5, 9);
    cut(view);
    expect(text(view)).toBe("un  ya");
  });

  it("writes over a selection inside the format, keeping it", () => {
    const view = written("un **dato** ya", 5, 9);
    typed(view, "x");
    expect(text(view)).toBe("un **x** ya");
    expect(caret(view)).toBe(6);
  });

  it("writes inside the format over a selection that runs out of it", () => {
    const view = written("**dato** y", 4, 8);
    typed(view, "x");
    expect(text(view)).toBe("**dax** y");
    expect(caret(view)).toBe(5);
  });

  it("leaves plain words and a selection of the whole format alone", () => {
    const plain = written("leche y pan", 5);
    deleteCharBackward(plain);
    expect(text(plain)).toBe("lech y pan");
    const whole = written("a **dato** b", 2, 10);
    cut(whole);
    expect(text(whole)).toBe("a  b");
  });

  it("lets a link the caret stands in lose its address letter by letter", () => {
    const view = written("[a](http://x.y)", 13);
    deleteCharBackward(view);
    expect(text(view)).toBe("[a](http://xy)");
  });

  it("stays out of the way when the marks are shown, and in the whole document", () => {
    const shown = written(
      "un **dato** ya",
      11,
      11,
      marksShown.init(() => true),
    );
    deleteCharBackward(shown);
    expect(text(shown)).toBe("un **dato* ya");
    const whole = written("un **dato** ya", 9, 9, revealing.of(true));
    deleteCharForward(whole);
    expect(text(whole)).toBe("un **dato* ya");
  });
});

describe("Enter in a block", () => {
  it("takes a title's words to the body with a paragraph between", () => {
    const view = written("# Plan", 6);
    expect(enter(view)).toBe(true);
    expect(text(view)).toBe("# Plan\n\n");
    expect(caret(view)).toBe(8);
  });

  it("splits a title in two where the caret stands", () => {
    const view = written("# Plan nuevo", 6);
    enter(view);
    expect(text(view)).toBe("# Plan\n\n nuevo");
    expect(caret(view)).toBe(8);
  });

  it("starts a new paragraph, and Shift+Enter only breaks the line", () => {
    const view = written("leche y pan", 5);
    enter(view);
    expect(text(view)).toBe("leche\n\n y pan");
    const other = written("leche", 5);
    enter(other, true);
    expect(text(other)).toBe("leche\n");
  });

  it("uses the line ends the block was written with", () => {
    const view = written("leche", 5, 5, EditorState.lineSeparator.of("\r\n"));
    enter(view);
    expect(text(view)).toBe("leche\r\n\r\n");
    expect(caret(view)).toBe(7);
  });

  it("steps past a format's marks at its end, and before them at its start", () => {
    const end = written("**dato**", 6);
    enter(end);
    expect(text(end)).toBe("**dato**\n\n");
    expect(caret(end)).toBe(10);
    const start = written("a **dato**", 4);
    expect(outOfPairs(start.state, 4)).toBe(2);
  });

  it("only breaks the line in the middle of a format", () => {
    const view = written("**dato**", 4);
    expect(enter(view)).toBe(false);
    expect(text(view)).toBe("**dato**");
  });

  it("leaves lists, quotes, code and blank lines to their own keys", () => {
    for (const [doc, at] of [
      ["- uno", 5],
      ["> uno", 5],
      ["```\nuno\n```", 7],
      ["uno\n\n", 5],
    ] as const) {
      expect(enter(written(doc, at))).toBe(false);
    }
  });
});
