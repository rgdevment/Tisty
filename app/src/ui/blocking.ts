import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { tags } from "@lezer/highlight";

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
    const text = view.state.doc.toString();
    view.destroy();
    onLeave(text);
  };
  const view = new EditorView({
    parent: host,
    root: page,
    state: EditorState.create({
      doc: said.replace(/\n$/, ""),
      extensions: [
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
        dressing,
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChange(update.state.doc.toString());
        }),
      ],
    }),
  });
  view.focus();
  return { view, leave };
};
