import type { Schema, Table } from "./schema";
import { tallOf, wideOf } from "./schemalayout";
import { type Altered, type Picked, tableAdded } from "./scheming";

type Point = { readonly x: number; readonly y: number };

// A card never goes above or left of the sheet, where nothing would show it.
const onSheet = (at: Point) => ({
  x: Math.max(0, Math.round(at.x)),
  y: Math.max(0, Math.round(at.y)),
});

const unplaced = (table: Table): Table => {
  const { at: _at, ...rest } = table;
  return rest;
};

/** Tables put where they were let go; only they get a place written, as before the canvas. */
export const tablesPlaced = (
  schema: Schema,
  moves: readonly { readonly name: string; readonly x: number; readonly y: number }[],
  picked: Picked,
): Altered => {
  const to = new Map(moves.map((one) => [one.name, onSheet(one)]));
  return {
    schema: {
      ...schema,
      tables: schema.tables.map((table) => {
        const at = to.get(table.name);
        return at ? { ...table, at } : table;
      }),
    },
    picked,
  };
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
