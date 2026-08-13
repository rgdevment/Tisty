import { useEffect, useRef } from "react";
import { EditorState, type Extension, type Range } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, keymap, type DecorationSet } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { syntaxTree } from "@codemirror/language";
import { markdown } from "@codemirror/lang-markdown";

const SYNTAX = new Set([
  "HeaderMark",
  "StrongMark",
  "EmphasisMark",
  "CodeMark",
  "StrikethroughMark",
  "QuoteMark",
  "LinkMark",
]);

const STYLED: Record<string, string> = {
  ATXHeading1: "cm-doc-h1",
  ATXHeading2: "cm-doc-h2",
  ATXHeading3: "cm-doc-h3",
  ATXHeading4: "cm-doc-h4",
  ATXHeading5: "cm-doc-h4",
  ATXHeading6: "cm-doc-h4",
  Blockquote: "cm-doc-quote",
};

const INLINE: Record<string, string> = {
  StrongEmphasis: "cm-doc-strong",
  Emphasis: "cm-doc-em",
  InlineCode: "cm-doc-code",
  Strikethrough: "cm-doc-struck",
  URL: "cm-doc-url",
};

const gone = Decoration.replace({});

function painted(view: EditorView): DecorationSet {
  const live = new Set(
    view.state.selection.ranges.map((one) => view.state.doc.lineAt(one.head).number),
  );
  const found: Range<Decoration>[] = [];

  for (const { from, to } of view.visibleRanges) {
    syntaxTree(view.state).iterate({
      from,
      to,
      enter: (node) => {
        const line = view.state.doc.lineAt(node.from);
        const writing = live.has(line.number);

        const block = STYLED[node.name];
        if (block) found.push(Decoration.line({ class: block }).range(line.from));

        const inline = INLINE[node.name];
        if (inline && node.to > node.from) {
          found.push(Decoration.mark({ class: inline }).range(node.from, node.to));
        }

        if (!writing && SYNTAX.has(node.name) && node.to > node.from) {
          found.push(gone.range(node.from, node.to));
        }
      },
    });
  }
  return Decoration.set(found, true);
}

const live = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = painted(view);
    }

    update(change: { view: EditorView; docChanged: boolean; selectionSet: boolean; viewportChanged: boolean }) {
      if (change.docChanged || change.selectionSet || change.viewportChanged) {
        this.decorations = painted(change.view);
      }
    }
  },
  { decorations: (plugin) => plugin.decorations },
);

const looks = EditorView.theme({
  "&": { color: "var(--tisty-ink)", backgroundColor: "transparent", fontSize: "15px" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "1.7" },
  "&.cm-focused": { outline: "none" },
  ".cm-content": {
    fontFamily: "inherit",
    lineHeight: "1.7",
    padding: "0 0 40vh 0",
    caretColor: "var(--tisty-accent)",
  },
  ".cm-line": { padding: "0" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--tisty-accent)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
    backgroundColor: "var(--tisty-accent-soft)",
  },
  ".cm-gutters": { display: "none" },
  ".cm-doc-h1": { fontSize: "1.7em", fontWeight: "600", lineHeight: "1.3" },
  ".cm-doc-h2": { fontSize: "1.35em", fontWeight: "600", lineHeight: "1.35" },
  ".cm-doc-h3": { fontSize: "1.15em", fontWeight: "600" },
  ".cm-doc-h4": { fontSize: "1em", fontWeight: "600" },
  ".cm-doc-quote": {
    borderLeft: "2px solid var(--tisty-line)",
    paddingLeft: "12px",
    color: "var(--tisty-soft)",
  },
  ".cm-doc-strong": { fontWeight: "600" },
  ".cm-doc-em": { fontStyle: "italic" },
  ".cm-doc-struck": { textDecoration: "line-through", color: "var(--tisty-faint)" },
  ".cm-doc-code": {
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
    fontSize: "0.9em",
    backgroundColor: "var(--tisty-hover)",
    borderRadius: "4px",
    padding: "1px 4px",
  },
  ".cm-doc-url": { color: "var(--tisty-accent)" },
});

interface Props {
  value: string;
  taking?: boolean;
  onWrite: (text: string) => void;
}

export default function Editor({ value, taking, onWrite }: Props) {
  const slot = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView>(null);
  const latest = useRef(onWrite);
  latest.current = onWrite;

  useEffect(() => {
    if (!slot.current) return;

    const extensions: Extension[] = [
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      markdown(),
      EditorView.lineWrapping,
      live,
      looks,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) latest.current(update.state.doc.toString());
      }),
    ];

    const editor = new EditorView({
      state: EditorState.create({ doc: value, extensions }),
      parent: slot.current,
    });
    view.current = editor;
    if (taking) editor.focus();

    return () => {
      editor.destroy();
      view.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const editor = view.current;
    if (!editor || editor.state.doc.toString() === value) return;
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: value },
    });
  }, [value]);

  return <div ref={slot} className="h-full overflow-auto" />;
}
