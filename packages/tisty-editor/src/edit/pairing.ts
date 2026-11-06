import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { syntaxTree } from "@codemirror/language";
import {
  ChangeSet,
  EditorState,
  type Extension,
  findClusterBreak,
  StateEffect,
  StateField,
  type Text,
  Transaction,
  type TransactionSpec,
} from "@codemirror/state";
import { frontLines } from "../front";
import { formless, owning } from "./formatting";
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

export const GRAMMAR = markdown({ base: markdownLanguage });

type Seam = { caret: number; seam: number; back: ChangeSet };
type Carried = { fix: ChangeSet; caret: number; seam: number; outside: boolean };
type Lost = { kept: Pair[]; lost: Pair[]; seams: number[] };

const carriedOff = StateEffect.define<Seam>();

// Only the very next key may give back the space that was deleted.
const lastSeam = StateField.define<Seam | null>({
  create: () => null,
  update: (was, done) => {
    for (const one of done.effects) if (one.is(carriedOff)) return one.value;
    return done.docChanged || done.selection ? null : was;
  },
});

// A link is left out: it holds beside anything, and it is not told while the caret stands in it.
const formatsOn = (state: EditorState, from: number, to: number): Pair[] =>
  inlinePairs(state, state.doc.lineAt(from).from, state.doc.lineAt(to).to).filter(
    ({ open }) => state.sliceDoc(open.from, open.to) !== "[",
  );

const lostFormats = (done: Transaction): Lost | null => {
  const ranges: { fromA: number; toA: number; fromB: number; toB: number }[] = [];
  done.changes.iterChangedRanges((fromA, toA, fromB, toB) => {
    ranges.push({ fromA, toA, fromB, toB });
  });
  const first = ranges[0];
  const last = ranges[ranges.length - 1];
  if (!first || !last) return null;
  const mapped = ({ from, to }: Span) => ({
    from: done.changes.mapPos(from, 1),
    to: done.changes.mapPos(to, -1),
  });
  const kept = formatsOn(done.startState, first.fromA, last.toA)
    .filter(({ open, shut }) =>
      ranges.every(({ fromA, toA }) => !hit(open, fromA, toA) && !hit(shut, fromA, toA)),
    )
    .map(({ open, shut }) => ({ open: mapped(open), shut: mapped(shut) }));
  if (!kept.length) return null;
  const now = formatsOn(done.state, first.fromB, last.toB);
  const lost = kept.filter(
    ({ open, shut }) =>
      !now.some((one) => one.open.from === open.from && one.shut.from === shut.from),
  );
  return lost.length ? { kept, lost, seams: ranges.map(({ fromB }) => fromB) } : null;
};

const touching = (marks: Span[], at: number): Span | undefined =>
  merged(marks).find(({ from, to }) => from <= at && at <= to);

const spaceAt = (doc: Text, at: number): boolean =>
  at < 0 || at >= doc.length || /\s/.test(doc.sliceString(at, at + 1));

const wordEnd = (doc: Text, from: number, formats: Pair[]): number => {
  const line = doc.lineAt(from);
  const space = line.text.slice(from - line.from).search(/\s/);
  const opens = formats.map(({ open }) => open.from).filter((at) => at >= from);
  return Math.min(space < 0 ? line.to : from + space, ...opens);
};

const wordStart = (doc: Text, to: number, formats: Pair[]): number => {
  const line = doc.lineAt(to);
  const space = line.text.slice(0, to - line.from).search(/\s\S*$/);
  const shuts = formats.map(({ shut }) => shut.to).filter((at) => at <= to);
  return Math.max(space < 0 ? line.from : line.from + space + 1, ...shuts);
};

const inFront = (doc: Text, at: number): boolean =>
  doc.lineAt(at).number <= (frontLines(doc.toString())?.length ?? 0);

