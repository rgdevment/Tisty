import {
  type ChangeSpec,
  EditorState,
  type Range,
  StateEffect,
  StateField,
  type Text,
  type Transaction,
} from "@codemirror/state";
import { Decoration, type EditorView, type KeyBinding } from "@codemirror/view";

// A format pressed with nothing selected: the words typed between its marks, from `from` to `to`.
export type Pending = {
  readonly from: number;
  readonly to: number;
  readonly mark: string;
  readonly close: string;
};

export const pend = StateEffect.define<Pending | null>();

export const pending = StateField.define<Pending | null>({
  create: () => null,
  update: (was, done) => {
    for (const one of done.effects) if (one.is(pend)) return one.value;
    if (!was || !done.docChanged) return was;
    return {
      ...was,
      from: done.changes.mapPos(was.from, -1),
      to: done.changes.mapPos(was.to, 1),
    };
  },
});

// Spaces typed at the edges go outside the marks, and marks left with nothing between them go away.
// When the format ends early, at `end`, the words after it stay plain.
const tidied = (doc: Text, one: Pending, end = one.to): ChangeSpec[] => {
  const open = one.from - one.mark.length;
  const shut = one.to + one.close.length;
  const body = doc.sliceString(one.from, end);
  const moved = end < one.to;
  const rest: ChangeSpec[] = moved ? [{ from: one.to, to: shut }] : [];
  if (!body.trim()) {
    if (moved) return [{ from: open, to: one.from }, ...rest];
    return [{ from: open, to: shut, insert: body }];
  }
  const lead = body.length - body.trimStart().length;
  const trail = body.length - body.trimEnd().length;
  const trailing = body.slice(body.length - trail);
  const opening: ChangeSpec[] = lead
    ? [{ from: open, to: one.from + lead, insert: body.slice(0, lead) + one.mark }]
    : [];
  if (!moved) {
    if (!lead && !trail) return [];
    return [...opening, { from: end - trail, to: shut, insert: one.close + trailing }];
  }
  return [...opening, { from: end - trail, to: end, insert: one.close + trailing }, ...rest];
};

const left = (done: Transaction, one: Pending) => {
  const { head } = done.newSelection.main;
  return head < one.from || head > one.to;
};

const settling = EditorState.transactionFilter.of((done) => {
  if (done.effects.some((one) => one.is(pend))) return done;
  const was = done.startState.field(pending, false);
  if (!was) return done;
  const one = {
    ...was,
    from: done.changes.mapPos(was.from, -1),
    to: done.changes.mapPos(was.to, 1),
  };
  if (!left(done, one)) return done;
  return [done, { changes: tidied(done.newDoc, one), effects: pend.of(null), sequential: true }];
});

export const settle = (view: EditorView) => {
  const one = view.state.field(pending, false);
  if (!one) return;
  view.dispatch({ changes: tidied(view.state.doc, one), effects: pend.of(null) });
};

// The same format pressed again while it waits: a pen changes colour, anything else steps out of it.
export const settled = (view: EditorView, close?: string) => {
  const one = view.state.field(pending, false);
  if (!one) return;
  if (close !== undefined) {
    view.dispatch({
      changes: { from: one.to, to: one.to + one.close.length, insert: close },
      effects: pend.of({ ...one, close }),
    });
    return;
  }
  const changes = view.state.changes(tidied(view.state.doc, one));
  view.dispatch({
    changes,
    effects: pend.of(null),
    selection: { anchor: changes.mapPos(one.to + one.close.length, 1) },
  });
};

const inside = (view: EditorView) => {
  const one = view.state.field(pending, false);
  const { main } = view.state.selection;
  if (!one || !main.empty || main.head < one.from || main.head > one.to) return undefined;
  return one;
};

export const pendingKeys: KeyBinding[] = [
  {
    // A new line ends the format where the caret is, so marks never span two lines.
    key: "Enter",
    run: (view) => {
      const one = inside(view);
      if (!one) return false;
      const at = view.state.selection.main.head;
      const changes = view.state.changes(tidied(view.state.doc, one, at));
      view.dispatch({
        changes,
        effects: pend.of(null),
        selection: { anchor: changes.mapPos(at, 1) },
      });
      return false;
    },
  },
  {
    key: "Backspace",
    run: (view) => {
      const one = inside(view);
      if (!one || one.from !== one.to) return false;
      view.dispatch({
        changes: { from: one.from - one.mark.length, to: one.to + one.close.length },
        effects: pend.of(null),
      });
      return true;
    },
  },
];

const LOOK: Record<string, string> = {
  "**": "cm-pending-strong",
  "*": "cm-pending-em",
  "~~": "cm-pending-struck",
  "++": "cm-under",
  "`": "cm-pending-code",
};

// While its words are typed, the format already looks as it will, its marks out of sight.
export const waiting = (state: EditorState, out: Range<Decoration>[]) => {
  const one = state.field(pending, false);
  if (!one) return;
  out.push(Decoration.replace({}).range(one.from - one.mark.length, one.from));
  out.push(Decoration.replace({}).range(one.to, one.to + one.close.length));
  const hue = one.mark === "==" ? (/\{([a-z]+)\}/.exec(one.close)?.[1] ?? "yellow") : undefined;
  const look = hue ? `cm-pen cm-pen-${hue}` : LOOK[one.mark];
  if (look && one.to > one.from) out.push(Decoration.mark({ class: look }).range(one.from, one.to));
};

export const pendingFormat = [pending, settling];
