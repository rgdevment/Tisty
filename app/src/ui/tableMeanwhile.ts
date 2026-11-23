import { charsAt } from "tisty-editor/read";
import { applied, changeOf } from "./pageEdits";
import { wordsAsked } from "./pageLate";

const BYTES = new TextEncoder();

const grownBy = (was: string, now: string) => {
  const moved = changeOf(was, now);
  return BYTES.encode(moved.insert).length - BYTES.encode(was.slice(moved.from, moved.to)).length;
};

// The table's end in the text after its own edit, the only change between the two.
export const endAfter = (was: string, next: string, at: readonly [number, number]) =>
  charsAt(next, at[1] + grownBy(was, next));

export const laidOver = (was: string, next: string, now: string): string | null => {
  if (now === was || now === next) return next;
  const ours = changeOf(was, next);
  const span = wordsAsked(was, ours)(now);
  return span && applied(now, { ...span, insert: ours.insert });
};

export const rowAskedAgain = (
  was: string,
  now: string,
  at: readonly [number, number],
): [number, number] | null => {
  if (now === was) return null;
  const theirs = changeOf(was, now);
  const end = charsAt(was, at[1]);
  const inside = charsAt(was, at[0]) <= theirs.from && theirs.from < end && theirs.to <= end;
  const lined = was.slice(theirs.from, theirs.to).includes("\n") || theirs.insert.includes("\n");
  if (!inside || lined) return null;
  return [at[0], at[1] + grownBy(was, now)];
};