const carriedAt = (done: Transaction, { kept, lost }: Lost, seam: number): Carried | null => {
  const doc = done.newDoc;
  const free = (run?: Span): run is Span =>
    run !== undefined && !spaceAt(doc, run.from - 1) && !spaceAt(doc, run.to);
  const closing = touching(
    lost.map(({ shut }) => shut),
    seam,
  );
  const opening = touching(
    lost.map(({ open }) => open),
    seam,
  );
  const forward = free(closing);
  const run = forward ? closing : opening;
  if (!free(run)) return null;
  const marks = doc.sliceString(run.from, run.to);
  const edge = forward ? wordEnd(doc, run.to, kept) : wordStart(doc, run.from, kept);
  const far = forward
    ? { from: edge, to: Math.min(edge + marks.length, doc.length) }
    : { from: Math.max(edge - marks.length, 0), to: edge };
  const joining = doc.sliceString(far.from, far.to) === marks;
  const fix = ChangeSet.of([run, joining ? far : { from: edge, insert: marks }], doc.length);
  const taken = lost.filter(({ open, shut }) => hit(forward ? shut : open, run.from, run.to));
  const at = fix.mapPos(seam);
  const now = done.state.update({ changes: fix, filter: false }).state;
  if (formatsOn(now, at, at).length < kept.length - (joining ? taken.length : 0)) return null;
  return {
    fix,
    seam,
    caret: fix.mapPos(done.newSelection.main.head),
    outside: seam === (forward ? run.to : run.from),
  };
};

const heldAfter = (done: Transaction): Carried | null => {
  const found = lostFormats(done);
  if (!found || formless(done.state)) return null;
  for (const seam of found.seams) {
    const carried = inFront(done.newDoc, seam) ? null : carriedAt(done, found, seam);
    if (carried) return carried;
  }
  return null;
};

// A word brought up against a format's marks can undo it, so the format takes the word in.
export const heldTogether = (
  head: string,
  rest: string,
): { left: string; right: string } | null => {
  if (!head.slice(-1).trim() || !rest.slice(0, 1).trim()) return null;
  const state = EditorState.create({ doc: `${head}\n\n${rest}`, extensions: GRAMMAR });
  const at = state.toText(head).length;
  const done = state.update({
    changes: { from: at, to: at + 2 },
    selection: { anchor: at },
    filter: false,
  });
  const carried = heldAfter(done);
  if (!carried) return null;
  const line = carried.fix.apply(done.newDoc).lineAt(carried.caret);
  const cut = carried.caret - line.from;
  return { left: line.text.slice(0, cut), right: line.text.slice(cut) };
};

const respaced = (done: Transaction): TransactionSpec | null => {
  const last = done.startState.field(lastSeam, false);
  if (!last || !done.isUserEvent("input.type")) return null;
  const typed: { from: number; to: number; text: string }[] = [];
  done.changes.iterChanges((from, to, _a, _b, text) => {
    typed.push({ from, to, text: text.toString() });
  });
  const only = typed[0];
  if (typed.length !== 1 || !only || only.from !== last.caret || only.to !== last.caret)
    return null;
  if (!/^\s+$/.test(only.text)) return null;
  return {
    changes: last.back.compose(
      ChangeSet.of({ from: last.seam, insert: only.text }, last.back.newLength),
    ),
    selection: { anchor: last.seam + only.text.length },
    scrollIntoView: done.scrollIntoView,
    annotations: Transaction.userEvent.of("input.type"),
  };
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
const keepingPairs = EditorState.transactionFilter.of((done) => {
  const state = done.startState;
  if (!done.docChanged) return done;
  if (done.isUserEvent("input.type.compose")) return done;
  const deleting = done.isUserEvent("delete");
  if (!deleting && !done.isUserEvent("input.type") && !done.isUserEvent("input.paste")) return done;
  if (state.field(marksShown, false) || state.facet(revealing) || state.field(pending, false))
    return done;
  const spaced = respaced(done);
  if (spaced) return spaced;
  const spared = marksSpared(done);
  if (!deleting) return spared;
  const applied =
    spared instanceof Transaction ? spared : state.update({ ...spared, filter: false });
  const carried = heldAfter(applied);
  if (!carried) return spared;
  const { fix, caret, seam, outside } = carried;
  const remembered = outside
    ? [carriedOff.of({ caret, seam, back: fix.invert(applied.newDoc) })]
    : [];
  return {
    changes: applied.changes.compose(fix),
    selection: { anchor: caret },
    effects: [...applied.effects, ...remembered],
    scrollIntoView: applied.scrollIntoView,
    annotations: Transaction.userEvent.of(applied.annotation(Transaction.userEvent) ?? "delete"),
  };
});

export const pairsKept: Extension = [lastSeam, keepingPairs];

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
