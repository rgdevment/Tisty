import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { syntaxTree } from "@codemirror/language";
import {
  ChangeSet,
  EditorState,
  findClusterBreak,
  Transaction,
  type TransactionSpec,
} from "@codemirror/state";
import { coded, owning } from "./formatting";
import { pending } from "./pending";
import { fencePairs, INLINE, marksShown, ownPair, type Pair, revealing, type Span } from "./seeing";

const treePairs = (state: EditorState, from: number, to: number, out: Pair[]) => {
  const { main } = state.selection;
  syntaxTree(state).iterate({
    from,
    to,
    enter: (node) => {
      if (node.name === "Image" || node.name === "FencedCode" || node.name === "CodeBlock")
        return false;
      if (!INLINE.has(node.name)) return true;
      const first = node.node.firstChild;
      const last = node.node.lastChild;
      if (!first || !last) return true;
      if (node.name !== "Link") {
        out.push({ open: first, shut: last });
        return true;
      }
      const openByCaret = main.empty && main.from >= node.from && main.to <= node.to;
      const closing = node.node.getChildren("LinkMark")[1];
      if (node.node.getChild("URL") && closing && !openByCaret)
        out.push({ open: first, shut: { from: closing.from, to: node.to } });
      return true;
    },
  });
};

const inlinePairs = (state: EditorState, from: number, to: number): Pair[] => {
  const start = state.doc.lineAt(from).from;
  const end = state.doc.lineAt(to).to;
  const out: Pair[] = [];
  treePairs(state, from, to, out);
  for (const found of state.sliceDoc(start, end).matchAll(owning())) {
    const pair = ownPair(state, found, start + (found.index ?? 0));
    if (pair) out.push(pair);
  }
  return out;
};

const hiddenFences = (state: EditorState): Pair[] => {
  const { main } = state.selection;
  const onLine = (mark: Span) => {
    const line = state.doc.lineAt(mark.from);
    return main.empty && main.head >= line.from && main.head <= line.to;
  };
  return fencePairs(state).filter(({ open, shut }) => !onLine(open) && !onLine(shut));
};

const hit = (mark: Span, from: number, to: number) => mark.from < to && mark.to > from;

const merged = (spans: Span[]): Span[] => {
  const out: Span[] = [];
  for (const one of [...spans].sort((a, b) => a.from - b.from)) {
    const last = out[out.length - 1];
    if (last && one.from <= last.to)
      out[out.length - 1] = { from: last.from, to: Math.max(last.to, one.to) };
    else out.push({ from: one.from, to: one.to });
  }
  return out;
};

const without = (spans: Span[], kept: Span[]): Span[] =>
  kept.reduce<Span[]>(
    (left, mark) =>
      left.flatMap((one) =>
        hit(mark, one.from, one.to)
          ? [
              { from: one.from, to: mark.from },
              { from: mark.to, to: one.to },
            ].filter((part) => part.from < part.to)
          : [one],
      ),
    spans,
  );

const letterPast = (state: EditorState, at: number, forward: boolean): Span | null => {
  if (forward ? at >= state.doc.length : at <= 0) return null;
  const line = state.doc.lineAt(at);
  if (at === (forward ? line.to : line.from))
    return forward ? { from: at, to: at + 1 } : { from: at - 1, to: at };
  const other = line.from + findClusterBreak(line.text, at - line.from, forward);
  return forward ? { from: at, to: other } : { from: other, to: at };
};

const spared = (
  state: EditorState,
  from: number,
  to: number,
  inserted: boolean,
  forward: boolean | null,
): Span[] | null => {
  const going: Span[] = [{ from, to }];
  const kept: Span[] = [];
  for (const { open, shut } of [...inlinePairs(state, from, to), ...hiddenFences(state)]) {
    const inOpen = hit(open, from, to);
    const inShut = hit(shut, from, to);
    if (!inserted && from <= open.to && to >= shut.from) going.push(open, shut);
    else if (inOpen !== inShut) kept.push(inOpen ? open : shut);
  }
  if (going.length === 1 && !kept.length) return null;
  const whole = merged(going);
  const widened = whole.length === 1 && !inserted && whole[0];
  if (widened && (widened.from < from || widened.to > to))
    return spared(state, widened.from, widened.to, false, forward) ?? whole;
  const left = without(whole, kept);
  if (left.length || inserted || forward === null) return left;
  const edge = forward
    ? Math.max(to, ...kept.map((mark) => mark.to))
    : Math.min(from, ...kept.map((mark) => mark.from));
  const next = letterPast(state, edge, forward);
  if (!next) return [];
  return spared(state, next.from, next.to, false, forward) ?? [next];
};

// Written against a format's edge, a letter that would undo it goes inside it instead.
const insideFormat = (done: Transaction, at: number): number | null => {
  const state = done.startState;
  const edge = inlinePairs(state, at, at).filter(
    ({ open, shut }) => at === shut.to || at === open.from,
  );
  if (!edge.length) return null;
  const line = done.newDoc.lineAt(done.changes.mapPos(at, 1));
  const after = inlinePairs(done.state, line.from, line.to);
  const lost = edge.filter(
    ({ open }) => !after.some((one) => one.open.from === done.changes.mapPos(open.from, 1)),
  );
  if (!lost.length) return null;
  const closing = lost.filter(({ shut }) => at === shut.to).map(({ shut }) => shut.from);
  if (closing.length) return Math.min(...closing);
  return Math.max(...lost.map(({ open }) => open.to));
};

