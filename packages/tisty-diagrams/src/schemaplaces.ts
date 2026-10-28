import type { Schema, Table } from "./schema";
import { schemaLaid, tallOf, wideOf } from "./schemalayout";
import { type Altered, type Picked, tableAdded } from "./scheming";

type Point = { readonly x: number; readonly y: number };

const rounded = (at: Point) => ({ x: Math.round(at.x), y: Math.round(at.y) });

const unplaced = (table: Table): Table => {
  const { at: _at, ...rest } = table;
  return rest;
};

/** Tables put where they were let go; another gets a place written only when the grid would shove it. */
export const tablesPlaced = (
  schema: Schema,
  moves: readonly { readonly name: string; readonly x: number; readonly y: number }[],
  picked: Picked,
): Altered => {
  const to = new Map(moves.map((one) => [one.name, rounded(one)]));
  const placed = (from: Schema, at: ReadonlyMap<string, Point>): Schema => ({
    ...from,
    tables: from.tables.map((table) => {
      const spot = at.get(table.name);
      return spot ? { ...table, at: spot } : table;
    }),
  });
  const moved = placed(schema, to);
  const before = schemaLaid(schema);
  const after = schemaLaid(moved);
  const shoved = new Map(
    [...before].flatMap(([name, rect]) => {
      const now = after.get(name);
      return !to.has(name) && now && (now.x !== rect.x || now.y !== rect.y)
        ? [[name, rounded(rect)] as const]
        : [];
    }),
  );
  return { schema: placed(moved, shoved), picked };
};

/** Every table back in its cell of the grid, in the order they are written. */
export const schemaRelaid = (schema: Schema, picked: Picked): Altered => ({
  schema: { ...schema, tables: schema.tables.map(unplaced) },
  picked,
});

/** A new table with its id, centred where it was put. */
export const tableAddedAt = (schema: Schema, wanted: string, middle: Point): Altered => {
  const made = tableAdded(schema, wanted);
  const table = made.schema.tables[made.schema.tables.length - 1];
  return tablesPlaced(
    made.schema,
    [{ name: table.name, x: middle.x - wideOf(table) / 2, y: middle.y - tallOf(table) / 2 }],
    made.picked,
  );
};
