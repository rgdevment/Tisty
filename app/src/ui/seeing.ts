import { syntaxTree } from "@codemirror/language";
import { type EditorState, type Range, StateEffect, StateField } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
  WidgetType,
} from "@codemirror/view";
import type { SyntaxNode } from "@lezer/common";
import { known, markup } from "../glyphs";
import { t } from "../locales";
import { HUES } from "./paging";
import { coded, owning } from "./slashing";

export const showMarks = StateEffect.define<boolean>();

export const marksShown = StateField.define<boolean>({
  create: () => false,
  update: (was, done) => {
    for (const one of done.effects) if (one.is(showMarks)) return one.value;
    return was;
  },
});

const HEADING = /^ATXHeading(\d)$/;
const INLINE = new Set(["Emphasis", "StrongEmphasis", "InlineCode", "Strikethrough", "Link"]);
const MARKS = new Set([
  "EmphasisMark",
  "StrikethroughMark",
  "CodeMark",
  "LinkMark",
  "URL",
  "LinkTitle",
]);
const CALLOUT = /^(\s*>\s?)(\[!(note|tip|important|warning|caution)\])\s*$/i;

class Bullet extends WidgetType {
  eq() {
    return true;
  }
  toDOM(view: EditorView) {
    const one = view.dom.ownerDocument.createElement("span");
    one.className = "cm-bullet";
    one.textContent = "•";
    return one;
  }
}

class Check extends WidgetType {
  constructor(
    readonly done: boolean,
    readonly at: number,
  ) {
    super();
  }
  eq(other: WidgetType) {
    if (!(other instanceof Check)) return false;
    return other.done === this.done && other.at === this.at;
  }
  toDOM(view: EditorView) {
    const box = view.dom.ownerDocument.createElement("span");
    box.className = "cm-check";
    box.dataset.done = String(this.done);
    box.setAttribute("role", "checkbox");
    box.setAttribute("aria-checked", String(this.done));
    box.addEventListener("mousedown", (event) => {
      event.preventDefault();
      view.dispatch({
        changes: { from: this.at, to: this.at + 3, insert: this.done ? "[ ]" : "[x]" },
        userEvent: "input.check",
      });
    });
    return box;
  }
  ignoreEvent() {
    return true;
  }
}

class Said extends WidgetType {
  constructor(readonly kind: string) {
    super();
  }
  eq(other: WidgetType) {
    if (!(other instanceof Said)) return false;
    return other.kind === this.kind;
  }
  toDOM(view: EditorView) {
    const one = view.dom.ownerDocument.createElement("span");
    one.className = "cm-callout-said";
    one.textContent = t(`said${this.kind}` as Parameters<typeof t>[0]);
    return one;
  }
}

class Glyph extends WidgetType {
  constructor(
    readonly name: string,
    readonly hue?: string,
  ) {
    super();
  }
  eq(other: WidgetType) {
    if (!(other instanceof Glyph)) return false;
    return other.name === this.name && other.hue === this.hue;
  }
  toDOM(view: EditorView) {
    const page = view.dom.ownerDocument;
    const held = page.createElement("span");
    held.className = this.hue ? `cm-glyph ico-${this.hue}` : "cm-glyph";
    const svg = page.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("aria-hidden", "true");
    svg.innerHTML = markup(this.name) ?? "";
    held.append(svg);
    return held;
  }
}