const GRAMMAR = markdown({ base: markdownLanguage, addKeymap: false });
const WORD_AHEAD = /^[\p{L}\p{M}\p{N}]+/u;
const WORD_BEHIND = /[\p{L}\p{M}\p{N}]+$/u;

const pairsIn = (doc: string) =>
  inlinePairs(EditorState.create({ doc, extensions: GRAMMAR }), 0, doc.length);

const closedFrom = (pairs: Pair[], at: number): number => {
  const closing = pairs.find(({ shut }) => shut.to === at);
  return closing ? closedFrom(pairs, closing.shut.from) : at;
};

const openedTo = (pairs: Pair[], at: number): number => {
  const opening = pairs.find(({ open }) => open.from === at);
  return opening ? openedTo(pairs, opening.open.to) : at;
};

// A word brought up against a format's marks can undo it, so the format takes the word in.
export const heldTogether = (
  left: string,
  right: string,
): { left: string; right: string } | null => {
  if (!/\S$/.test(left) || !/^\S/.test(right)) return null;
  const ahead = WORD_AHEAD.exec(right)?.[0];
  const behind = WORD_BEHIND.exec(left)?.[0];
  if (Boolean(ahead) === Boolean(behind)) return null;
  const before = pairsIn(left);
  const after = pairsIn(right);
  const count = before.length + after.length;
  if (pairsIn(left + right).length >= count) return null;
  const closing = closedFrom(before, left.length);
  const opening = openedTo(after, 0);
  const moved = ahead
    ? {
        left: left.slice(0, closing),
        right: `${ahead}${left.slice(closing)}${right.slice(ahead.length)}`,
      }
    : {
        left: `${left.slice(0, left.length - (behind ?? "").length)}${right.slice(0, opening)}${behind}`,
        right: right.slice(opening),
      };
  return pairsIn(moved.left + moved.right).length >= count ? moved : null;
};

const heldAfter = (done: Transaction): TransactionSpec | null => {
  const seams: number[] = [];
  done.changes.iterChangedRanges((_fromA, _toA, fromB) => seams.push(fromB));
  for (const seam of seams) {
    const line = done.newDoc.lineAt(seam);
    const cut = seam - line.from;
    const held = heldTogether(line.text.slice(0, cut), line.text.slice(cut));
    if (held && !coded(done.state, seam))
      return {
        changes: done.changes.compose(
          ChangeSet.of(
            { from: line.from, to: line.to, insert: held.left + held.right },
            done.newDoc.length,
          ),
        ),
        selection: { anchor: line.from + held.left.length },
        effects: done.effects,
        scrollIntoView: done.scrollIntoView,
        annotations: Transaction.userEvent.of(done.annotation(Transaction.userEvent) ?? "delete"),
      };
  }
  return null;
};

const marksSpared = (done: Transaction): Transaction | TransactionSpec => {
  const state = done.startState;
  const changed: { from: number; to: number; insert: string }[] = [];
  done.changes.iterChanges((from, to, _a, _b, text) => {
    changed.push({ from, to, insert: text.sliceString(0, undefined, state.lineBreak) });
  });
  const only = changed[0];
  if (changed.length !== 1 || !only) return done;
  if (only.from === only.to) {
    const inside = done.isUserEvent("input.type") ? insideFormat(done, only.from) : null;
    if (inside === null) return done;
    return {
      changes: { from: inside, insert: only.insert },
      selection: { anchor: inside + state.toText(only.insert).length },
      scrollIntoView: done.scrollIntoView,
      annotations: Transaction.userEvent.of("input.type"),
    };
  }
  const forward = done.isUserEvent("delete.forward");
  const walks = state.selection.main.empty ? forward : null;
  const left = spared(state, only.from, only.to, Boolean(only.insert), walks);
  if (!left) return done;
  const at = left[0]?.from ?? only.from;
  const changes = state.changes([
    ...left,
    ...(only.insert ? [{ from: at, insert: only.insert }] : []),
  ]);
  const { main } = state.selection;
  const anchor =
    main.empty && !only.insert
      ? changes.mapPos(main.head, forward ? -1 : 1)
      : at + state.toText(only.insert).length;
  return {
    changes,
    selection: { anchor },
    effects: done.effects,
    scrollIntoView: done.scrollIntoView,
    annotations: Transaction.userEvent.of(done.annotation(Transaction.userEvent) ?? "delete"),
  };
};

// With the marks out of sight a deletion must not leave half a format lying in the words.
export const pairsKept = EditorState.transactionFilter.of((done) => {
  const state = done.startState;
  if (!done.docChanged) return done;
  if (done.isUserEvent("input.type.compose")) return done;
  const deleting = done.isUserEvent("delete");
  if (!deleting && !done.isUserEvent("input.type") && !done.isUserEvent("input.paste")) return done;
  if (state.field(marksShown, false) || state.facet(revealing) || state.field(pending, false))
    return done;
  const spared = marksSpared(done);
  if (!deleting) return spared;
  const applied =
    spared instanceof Transaction ? spared : state.update({ ...spared, filter: false });
  return heldAfter(applied) ?? spared;
});

export const outOfPairs = (state: EditorState, head: number): number | null => {
  let at = head;
  for (let moved = true; moved; ) {
    moved = false;
    for (const { open, shut } of inlinePairs(state, at, at)) {
      if (at <= open.from || at >= shut.to) continue;
      if (at === shut.from) at = shut.to;
      else if (at === open.to) at = open.from;
      else return null;
      moved = true;
      break;
    }
  }
  return at;
};
