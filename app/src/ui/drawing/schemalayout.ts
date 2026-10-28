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

/** Each table where its `%% tisty:` line puts it; one without a line in its own cell of a grid. */
export const boxesOf = (schema: Schema): Box[] => {
  // Every table keeps the cell its order gives it, placed or not, so placing one never moves another.
  const rows: number[] = [];
  schema.tables.forEach((table, at) => {
    const row = Math.floor(at / PER_ROW);
    rows[row] = Math.max(rows[row] ?? 0, tallOf(table));
  });
  const usual = Math.max(HEAD, ...rows);
  const topOf = (row: number) =>
    Array.from({ length: row }, (_, at) => (rows[at] ?? usual) + 60).reduce(
      (sum, step) => sum + step,
      MARGIN,
    );
  const cell = (at: number) => {
    const row = Math.floor(at / PER_ROW);
    return {
      x: MARGIN + (at % PER_ROW) * CELL,
      y: topOf(row),
      wide: CELL - 40,
      tall: rows[row] ?? usual,
    };
  };
  const placed = schema.tables.flatMap((table) =>
    table.at
      ? [
          {
            x: Math.max(0, table.at.x),
            y: Math.max(0, table.at.y),
            wide: wideOf(table),
            tall: tallOf(table),
          },
        ]
      : [],
  );
  const taken = new Set<number>();
  // A cell a table placed by hand covers is skipped, so no two cards are drawn one over the other.
  const free = (at: number) => {
    let spot = at;
    while (taken.has(spot) || placed.some((one) => overlaps(one, cell(spot)))) spot += 1;
    taken.add(spot);
    return cell(spot);
  };
  return schema.tables.map((table, at) => {
    const wide = wideOf(table);
    const tall = tallOf(table);
    if (table.at)
      return { table, x: Math.max(0, table.at.x), y: Math.max(0, table.at.y), wide, tall };
    const { x, y } = free(at);
    return { table, x, y, wide, tall };
  });
};

type Rect = { x: number; y: number; wide: number; tall: number };

const overlaps = (a: Rect, b: Rect) =>
  a.x < b.x + b.wide && b.x < a.x + a.wide && a.y < b.y + b.tall && b.y < a.y + a.tall;

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
