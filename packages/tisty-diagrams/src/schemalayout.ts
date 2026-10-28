import type { Rect } from "./graphlayout";
import type { Joint, Scene } from "./scenes";
import type { Schema, Table } from "./schema";
import { relationsOf } from "./scheming";

export interface Box {
  readonly table: Table;
  readonly x: number;
  readonly y: number;
  readonly wide: number;
  readonly tall: number;
}

export const HEAD = 34;
export const ROW = 26;
const MARGIN = 40;
const CELL = 280;
const PER_ROW = 3;

// Measuring text needs a laid-out page; a width per character is close enough for a table card.
export const wideOf = (table: Table) =>
  Math.max(
    180,
    table.name.length * 8 + 40,
    ...table.columns.map(
      (one) => (one.name.length + one.type.length + one.keys.join(",").length) * 7 + 70,
    ),
  );

export const tallOf = (table: Table) => HEAD + Math.max(table.columns.length, 1) * ROW + 6;

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

const overlaps = (a: Rect, b: Rect) =>
  a.x < b.x + b.wide && b.x < a.x + a.wide && a.y < b.y + b.tall && b.y < a.y + a.tall;

/** Each table's place and size, by its name. */
export const schemaLaid = (schema: Schema): ReadonlyMap<string, Rect> =>
  new Map(
    boxesOf(schema).map((one) => [
      one.table.name,
      { x: one.x, y: one.y, wide: one.wide, tall: one.tall },
    ]),
  );

const END_MARKS = { one: "1", "zero-one": "0..1", "zero-many": "0..N", "one-many": "1..N" };

/** Each relation as drawn: solid when identifying, with how many on each end beside its table. */
export const schemaJointsOf = (schema: Schema): Joint[] =>
  relationsOf(schema).map((one, index) => ({
    key: String(index),
    from: one.from,
    to: one.to,
    label: one.label,
    dashed: !one.identifying,
    thick: false,
    start: null,
    end: null,
    startLabel: END_MARKS[one.fromEnd],
    endLabel: END_MARKS[one.toEnd],
  }));

/** What the window and the picture draw of a schema, with room around it. */
export const schemaScene = (schema: Schema): Scene => {
  const at = schemaLaid(schema);
  const rects = [...at.values()];
  const wide = Math.max(MARGIN * 2, ...rects.map((one) => one.x + one.wide + MARGIN));
  const tall = Math.max(MARGIN * 2, ...rects.map((one) => one.y + one.tall + MARGIN));
  return { laid: { at, size: { wide, tall } }, joints: schemaJointsOf(schema), groups: [] };
};
