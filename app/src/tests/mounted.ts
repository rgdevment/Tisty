import { Editor } from "@tiptap/core";
import { asMarkdown, loosened, written } from "../ui/writing";

export interface Open {
  editor: Editor;
  dom: HTMLElement;
  markdown: () => string;
  typed: (text: string) => void;
  wrote: (text: string) => void;
  pressed: (key: string, held?: Partial<KeyboardEventInit>) => void;
  at: (spot: number) => void;
  shut: () => void;
}

export const opened = (content = ""): Open => {
  const dom = document.createElement("div");
  document.body.append(dom);
  const editor = new Editor({ extensions: written(), content: loosened(content), element: dom });

  const typed = (text: string) => {
    for (const one of text) {
      const { from, to } = editor.state.selection;
      const took = editor.view.someProp("handleTextInput", (fn) =>
        fn(editor.view, from, to, one, () => editor.state.tr),
      );
      if (!took) editor.view.dispatch(editor.state.tr.insertText(one, from, to));
    }
  };

  return {
    editor,
    dom,
    markdown: () => asMarkdown(editor) ?? "",
    typed,
    wrote: (text) => {
      editor.view.dispatch(editor.state.tr.insertText(text));
    },
    pressed: (key, held = {}) => {
      editor.view.dom.dispatchEvent(
        new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...held }),
      );
    },
    at: (spot) => {
      editor.commands.setTextSelection(spot);
    },
    shut: () => {
      editor.destroy();
      dom.remove();
    },
  };
};

export const inCell = (open: Open, row: number, column: number): void => {
  let at = -1;
  let rows = 0;
  open.editor.state.doc.descendants((node, pos) => {
    if (node.type.name !== "tableRow") return true;
    if (rows === row) {
      let spot = pos + 1;
      node.forEach((cell, _offset, index) => {
        if (index === column) at = spot + 2;
        spot += cell.nodeSize;
      });
    }
    rows += 1;
    return false;
  });
  open.at(at);
};
