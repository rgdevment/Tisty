import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { wrapped } from "../ui/formatting";
import { pending, pendingFormat, settle } from "../ui/pending";
import { visual } from "../ui/seeing";

const view = (doc: string, at = doc.length) =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc,
      selection: EditorSelection.cursor(at),
      extensions: [markdown({ base: markdownLanguage }), visual, pendingFormat],
    }),
  });

const type = (editor: EditorView, text: string) => {
  const at = editor.state.selection.main.head;
  editor.dispatch({ changes: { from: at, insert: text }, selection: { anchor: at + text.length } });
};

describe("a format pressed with nothing selected", () => {
  it("looks like the format while it is typed, its marks out of sight", () => {
    const editor = view("Esto ");
    wrapped(editor, "==");
    expect(editor.state.sliceDoc()).toBe("Esto ====");
    expect(editor.dom.querySelector(".cm-content")?.textContent).toBe("Esto ");
    type(editor, "me llamo");
    expect(editor.dom.querySelector(".cm-pen-yellow")?.textContent).toBe("me llamo");
    expect(editor.dom.querySelector(".cm-content")?.textContent).toBe("Esto me llamo");
    editor.destroy();
  });

  it("moves the spaces typed at its edges outside the marks once the caret leaves", () => {
    const editor = view("Esto ");
    wrapped(editor, "==", "=={green}");
    type(editor, "me llamo juan ");
    editor.dispatch({ selection: { anchor: 0 } });
    expect(editor.state.sliceDoc()).toBe("Esto ==me llamo juan=={green} ");
    expect(editor.state.field(pending)).toBeNull();
    editor.destroy();
  });

  it("settles when the block is left, and takes empty marks away", () => {
    const editor = view("Hola ");
    wrapped(editor, "**");
    type(editor, "juan ");
    settle(editor);
    expect(editor.state.sliceDoc()).toBe("Hola **juan** ");
    wrapped(editor, "*");
    settle(editor);
    expect(editor.state.sliceDoc()).toBe("Hola **juan** ");
    settle(editor);
    editor.destroy();
  });

  it("keeps going while the caret stays inside, and leaves a link's marks in sight", () => {
    const editor = view("");
    wrapped(editor, "~~");
    type(editor, "no");
    editor.dispatch({ selection: { anchor: 3 } });
    expect(editor.state.field(pending)).not.toBeNull();
    expect(editor.dom.querySelector(".cm-pending-struck")?.textContent).toBe("no");
    editor.dispatch({ selection: { anchor: editor.state.doc.length } });
    expect(editor.state.sliceDoc()).toBe("~~no~~");
    const link = view("ver ");
    wrapped(link, "[", "]()");
    expect(link.state.field(pending)).not.toBeNull();
    expect(link.contentDOM.textContent).toBe("ver []()");
    link.dispatch({ selection: { anchor: 0 } });
    expect(link.state.sliceDoc()).toBe("ver ");
    editor.destroy();
    link.destroy();
  });
});
