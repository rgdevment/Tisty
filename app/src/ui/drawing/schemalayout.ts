import type { Size } from "./framing";
import type { Relation, Schema, Table } from "./schema";
import { relationsOf } from "./scheming";

export interface Box {
  readonly table: Table;
  readonly x: number;
  readonly y: number;
  readonly wide: number;
  readonly tall: number;
}

export interface Link {
  readonly index: number;
  readonly relation: Relation;
  readonly from: { readonly x: number; readonly y: number };
  readonly to: { readonly x: number; readonly y: number };
}

export interface Drawn {
  readonly boxes: readonly Box[];
  readonly links: readonly Link[];
  readonly size: Size;
}

export const HEAD = 34;
export const ROW = 26;
const MARGIN = 40;
const CELL = 280;
const PER_ROW = 3;

// Measuring text needs a laid-out page; a width per character is close enough for a table card.
const wideOf = (table: Table) =>
  Math.max(
    180,
    table.name.length * 8 + 40,
    ...table.columns.map(
      (one) => (one.name.length + one.type.length + one.keys.join(",").length) * 7 + 70,
    ),
  );

const tallOf = (table: Table) => HEAD + Math.max(table.columns.length, 1) * ROW + 6;

/** Each table where its `%% tisty:` line puts it; one without a line in a grid to the right of the others. */
export const boxesOf = (schema: Schema): Box[] => {
  const kept = schema.tables.filter((one) => one.at);
  const right = kept.reduce((most, one) => Math.max(most, (one.at?.x ?? 0) + wideOf(one)), 0);
  let x = kept.length ? right + 80 : MARGIN;
  const start = x;
  let y = MARGIN;
  let tallest = 0;
  let inRow = 0;
  return schema.tables.map((table) => {
    const wide = wideOf(table);
    const tall = tallOf(table);
    if (table.at)
      return { table, x: Math.max(0, table.at.x), y: Math.max(0, table.at.y), wide, tall };
    const box = { table, x, y, wide, tall };
    tallest = Math.max(tallest, tall);
    inRow += 1;
    x += CELL;
    if (inRow === PER_ROW) {
      x = start;
      y += tallest + 60;
      tallest = 0;
      inRow = 0;
    }
    return box;
  });
};

// A relation leaves the side of one table that faces the other.
const linkOf = (index: number, relation: Relation, a: Box, b: Box): Link => {
  const rightward = a.x + a.wide / 2 <= b.x + b.wide / 2;
  return {
    index,
    relation,
    from: { x: rightward ? a.x + a.wide : a.x, y: a.y + HEAD / 2 },
    to: { x: rightward ? b.x : b.x + b.wide, y: b.y + HEAD / 2 },
  };
};

export const drawnOf = (schema: Schema): Drawn => {
  const boxes = boxesOf(schema);
  const byName = new Map(boxes.map((one) => [one.table.name, one]));
  const links = relationsOf(schema).flatMap((relation, index) => {
    const a = byName.get(relation.from);
    const b = byName.get(relation.to);
    return a && b ? [linkOf(index, relation, a, b)] : [];
  });
  const wide = Math.max(MARGIN * 2, ...boxes.map((one) => one.x + one.wide + MARGIN));
  const tall = Math.max(MARGIN * 2, ...boxes.map((one) => one.y + one.tall + MARGIN));
  return { boxes, links, size: { wide, tall } };
};

export const bend = ({ from, to }: Pick<Link, "from" | "to">): string => {
  const reach = Math.max(40, Math.abs(to.x - from.x) / 2);
  const out = from.x <= to.x ? reach : -reach;
  return `M ${from.x} ${from.y} C ${from.x + out} ${from.y}, ${to.x - out} ${to.y}, ${to.x} ${to.y}`;
};
