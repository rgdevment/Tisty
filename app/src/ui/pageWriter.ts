import type { Asking } from "tisty-editor/edit";
import type { PlacedNear } from "../placing";
import type { Anchor } from "./dressing";
import { changeOf, type Span } from "./pageEdits";
import type { Marks } from "./pageMarks";

/** `lead` and `tail` wrap the first key of a line begun from a hint or a button, written only once a key comes. */
export type Kept = {
  readonly node: Node;
  readonly offset: number;
  readonly at: number;
  readonly lead?: string;
  readonly tail?: string;
};

export type Turn = {
  readonly from: number;
  readonly removed: string;
  readonly added: string;
  readonly at: number;
  readonly after?: number;
  readonly typed?: number;
};

/** What Ctrl+Z walks back through, kept by whoever shows the page so a turn to the Markdown keeps it. */
export type Turns = { readonly undone: Turn[]; readonly redone: Turn[]; left?: string };

export const turnsAnew = (): Turns => ({ undone: [], redone: [] });

export const turnOf = (was: string, now: string, at?: number, after?: number): Turn => {
  const one = changeOf(was, now);
  const removed = was.slice(one.from, one.to);
  return { from: one.from, removed, added: one.insert, at: at ?? one.from, after };
};

const TYPED_TOGETHER_MS = 1000;

// Keys typed one after another, with no pause and no new line, are walked back as one step.
const joined = (last: Turn, next: Turn): Turn | null => {
  if (last.typed === undefined || next.typed === undefined) return null;
  if (next.typed - last.typed > TYPED_TOGETHER_MS) return null;
  const end = last.from + last.added.length;
  const later = { after: next.after, typed: next.typed };
  const lined = next.added.includes("\n") || last.added.includes("\n");
  if (!next.removed && next.from === end && !lined)
    return { ...last, ...later, added: last.added + next.added };
  if (last.added || next.added) return null;
  if (next.from + next.removed.length === last.from)
    return { ...last, ...later, from: next.from, removed: next.removed + last.removed };
  if (next.from === last.from) return { ...last, ...later, removed: last.removed + next.removed };
  return null;
};

export const recorded = (turns: Turns, turn: Turn) => {
  turns.redone.length = 0;
  const last = turns.undone[turns.undone.length - 1];
  const both = last && joined(last, turn);
  if (both) turns.undone[turns.undone.length - 1] = both;
  else turns.undone.push(turn);
};

export type Point = { readonly x: number; readonly y: number; readonly top?: number };

export type Hands = {
  readonly turns?: Turns;
  /** A block, the window or the page was left: what was written there is told once, never each key. */
  readonly settle?: () => void;
  readonly text: () => string;
  readonly write: (next: string) => void;
  readonly shown: (now: string) => void;
  readonly redraw: (now: string) => void;
  readonly near: (body: string, from: number, to: number) => Promise<PlacedNear>;
  readonly dressed: (sheet: Element, laid: readonly Element[]) => void;
  readonly translated: (html: string | null, plain: string) => Promise<string>;
  readonly ask: Asking["ask"];
  readonly blocks: (at: Point, slash: boolean) => void;
  readonly acts: (at: Anchor, top: Element) => void;
  readonly tags?: () => readonly string[];
  readonly pasted?: (files: readonly File[]) => Promise<string>;
};

export type Pending = { readonly at: number; readonly marks: Marks };
export type Carried = { readonly from: number; readonly close: string; readonly caret: number };

export type Writer = {
  readonly page: Document;
  readonly hands: Hands;
  readonly turns: Turns;
  kept: Kept | null;
  pending: Pending | null;
  carried: Carried | null;
  typing: boolean;
  whole: boolean;
  lastChosen: Span | null;
  leftFrom: string | null | undefined;
};

export const writerOf = (page: Document, hands: Hands): Writer => ({
  page,
  hands,
  turns: hands.turns ?? turnsAnew(),
  kept: null,
  pending: null,
  carried: null,
  typing: false,
  whole: false,
  lastChosen: null,
  leftFrom: undefined,
});
