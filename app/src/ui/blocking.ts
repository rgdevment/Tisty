import { closeCompletion, completionStatus } from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap, indentWithTab, redo } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import {
  EditorState,
  type Line,
  MapMode,
  Prec,
  type RangeSet,
  RangeSetBuilder,
  RangeValue,
  StateField,
  type Text,
} from "@codemirror/state";
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
import { latelyMarked } from "./lately";
import { HUES } from "./paging";
import { pendingFormat, pendingKeys, settle } from "./pending";
import { gridFailed, gridKeys } from "./regridding";
import { visual } from "./seeing";
import { type Asking, BOX, putBlock, putTable, slashing } from "./slashing";
import { tableHere, tableKeys } from "./tabling";

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

// A table drawn with corners or copied from a spreadsheet, or a few lines with code's punctuation, are worth a look; the core decides.
export const shaped = (plain: string): boolean => {
  const lines = plain.trim().split("\n").length;
  return (
    /^\s*[┌╔┏]/m.test(plain) ||
    (lines >= 2 && plain.includes("\t")) ||
    (lines >= 3 && /^[^\n]*,[^\n]*,/.test(plain.trim())) ||
    (lines >= 3 && /[{};=(:]/.test(plain))
  );
};

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

class Boxed extends RangeValue {
  constructor(readonly said: string) {
    super();
  }
  eq(other: RangeValue): boolean {
    return other instanceof Boxed && other.said === this.said;
  }
}

const boxesIn = (doc: Text): RangeSet<Boxed> => {
  const found = new RangeSetBuilder<Boxed>();
  for (let at = 1; at <= doc.lines; at++) {
    const line = doc.line(at);
    const box = BOX.exec(line.text);
    if (box) found.add(line.from, line.from, new Boxed(box[1].trim()));
  }
  return found.finish();
};

// Each box keeps what it said when the block opened, following its line through every edit.
const boxed = StateField.define<RangeSet<Boxed>>({
  create: (state) => boxesIn(state.doc),
  // A box whose line went away takes its mark along, so the line under it never inherits it.
  update: (set, tr) => {
    if (!tr.docChanged) return set;
    const kept = new RangeSetBuilder<Boxed>();
    for (let at = set.iter(); at.value; at.next()) {
      const to = tr.changes.mapPos(at.from, 1, MapMode.TrackAfter);
      if (to !== null) kept.add(to, to, at.value);
    }
    return kept.finish();
  },
});

export const renamedIn = (state: EditorState): [string, string][] => {
  const out: [string, string][] = [];
  const seen = new Set<number>();
  for (let at = state.field(boxed).iter(); at.value; at.next()) {
    const line = state.doc.lineAt(at.from);
    const now = BOX.exec(line.text)?.[1].trim();
    if (seen.has(line.number) || !now || now === at.value.said) continue;
    seen.add(line.number);
    out.push([at.value.said, now]);
  }
  return out;
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

const EMPTY_STEP = /^[\s>]*(?:[-*+]|\d+[.)])\s+\[[ xX]\]\s*$/;

// A step with no words is no step, and a blank line between steps splits them into two lists.
export const tightened = (state: EditorState): string => {
  const { doc } = state;
  const lines: Line[] = [];
  for (let at = 1; at <= doc.lines; at++) {
    const line = doc.line(at);
    if (coded(state, line.from) || !EMPTY_STEP.test(line.text)) lines.push(line);
  }
  const blank = (line: Line) => !line.text.trim() && !coded(state, line.from);
  const kept = lines.filter((line, at) => {
    if (!blank(line)) return true;
    const back = lines
      .slice(0, at)
      .reverse()
      .find((one) => !blank(one));
    const ahead = lines.slice(at + 1).find((one) => !blank(one));
    if (!ahead) return false;
    return !(back && BOX.test(back.text) && BOX.test(ahead.text));
  });
  return kept.map((line) => line.text).join(state.lineBreak);
};

/** Where the bar rests: over the words chosen, over the block being written, or at the top of a task's page. */
export const aimed = (view: EditorView, untitled: boolean): number | null => {
  const chosen = view.state.selection.main;
  // Inside a table the bar stays above it, so walking its cells never lays it over the rows.
  if (!chosen.empty && !tableHere(view.state)) return view.coordsAtPos(chosen.from)?.top ?? null;
  return untitled ? null : view.dom.getBoundingClientRect().top;
};

export const edited = (
  block: HTMLElement,
  said: string,
  onChange: (text: string) => void,
  onLeave: (text: string, renamed: [string, string][]) => void,
  asking?: () => Asking | undefined,
  oneLine = false,
  whole = false,
  onWhole?: (all: boolean) => void,
): Editing => {
  const page = block.ownerDocument;
  const host = page.createElement("div");
  host.className = whole ? "editing whole" : "editing";
  // The block is written where it is read, so it keeps the room and the look the page gave it.
  const look = page.defaultView?.getComputedStyle(block);
  if (look) host.style.margin = look.margin;
  const first = block.parentElement?.firstElementChild;
  const heads = first === block || (first?.matches(".front") && first.nextElementSibling === block);
  if (!whole && heads && block.matches("html:not(.untitled) .page > *:not(.front)")) {
    host.classList.add("titled");
  }
  if (block.matches(".page > .front:first-child")) host.classList.add("fronted");
  // A task's steps are a plain list: no bar over them, and an empty step ends the writing.
  const stepping = block.matches(
    "html.untitled .page > :is(.add-step, .step-hint, :is(ul, ol):has(> li.task))",
  );
  if (stepping) host.classList.add("stepping");
  block.replaceWith(host);
  let gone = false;
  let follow = (_: EditorView) => {};
  let undock = () => {};
  let replace = () => {};
  const leave = () => {
    if (gone) return;
    gone = true;
    if (oneLine) held(view, said);
    settle(view);
    const tight = tightened(view.state);
    if (tight !== view.state.sliceDoc()) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: tight } });
    }
    const text = view.state.sliceDoc();
    const renamed = renamedIn(view.state);
    view.destroy();
    undock();
    host.replaceWith(block);
    onLeave(text, renamed);
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
              run: (one) => {
                const { main } = one.state.selection;
                const line = one.state.doc.lineAt(main.head);
                const ends =
                  stepping && main.empty && main.head === line.to && EMPTY_STEP.test(line.text);
                if (oneLine || ends) leave();
                return oneLine || ends;
              },
            },
            // A block already chosen whole is chosen past, so the next Ctrl+A takes the whole document.
            {
              key: "Mod-a",
              run: (one) => {
                const chosen = one.state.selection.main;
                const length = one.state.doc.length;
                if (whole || !onWhole || !length || chosen.from !== 0 || chosen.to !== length) {
                  return false;
                }
                onWhole(true);
                return true;
              },
            },
            ...pendingKeys,
            ...blockKeys,
            ...gridKeys,
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
        gridFailed.of((problem) => asking?.()?.failed?.(problem)),
        visual,
        latelyMarked,
        pendingFormat,
        syntaxHighlighting(inked),
        marked,
        slashing(asking),
        EditorView.domEventHandlers({
          paste: (event, view) => pasting(event, view, asking),
        }),
        dressing,
        EditorView.lineWrapping,
        boxed,
        oneLine ? labelled : [],
        EditorView.updateListener.of((update) => {
          // Words taken in from outside are already the sheet's; only the person's are handed back.
          const theirs = update.transactions.every((one) => one.isUserEvent("outside"));
          if (update.docChanged && !theirs) onChange(update.state.sliceDoc());
          if (update.transactions.length > 0) follow(update.view);
          // The editor's layout cannot be read inside its own update, so the bar moves on the next frame.
          if (update.selectionSet || update.docChanged)
            page.defaultView?.requestAnimationFrame(() => {
              if (!gone) replace();
            });
        }),
      ],
    }),
  });
  const bar = barred(view, whole, onWhole && (() => onWhole(false)), () =>
    putTable(view, asking?.()),
  );
  if (stepping) bar.bar.style.display = "none";
  follow = bar.follow;
  const dock = docked(
    bar.bar,
    bar.hint,
    () => aimed(view, page.documentElement.classList.contains("untitled")),
    host,
    whole,
  );
  undock = dock.stop;
  replace = dock.place;
  view.focus();
  return { view, leave, kept: bar.kept };
};
