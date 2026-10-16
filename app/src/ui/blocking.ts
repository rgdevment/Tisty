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
import { known } from "../glyphs";
import { HUES } from "./paging";
import { type Asking, coded, slashing } from "./slashing";

const PENS = ["yellow", ...HUES];
const PEN = /==(?=\S)((?:(?!==).)+?)(?<=\S)==(?:\{([a-z]+)\})?/u;
const UNDER = /(?<![\p{L}\p{N}+])\+\+(?=\S)(.+?)(?<=\S)\+\+(?![\p{L}\p{N}+])/u;
const NAMED = /(?<![\p{L}\p{N}])::([a-z0-9-]+)(?: ([a-z]+))?::(?![\p{L}\p{N}])/u;

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
  regexp: new RegExp(`${PEN.source}|${UNDER.source}|${NAMED.source}`, "gu"),
  decoration: (found, view, at) => {
    if (coded(view.state, at)) return null;
    if (found[3] !== undefined) return Decoration.mark({ class: "cm-under" });
    if (found[4] !== undefined) {
      const hue = found[5];
      if (!known(found[4]) || (hue !== undefined && !HUES.includes(hue))) return null;
      return Decoration.mark({ class: hue === undefined ? "cm-ico" : `cm-ico ico-${hue}` });
    }
    const pen = found[2];
    if (pen === undefined) return Decoration.mark({ class: "cm-pen cm-pen-yellow" });
    return PENS.includes(pen) ? Decoration.mark({ class: `cm-pen cm-pen-${pen}` }) : null;
  },
});

// Pens, underlines and icons show as they will read, so Tisty's own words never look like markup.
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

// The page shows the words without their marks, so the place clicked is found by the last words read before it.
export const caretAt = (source: string, read: string): number => {
  if (!read.trim()) return 0;
  for (let size = Math.min(read.length, 24); size > 0; size--) {
    const tail = read.slice(-size);
    const ends: number[] = [];
    for (let at = source.indexOf(tail); at >= 0; at = source.indexOf(tail, at + 1)) {
      ends.push(at + tail.length);
    }
    if (ends.length === 0) continue;
    return ends.find((end) => end >= read.length) ?? ends[ends.length - 1];
  }
  return source.length;
};

export const edited = (
  block: HTMLElement,
  said: string,
  onChange: (text: string) => void,
  onLeave: (text: string) => void,
  asking?: Asking,
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
        slashing(asking),
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