const TAG = /(?<![\p{L}\p{N}#/\-_.:])#([\p{L}\p{N}][\p{L}\p{N}\p{M}_-]*)/gu;

// A tag reads as a chip once saved, so it wears one while it is written; «#1234» is a ticket, not a tag.
const tagged = (state: EditorState, from: number, to: number, out: Range<Decoration>[]) => {
  for (const found of state.sliceDoc(from, to).matchAll(TAG)) {
    const at = from + (found.index ?? 0);
    if (found[1].length < 2 || !/\p{L}/u.test(found[1]) || coded(state, at)) continue;
    out.push(Decoration.mark({ class: "cm-tag" }).range(at, at + found[0].length));
  }
};

// A format opens up while any selection touches it, edges included, so a letter typed there lands where it is seen.
const touched = (state: EditorState, from: number, to: number) =>
  state.selection.ranges.some((one) => one.from <= to && one.to >= from);

const spaced = (state: EditorState, at: number) => (state.sliceDoc(at, at + 1) === " " ? 1 : 0);

const lined = (
  state: EditorState,
  from: number,
  to: number,
  kind: string,
  out: Range<Decoration>[],
) => {
  for (let at = state.doc.lineAt(from).number; at <= state.doc.lineAt(to).number; at++) {
    out.push(Decoration.line({ class: kind }).range(state.doc.line(at).from));
  }
};

const quoted = (
  state: EditorState,
  from: number,
  to: number,
  shown: boolean,
  out: Range<Decoration>[],
) => {
  const first = state.doc.lineAt(from);
  const callout = CALLOUT.exec(first.text);
  if (!callout) return lined(state, from, to, "cm-quote", out);
  const kind = callout[3].toLowerCase();
  lined(state, from, to, `cm-callout cm-callout-${kind}`, out);
  if (shown) return;
  const head = first.from + callout[1].length;
  out.push(Decoration.replace({ widget: new Said(kind) }).range(head, head + callout[2].length));
};

const owned = (
  state: EditorState,
  from: number,
  to: number,
  shown: boolean,
  out: Range<Decoration>[],
) => {
  const text = state.sliceDoc(from, to);
  for (const found of text.matchAll(owning())) {
    const at = from + (found.index ?? 0);
    const end = at + found[0].length;
    if (shown || coded(state, at) || touched(state, at, end)) continue;
    if (found[4] !== undefined) {
      const hue = found[5];
      if (!known(found[4]) || (hue !== undefined && !HUES.includes(hue))) continue;
      out.push(Decoration.replace({ widget: new Glyph(found[4], hue) }).range(at, end));
      continue;
    }
    if (found[2] !== undefined && !HUES.includes(found[2])) continue;
    const tail = found[3] !== undefined ? 2 : found[0].length - 2 - found[1].length;
    out.push(Decoration.replace({}).range(at, at + 2));
    out.push(Decoration.replace({}).range(end - tail, end));
  }
};

const listed = (state: EditorState, mark: SyntaxNode, shown: boolean, out: Range<Decoration>[]) => {
  const item = mark.parent;
  const marker = item?.getChild("Task")?.getChild("TaskMarker");
  if (marker) {
    const done = /x/i.test(state.sliceDoc(marker.from, marker.to));
    if (done)
      out.push(Decoration.line({ class: "cm-done" }).range(state.doc.lineAt(mark.from).from));
    if (!shown) {
      const widget = new Check(done, marker.from);
      out.push(
        Decoration.replace({ widget }).range(mark.from, marker.to + spaced(state, marker.to)),
      );
    }
    return;
  }
  if (!shown && item?.parent?.name === "BulletList") {
    const widget = new Bullet();
    out.push(Decoration.replace({ widget }).range(mark.from, mark.to + spaced(state, mark.to)));
  }
};

// What the block shows while it is written: the page's own look, with Markdown's marks kept out of sight.
export const seen = (
  state: EditorState,
  ranges: readonly { from: number; to: number }[],
): DecorationSet => {
  const shown = state.field(marksShown, false) ?? false;
  const out: Range<Decoration>[] = [];
  const hide = (from: number, to: number) => {
    if (!shown && to > from && !state.sliceDoc(from, to).includes("\n"))
      out.push(Decoration.replace({}).range(from, to));
  };
  for (const { from, to } of ranges) {
    syntaxTree(state).iterate({
      from,
      to,
      enter: (node) => {
        const level = HEADING.exec(node.name);
        // A lone # may still become a tag, so it reads as a title only once a space follows it.
        if (level && /^#+$/.test(state.doc.lineAt(node.from).text)) return false;
        if (level)
          out.push(
            Decoration.line({ class: `cm-h${level[1]}` }).range(state.doc.lineAt(node.from).from),
          );
        if (node.name === "Image" || node.name === "FencedCode" || node.name === "CodeBlock")
          return false;
        if (node.name === "HeaderMark" && node.node.parent?.name.startsWith("ATXHeading")) {
          hide(node.from, node.to + spaced(state, node.to));
        }
        if (node.name === "Blockquote") quoted(state, node.from, node.to, shown, out);
        if (node.name === "QuoteMark") hide(node.from, node.to + spaced(state, node.to));
        if (node.name === "ListMark") listed(state, node.node, shown, out);
        if (node.name === "Link" && node.node.getChild("URL"))
          out.push(Decoration.mark({ class: "cm-link" }).range(node.from, node.to));
        if (MARKS.has(node.name)) {
          const parent = node.node.parent;
          if (parent?.name === "Link" && !parent.getChild("URL")) return true;
          if (parent && INLINE.has(parent.name) && !touched(state, parent.from, parent.to))
            hide(node.from, node.to);
        }
        return true;
      },
    });
    owned(state, from, to, shown, out);
    tagged(state, from, to, out);
  }
  return Decoration.set(out, true);
};

const seeing = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = seen(view.state, view.visibleRanges);
    }
    update(update: ViewUpdate) {
      const flipped = update.transactions.some((one) => one.effects.some((it) => it.is(showMarks)));
      if (update.docChanged || update.viewportChanged || update.selectionSet || flipped) {
        this.decorations = seen(update.state, update.view.visibleRanges);
      }
    }
  },
  { decorations: (one) => one.decorations },
);

export const visual = [
  marksShown,
  seeing,
  EditorView.atomicRanges.of((view) => {
    const all = view.plugin(seeing)?.decorations;
    if (!all) return Decoration.none;
    const out: Range<Decoration>[] = [];
    all.between(0, view.state.doc.length, (from, to, one) => {
      if (from < to && !one.spec.class) out.push(one.range(from, to));
    });
    return Decoration.set(out);
  }),
];
