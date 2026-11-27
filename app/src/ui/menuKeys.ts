import { folded } from "../folding";

export const asksForMenu = (e: React.KeyboardEvent) =>
  e.key === "ContextMenu" || (e.shiftKey && e.key === "F10");

export const menuAt = (e: React.KeyboardEvent) => {
  const box = (e.target as HTMLElement).getBoundingClientRect();
  return { x: box.left + 24, y: box.bottom };
};

/** Where a menu answers from: `y` is the edge it hangs down from, `top` the line's own top, `right` its right edge. */
export type Anchor = { x: number; y: number; top?: number; right?: number };

export type Size = { wide: number; tall: number };

/** A menu hangs from its `top`, or stands on its `bottom` (counted up from the window's foot), never both. */
export type Placed = { x: number; tall: number; top?: number; bottom?: number };

export const EDGE = 6;
const GAP = 4;
const TALLEST = 0.8;
const SHORTEST = 120;
const ROOMY_BELOW = 240;

// Put up, it stands on the line, so a list shortened by a search stays touching it rather than floating away.
export const placement = (
  at: Anchor,
  card: Size,
  view: Size,
  ceiling = EDGE,
  up = false,
  whole = false,
): Placed => {
  const tall = Math.min(card.tall, Math.round(view.tall * TALLEST));
  const top = at.top ?? at.y;
  const below = view.tall - EDGE - at.y;
  const above = top - GAP - ceiling;
  const rightward = at.right !== undefined && at.x > view.wide / 2;
  const left = rightward ? (at.right ?? at.x) - card.wide : at.x;
  const x = Math.max(EDGE, Math.min(left, view.wide - card.wide - EDGE));
  const over = (high: number): Placed => ({ x, bottom: view.tall - (top - GAP), tall: high });
  const under = (high: number): Placed => ({ x, top: at.y, tall: high });
  if (up && tall <= above) return over(tall);
  if (tall <= below) return under(tall);
  if (!whole && below >= ROOMY_BELOW) return under(below);
  if (tall <= above) return over(tall);
  if (Math.max(below, above) < SHORTEST)
    return { x, top: Math.max(EDGE, view.tall - tall - EDGE), tall };
  return below >= above ? under(below) : over(above);
};

/** A list within a list opens beside the row that holds it, to the right if it fits, to the left if not. */
export const beside = (
  row: { left: number; right: number; top: number },
  card: Size,
  view: Size,
): Placed => {
  const tall = Math.min(card.tall, view.tall - 2 * EDGE);
  const fits = row.right + card.wide <= view.wide - EDGE;
  const x = fits ? row.right : Math.max(EDGE, row.left - card.wide);
  const top = Math.max(EDGE, Math.min(row.top, view.tall - EDGE - tall));
  return { x, top, tall };
};

/** The next row the arrows land on, round from the last to the first. */
export const stepped = (count: number, from: number, by: number): number =>
  count === 0 ? -1 : (((from + by) % count) + count) % count;

/** The first row after `from` whose label starts with the letter typed, round from the top. */
export const typedTo = (labels: readonly string[], from: number, letter: string): number => {
  const sought = folded(letter);
  for (let step = 1; step <= labels.length; step++) {
    const at = (from + step) % labels.length;
    if (folded(labels[at]).startsWith(sought)) return at;
  }
  return -1;
};

/** A row is found when what was sought, folded, is anywhere in its words. */
export const found = (words: string, sought: string): boolean =>
  !sought || folded(words).includes(sought);

const begins = (words: string, sought: string) =>
  folded(words)
    .split(/[\s/.,;:()«»"'¿?¡!-]+/)
    .some((word) => word.startsWith(sought));

// A word begun by what was sought beats one only holding it, and a row that acts beats one that opens a list.
export const leadingOf = (
  rows: readonly { words: string; opens: boolean }[],
  sought: string,
): number => {
  const begun = rows.map((row) => !sought || begins(row.words, sought));
  const best = begun.some(Boolean) ? begun : rows.map(() => true);
  const plain = rows.findIndex((row, at) => best[at] && !row.opens);
  if (plain >= 0) return plain;
  return Math.max(0, best.indexOf(true));
};
