import {
  deleteCharBackward,
  deleteCharForward,
  history,
  insertNewlineAndIndent,
  undo,
} from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState, type Extension } from "@codemirror/state";
import { EditorView, keymap, runScopeHandlers } from "@codemirror/view";
import {
  hosted,
  latelyMarked,
  markLately,
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

describe("deleting what kept a format apart from a word", () => {
  it("takes the word inside when a closing mark would be undone", () => {
    const back = written("**a.** bcd efg", 7);
    deleteCharBackward(back);
    expect(text(back)).toBe("**a.bcd** efg");
    expect(caret(back)).toBe(4);
    const forward = written("**a.** bcd", 6);
    deleteCharForward(forward);
    expect(text(forward)).toBe("**a.bcd**");
    const inside = written("**a.** bcd", 4);
    deleteCharForward(inside);
    expect(text(inside)).toBe("**a.bcd**");
    expect(caret(inside)).toBe(4);
  });

  it("takes the word inside when an opening mark would be undone", () => {
    const under = written("el ++dato++", 3);
    deleteCharBackward(under);
    expect(text(under)).toBe("++eldato++");
    expect(caret(under)).toBe(4);
    const bold = written("b **.a**", 2);
    deleteCharBackward(bold);
    expect(text(bold)).toBe("**b.a**");
  });

  it("does the same for Tisty's own marks and for marks inside marks", () => {
    const under = written("++dato++ y", 9);
    deleteCharBackward(under);
    expect(text(under)).toBe("++datoy++");
    const nested = written("***a.*** b", 9);
    deleteCharBackward(nested);
    expect(text(nested)).toBe("***a.b***");
  });

  it("does the same when a selection is what kept them apart", () => {
    const view = written("**a.** y b", 6, 9);
    cut(view);
    expect(text(view)).toBe("**a.b**");
    expect(caret(view)).toBe(4);
  });

  it("leaves the marks where they are when the format holds", () => {
    const plain = written("**a** b", 6);
    deleteCharBackward(plain);
    expect(text(plain)).toBe("**a**b");
    const sign = written("**a.** (b", 7);
    deleteCharBackward(sign);
    expect(text(sign)).toBe("**a.**(b");
  });

  it("leaves code alone", () => {
    const inline = written("`**a.** b`", 8);
    deleteCharBackward(inline);
    expect(text(inline)).toBe("`**a.**b`");
    const fenced = written("```\n**a.** b\n```", 11);
    deleteCharBackward(fenced);
    expect(text(fenced)).toBe("```\n**a.**b\n```");
  });

  it("leaves the marks alone when they are shown", () => {
    const shown = written(
      "**a.** b",
      7,
      7,
      marksShown.init(() => true),
    );
    deleteCharBackward(shown);
    expect(text(shown)).toBe("**a.**b");
    const whole = written("**a.** b", 7, 7, revealing.of(true));
    deleteCharBackward(whole);
    expect(text(whole)).toBe("**a.**b");
  });

  it("undoes in one step", () => {
    const view = written("**a.** b", 7, 7, history());
    deleteCharBackward(view);
    expect(text(view)).toBe("**a.b**");
    undo(view);
    expect(text(view)).toBe("**a.** b");
  });

  const backspacedAt = (doc: string, at: string) => {
    const view = written(doc, doc.indexOf(at) + at.length);
    deleteCharBackward(view);
    return text(view);
  };

  it("leaves code and links out of it, with the word beside them", () => {
    expect(backspacedAt("**`c`** bcd", "** ")).toBe("**`c`bcd**");
    expect(backspacedAt("**[a](http://x.y)** bcd", "** ")).toBe("**[a](http://x.y)bcd**");
    expect(backspacedAt("x **`c`**", "x ")).toBe("**x`c`**");
  });

  it("leaves what is written literally alone", () => {
    expect(backspacedAt("$$\n**a.** b\n$$", "** ")).toBe("$$\n**a.**b\n$$");
    expect(backspacedAt("<div>\n**a.** b\n</div>", "** ")).toBe("<div>\n**a.**b\n</div>");
    expect(backspacedAt("---\ntitle: **a.** b\n---\n", "** ")).toBe("---\ntitle: **a.**b\n---\n");
  });

  it("takes everything up to the next space, whatever it holds", () => {
    expect(backspacedAt("**a.** http://x.y fin", "** ")).toBe("**a.http://x.y** fin");
    expect(backspacedAt("**a.** foo-bar", "** ")).toBe("**a.foo-bar**");
    expect(backspacedAt("foo-bar **.a**", "bar ")).toBe("**foo-bar.a**");
  });

  it("holds a format that started on a line above", () => {
    expect(backspacedAt("**uno\ndos.** b", "** ")).toBe("**uno\ndos.b**");
  });

  it("joins a format with one of its kind right after the word", () => {
    const view = written("**a.** b**c**", 7);
    deleteCharBackward(view);
    expect(text(view)).toBe("**a.bc**");
    expect(caret(view)).toBe(4);
    expect(backspacedAt("**c**b **.a**", "b ")).toBe("**cb.a**");
  });

  it("holds a format when the letter before its closing mark goes", () => {
    const view = written("**a.b**c", 5);
    deleteCharBackward(view);
    expect(text(view)).toBe("**a.c**");
    expect(caret(view)).toBe(4);
  });

  it("holds a paragraph indented in a list", () => {
    expect(backspacedAt("- x\n\n    **a.** b", "** ")).toBe("- x\n\n    **a.b**");
  });

  it("gives the space back with the format as it was", () => {
    const closing = written("**a.** bcd", 7);
    deleteCharBackward(closing);
    expect(text(closing)).toBe("**a.bcd**");
    typed(closing, " ");
    expect(text(closing)).toBe("**a.** bcd");
    expect(caret(closing)).toBe(7);
    const opening = written("el ++dato++", 3);
    deleteCharBackward(opening);
    typed(opening, " ");
    expect(text(opening)).toBe("el ++dato++");
    expect(caret(opening)).toBe(3);
  });

  it("writes a space inside once the caret has moved or something else was typed", () => {
    const moved = written("**a.** bcd", 7);
    deleteCharBackward(moved);
    moved.dispatch({ selection: { anchor: 5 } });
    moved.dispatch({ selection: { anchor: 4 } });
    typed(moved, " ");
    expect(text(moved)).toBe("**a. bcd**");
    const other = written("**a.** bcd", 7);
    deleteCharBackward(other);
    typed(other, "x");
    typed(other, " ");
    expect(text(other)).toBe("**a. xbcd**");
  });

  it("does not disturb what is marked elsewhere on the line", () => {
    const view = written("**a.** bcd fin", 7, 7, latelyMarked);
    view.dispatch({ effects: markLately.of({ from: 11, to: 14 }) });
    deleteCharBackward(view);
    const spans: string[] = [];
    view.state.field(latelyMarked).between(0, view.state.doc.length, (from, to) => {
      spans.push(view.state.sliceDoc(from, to));
    });
    expect(spans).toEqual(["fin"]);
  });
});

describe("keeping a section's fences out of sight", () => {
  const section = ":::details A\n\nuno\n\n:::";

  it("keeps the closing fence when a selection runs out of the body", () => {
    const view = written(`${section}\n\nfin`, 14, section.length);
    cut(view);
    expect(text(view)).toBe(":::details A\n\n:::\n\nfin");
  });

  it("deletes a whole section when all of it is selected", () => {
    const view = written(`a\n\n${section}\n\nb`, 3, 3 + section.length);
    cut(view);
    expect(text(view)).toBe("a\n\n\n\nb");
  });

  it("leaves a fence the caret stands on to be written", () => {
    const view = written(section, section.length);
    deleteCharBackward(view);
    expect(text(view)).toBe(":::details A\n\nuno\n\n::");
  });

  it("deletes nothing past a selection made only of a fence", () => {
    const view = written(`${section}\n\nfin`, section.length - 3, section.length);
    cut(view);
    expect(text(view)).toBe(`${section}\n\nfin`);
  });

  it("lets Enter start a paragraph inside the section", () => {
    const view = written(section, 17);
    expect(enter(view)).toBe(true);
    expect(text(view)).toBe(":::details A\n\nuno\n\n\n\n:::");
  });
});

describe("writing against a format's edge", () => {
  it("takes a letter inside when it would undo the format", () => {
    const end = written("el ++dato++ y", 11);
    typed(end, "s");
    expect(text(end)).toBe("el ++datos++ y");
    expect(caret(end)).toBe(10);
    const start = written("el ++dato++", 3);
    typed(start, "x");
    expect(text(start)).toBe("el ++xdato++");
    expect(caret(start)).toBe(6);
  });

  it("leaves a letter outside when the format holds", () => {
    const bold = written("**dato** y", 8);
    typed(bold, "s");
    expect(text(bold)).toBe("**dato**s y");
    const pen = written("==dato== y", 8);
    typed(pen, "s");
    expect(text(pen)).toBe("==dato==s y");
    const spaced = written("el ++dato++", 11);
    typed(spaced, " ");
    expect(text(spaced)).toBe("el ++dato++ ");
  });

  it("leaves what is composed alone, with its accents", () => {
    const view = written("**dato** y", 4, 9);
    view.dispatch({
      changes: { from: 4, to: 9, insert: "á" },
      userEvent: "input.type.compose",
    });
    expect(text(view)).toBe("**daáy");
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

  it("goes on to the body from a title with no words yet", () => {
    const view = written("# ", 2);
    expect(enter(view)).toBe(true);
    expect(text(view)).toBe("# \n\n");
    expect(caret(view)).toBe(4);
  });

  it("pushes a title down from its start, the caret staying with it", () => {
    const view = written("# Plan", 2);
    expect(enter(view)).toBe(true);
    expect(text(view)).toBe("\n\n# Plan");
    expect(caret(view)).toBe(4);
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
