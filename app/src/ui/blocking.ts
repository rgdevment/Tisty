import { closeCompletion, completionStatus } from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap, indentWithTab, redo } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorState, Prec, type Text } from "@codemirror/state";
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
import { docFromHtml, docFromText } from "../core";
import { known } from "../glyphs";
import { barred, blockKeys, flipped } from "./barring";
import { docked } from "./docking";
import { coded, owning } from "./formatting";
import { HUES } from "./paging";
import { pendingFormat, pendingKeys, settle } from "./pending";
import { visual } from "./seeing";
import { type Asking, putBlock, slashing } from "./slashing";
import { tableKeys } from "./tabling";

const PENS = ["yellow", ...HUES];

// A code editor copies coloured spans as preformatted text; Docs and Gmail use pre-wrap, which is prose.
export const worded = (html: string): boolean =>
  /<(h[1-6]|p|li|table|strong|b|em|i|a|img|blockquote|mark|u|s|del|details)\b/i.test(html) &&
  !/white-space:\s*pre\s*[;"']/i.test(html);

type Landing = { view: EditorView; lost?: () => void };

// The translation comes back a moment later; the words go where the caret was when they were pasted.
const landing = (view: EditorView, lost?: () => void): ((text: string, block: boolean) => void) => {
  const doc = view.state.doc;
  const { anchor, head } = view.state.selection.main;
  return (text, block) => {
    if (!view.dom.isConnected) return void lost?.();
    if (!text) return;
    if (view.state.doc === doc) view.dispatch({ selection: { anchor, head } });
    if (block && text.includes("\n")) putBlock(view, text);
    else view.dispatch(view.state.replaceSelection(text), { userEvent: "input.paste" });
  };
};

// Only a table opens on a top corner, and a few lines with code's punctuation are worth a look; the core decides.
export const shaped = (plain: string): boolean =>
  /^\s*[┌╔┏]/m.test(plain) || (plain.split("\n").length >= 3 && /[{};=(:]/.test(plain));

const rich = (event: ClipboardEvent, { view, lost }: Landing): boolean => {
  if (coded(view.state, view.state.selection.main.head)) return false;
  const html = event.clipboardData?.getData("text/html") ?? "";
  const plain = event.clipboardData?.getData("text/plain") ?? "";
  const put = landing(view, lost);
  if (worded(html)) {
    event.preventDefault();
    docFromHtml(html)
      .then((text) => put(text || plain, Boolean(text)))
      .catch(() => put(plain, false));
    return true;
  }
  if (!shaped(plain)) return false;
  event.preventDefault();
  docFromText(plain)
    .then((text) => put(text ?? plain, Boolean(text)))
    .catch(() => put(plain, false));
  return true;
};

const pasting = (event: ClipboardEvent, view: EditorView, asking?: () => Asking | undefined) => {
  const files = Array.from(event.clipboardData?.files ?? []);
  const pasted = asking?.()?.pasted;
  // Word, Excel and browsers put a picture beside the words they copy; the words are what was meant.
  const words = Boolean(event.clipboardData?.getData("text/plain"));
  if (files.length && pasted && !words) {
    event.preventDefault();
    pasted(files)
      .then((text) => {
        if (!text) return;
        if (view.dom.isConnected) putBlock(view, text);
        else asking?.()?.lost?.();
      })
      .catch(() => {});
    return true;
  }
  return rich(event, { view, lost: asking?.()?.lost });
};

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

const LABEL = 20;

// A task's section name is a short label, so the template refuses a longer one as it is typed.
const named = (doc: Text): number =>
  [...(/^#{1,6}\s+(.*)$/.exec(doc.line(1).text)?.[1] ?? "")].length;

// Composing skips the filter, so a label is held to its limit once more as it is left.
const held = (view: EditorView, was: string) => {
  const head = view.state.doc.line(1);
  const words = /^(#{1,6}\s+)(.*)$/.exec(head.text);
  const most = Math.max(LABEL, named(EditorState.create({ doc: was }).doc));
  if (!words || [...words[2]].length <= most) return;
  const kept = [...words[2]].slice(0, most).join("");
  view.dispatch({ changes: { from: head.from + words[1].length, to: head.to, insert: kept } });
};

const labelled = EditorState.transactionFilter.of((tr) => {
  if (!tr.docChanged || tr.isUserEvent("input.type.compose")) return tr;
  // A label or a hint is one line, so a pasted break would start words under it.
  if (tr.newDoc.lines > tr.startState.doc.lines) return [];
  // A name already longer than the limit can still be shortened, never grown.
  const now = named(tr.newDoc);
  return now > LABEL && now > named(tr.startState.doc) ? [] : tr;
});

export const edited = (
  block: HTMLElement,
  said: string,
  onChange: (text: string) => void,
  onLeave: (text: string) => void,
  asking?: () => Asking | undefined,
  oneLine = false,
): Editing => {
  const page = block.ownerDocument;
  const host = page.createElement("div");
  host.className = "editing";
  // The block is written where it is read, so it keeps the room and the look the page gave it.
  const look = page.defaultView?.getComputedStyle(block);
  if (look) host.style.margin = look.margin;
  const first = block.parentElement?.firstElementChild;
  const heads = first === block || (first?.matches(".front") && first.nextElementSibling === block);
  if (heads && block.matches("html:not(.untitled) .page > *:not(.front)")) {
    host.classList.add("titled");
  }
  if (block.matches(".page > .front:first-child")) host.classList.add("fronted");
  block.replaceWith(host);
  let gone = false;
  let follow = (_: EditorView) => {};
  let undock = () => {};
  const leave = () => {
    if (gone) return;
    gone = true;
    if (oneLine) held(view, said);
    settle(view);
    const text = view.state.sliceDoc();
    view.destroy();
    undock();
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
            // A template's label or hint is one line, so Enter ends it instead of starting words below.
            {
              key: "Enter",
              run: () => {
                if (oneLine) leave();
                return oneLine;
              },
            },
            ...pendingKeys,
            ...blockKeys,
            ...tableKeys,
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
        EditorView.domEventHandlers({ paste: (event, view) => pasting(event, view, asking) }),
        dressing,
        EditorView.lineWrapping,
        oneLine ? labelled : [],
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChange(update.state.sliceDoc());
          if (update.transactions.length > 0) follow(update.view);
        }),
      ],
    }),
  });
  const bar = barred(view);
  follow = bar.follow;
  undock = docked(bar.bar, bar.hint);
  view.focus();
  return { view, leave, kept: bar.kept };
};
