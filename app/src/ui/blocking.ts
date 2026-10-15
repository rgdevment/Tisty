import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorState } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  keymap,
  MatchDecorator,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { PENS } from "./translating";

export type Editing = {
  readonly view: EditorView;
  readonly leave: () => void;
};

const inked = HighlightStyle.define([
  { tag: tags.heading, fontWeight: "640" },
  { tag: tags.strong, fontWeight: "640" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.strikethrough, textDecoration: "line-through" },
  { tag: tags.link, color: "var(--tisty-accent)" },
  { tag: tags.url, color: "var(--tisty-faint)" },
  { tag: tags.monospace, fontFamily: "ui-monospace, 'Cascadia Mono', 'SF Mono', Menlo, monospace" },
  { tag: [tags.processingInstruction, tags.meta, tags.quote], color: "var(--tisty-faint)" },
]);

const dressing = EditorView.theme({
  "&": { fontSize: "inherit", background: "transparent", color: "inherit" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "inherit" },
  ".cm-content": { fontFamily: "inherit", padding: "0", caretColor: "var(--tisty-accent)" },
  ".cm-line": { padding: "0" },
  ".cm-cursor": { borderLeftColor: "var(--tisty-accent)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
    background: "var(--tisty-accent-soft)",
  },
});

const marking = new MatchDecorator({
  regexp:
    /==(?=\S)((?:(?!==).)+?)(?<=\S)==(?:\{([a-z]+)\})?|(?<![\p{L}\p{N}+])\+\+(?=\S)(.+?)(?<=\S)\+\+(?![\p{L}\p{N}+])/gu,
  decoration: (found) => {
    if (found[3] !== undefined) return Decoration.mark({ class: "cm-under" });
    const pen = found[2] ?? "yellow";
    return Decoration.mark({ class: `cm-pen cm-pen-${PENS.includes(pen) ? pen : "yellow"}` });
  },
});

// Pens and underlines show as they will read, so Tisty's own words never look like markup.
const marked = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = marking.createDeco(view);
    }
    update(update: ViewUpdate) {
      this.decorations = marking.updateDeco(update, this.decorations);
    }
  },
  { decorations: (one) => one.decorations },
);

export const edited = (
  block: HTMLElement,
  said: string,
  onChange: (text: string) => void,
  onLeave: (text: string) => void,
): Editing => {
  const page = block.ownerDocument;
  const host = page.createElement("div");
  host.className = "editing";
  block.replaceWith(host);
  let gone = false;
  const leave = () => {
    if (gone) return;
    gone = true;
    const text = view.state.sliceDoc();
    view.destroy();
    host.replaceWith(block);
    onLeave(text);
  };
  const view = new EditorView({
    parent: host,
    root: page,
    state: EditorState.create({
      doc: said.replace(/\r?\n$/, ""),
      extensions: [
        EditorState.lineSeparator.of(said.includes("\r\n") ? "\r\n" : "\n"),
        history(),
        keymap.of([
          {
            key: "Escape",
            run: () => {
              leave();
              return true;
            },
          },
          ...defaultKeymap,
          ...historyKeymap,
          indentWithTab,
        ]),
        markdown(),
        syntaxHighlighting(inked),
        marked,
        dressing,
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChange(update.state.sliceDoc());
        }),
      ],
    }),
  });
  view.focus();
  return { view, leave };
};
