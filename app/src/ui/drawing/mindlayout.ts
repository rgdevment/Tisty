import type { Size } from "./framing";
import type { Path } from "./minding";
import type { Idea, Mind } from "./mindmap";

export interface Placed {
  readonly path: Path;
  readonly idea: Idea;
  readonly depth: number;
  // Which first branch it hangs from, so a branch keeps one colour; the root has none.
  readonly hue: number;
  readonly x: number;
  readonly y: number;
  readonly wide: number;
  readonly tall: number;
}

export interface Line {
  readonly from: { readonly x: number; readonly y: number };
  readonly to: { readonly x: number; readonly y: number };
  readonly hue: number;
}

export interface Laid {
  readonly ideas: readonly Placed[];
  readonly lines: readonly Line[];
  readonly size: Size;
}

const ROW = 46;
const GAP = 46;
const MARGIN = 40;

// Measuring text needs a laid-out page; a width per character is close enough for a map that places itself.
export const wideOf = (text: string, depth: number): number =>
  Math.round(Math.max(text.length, 4) * (depth === 0 ? 9 : 7.4) + (depth === 0 ? 48 : 30));

const tallOf = (depth: number) => (depth === 0 ? 50 : 34);

const leaves = (idea: Idea): number =>
  idea.children.length ? idea.children.reduce((sum, one) => sum + leaves(one), 0) : 1;

export const curve = ({ from, to }: Line): string => {
  const middle = (from.x + to.x) / 2;
  return `M ${from.x} ${from.y} C ${middle} ${from.y}, ${middle} ${to.y}, ${to.x} ${to.y}`;
};

interface Hanging {
  readonly idea: Idea;
  readonly path: Path;
  readonly top: number;
  readonly edge: number;
  readonly from: number;
  readonly side: number;
  readonly hue: number;
}

/** Root in the middle, first branches taking turns right and left, each branch as tall as its leaves. */
export const laidOut = (mind: Mind): Laid => {
  const ideas: Placed[] = [];
  const lines: Line[] = [];
  const root = mind.root;
  const rootWide = wideOf(root.text, 0);
  ideas.push({
    path: [],
    idea: root,
    depth: 0,
    hue: -1,
    x: -rootWide / 2,
    y: -25,
    wide: rootWide,
    tall: 50,
  });

  const place = ({ idea, path, top, edge, from, side, hue }: Hanging) => {
    const depth = path.length;
    const y = top + (leaves(idea) * ROW) / 2;
    const wide = wideOf(idea.text, depth);
    const x = side > 0 ? edge + GAP : edge - GAP - wide;
    lines.push({ from: { x: edge, y: from }, to: { x: side > 0 ? x : x + wide, y }, hue });
    ideas.push({ path, idea, depth, hue, x, y: y - tallOf(depth) / 2, wide, tall: tallOf(depth) });
    let below = top;
    idea.children.forEach((child, at) => {
      const outer = side > 0 ? x + wide : x;
      place({ idea: child, path: [...path, at], top: below, edge: outer, from: y, side, hue });
      below += leaves(child) * ROW;
    });
  };

  for (const side of [1, -1]) {
    const mine = root.children
      .map((idea, at) => ({ idea, at }))
      .filter(({ at }) => (at % 2 === 0 ? 1 : -1) === side);
    let top = -(mine.reduce((sum, { idea }) => sum + leaves(idea), 0) * ROW) / 2;
    for (const { idea, at } of mine) {
      place({ idea, path: [at], top, edge: (side * rootWide) / 2, from: 0, side, hue: at });
      top += leaves(idea) * ROW;
    }
  }
  return framed(ideas, lines);
};

// Everything moves so the map starts at the margin: the board knows nothing of negative room.
const framed = (ideas: Placed[], lines: Line[]): Laid => {
  const left = Math.min(...ideas.map((one) => one.x)) - MARGIN;
  const top = Math.min(...ideas.map((one) => one.y)) - MARGIN;
  const right = Math.max(...ideas.map((one) => one.x + one.wide)) + MARGIN;
  const bottom = Math.max(...ideas.map((one) => one.y + one.tall)) + MARGIN;
  const shifted = (at: { x: number; y: number }) => ({ x: at.x - left, y: at.y - top });
  return {
    ideas: ideas.map((one) => ({ ...one, x: one.x - left, y: one.y - top })),
    lines: lines.map((one) => ({ ...one, from: shifted(one.from), to: shifted(one.to) })),
    size: { wide: right - left, tall: bottom - top },
  };
};
