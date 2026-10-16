import { closeCompletion, completionStatus } from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap, indentWithTab, redo } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorState, Prec } from "@codemirror/state";
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
import { barred, blockKeys, flipped } from "./barring";
import { coded, owning } from "./formatting";
import { HUES } from "./paging";
import { pendingFormat, pendingKeys, settle } from "./pending";
import { visual } from "./seeing";
import { type Asking, slashing } from "./slashing";

const PENS = ["yellow", ...HUES];

export type Editing = {
  readonly view: EditorView;
  readonly leave: () => void;
  readonly kept: (yes: boolean) => void;
};

const inked = HighlightStyle.define([
  { tag: tags.heading, fontWeight: "640" },
  { tag: tags.strong, fontWeight: "640" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.strikethrough, textDecoration: "line-through" },
  { tag: tags.link, color: "var(--tisty-accent)" },
  { tag: tags.url, color: "var(--tisty-faint)" },
  { tag: tags.monospace, fontFamily: "ui-monospace, 'Cascadia Mono', 'SF Mono', Menlo, monospace" },
  { tag: [tags.processingInstruction, tags.meta], color: "var(--tisty-faint)" },
]);

const dressing = EditorView.theme({
  "&": { fontSize: "inherit", background: "transparent", color: "inherit" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "inherit", overflow: "visible" },
  ".cm-content": { fontFamily: "inherit", padding: "0", caretColor: "var(--tisty-accent)" },
  ".cm-line": { padding: "0" },
  ".cm-cursor": { borderLeftColor: "var(--tisty-accent)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
    background: "var(--tisty-accent-soft)",
  },
});

const marking = new MatchDecorator({
  regexp: owning(),
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
  asking?: () => Asking | undefined,
): Editing => {
  const page = block.ownerDocument;
  const host = page.createElement("div");
  host.className = "editing";
  // The block is written where it is read, so it keeps the room and the look the page gave it.
  const look = page.defaultView?.getComputedStyle(block);
  if (look) host.style.margin = look.margin;
  if (block.parentElement?.firstElementChild === block && block.matches(".page > *")) {
    host.classList.add("titled");
  }
  block.replaceWith(host);
  let gone = false;
  let follow = (_: EditorView) => {};
  let hint: HTMLElement = page.createElement("div");
  const leave = () => {
    if (gone) return;
    gone = true;
    settle(view);
    const text = view.state.sliceDoc();
    view.destroy();
    hint.remove();
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
        // A completion still thinking shows nothing, so only one in sight may keep Escape from leaving.
        Prec.highest(
          keymap.of([
            {
              key: "Escape",
              run: (one) => {
                if (completionStatus(one.state) === "active") return closeCompletion(one);
                leave();
                return true;
              },
            },
            ...pendingKeys,
            ...blockKeys,
            { key: "Mod-Shift-z", run: redo },
          ]),
        ),
        keymap.of([
          { key: "Mod-Shift-m", run: flipped },
          ...defaultKeymap,
          ...historyKeymap,
          indentWithTab,
        ]),
        markdown({ base: markdownLanguage }),
        visual,
        pendingFormat,
        syntaxHighlighting(inked),
        marked,
        slashing(asking),
        dressing,
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChange(update.state.sliceDoc());
          if (update.transactions.length > 0) follow(update.view);
        }),
      ],
    }),
  });
  const bar = barred(view);
  follow = bar.follow;
  host.prepend(bar.bar);
  hint = bar.hint;
  // At the foot of the sheet, so the blocks after this one stay where they were read.
  (host.closest(".page") ?? host).append(hint);
  view.focus();
  return { view, leave, kept: bar.kept };
};
