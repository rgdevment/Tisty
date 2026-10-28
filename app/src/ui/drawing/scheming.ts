import type { Column, Piece, Relation, Schema, Table } from "./schema";

/** What is chosen on the board: a table, one of its columns, or a relation by its place among them. */
export type Picked =
  | { readonly kind: "table"; readonly name: string }
  | { readonly kind: "column"; readonly table: string; readonly index: number }
  | { readonly kind: "relation"; readonly index: number }
  | null;

export interface Altered {
  readonly schema: Schema;
  readonly picked: Picked;
}

export const relationsOf = (schema: Schema): Relation[] =>
  schema.pieces.flatMap((one) => (one.kind === "relation" ? [one.relation] : []));

export const tableOf = (schema: Schema, name: string): Table | undefined =>
  schema.tables.find((one) => one.name === name);

// A changed table is written anew; its block as read would say the old thing.
const touched = (table: Table, change: Partial<Table>): Table => {
  const { block: _block, ...rest } = table;
  return { ...rest, ...change };
};

const withTable = (schema: Schema, name: string, change: (table: Table) => Table): Schema => ({
  ...schema,
  tables: schema.tables.map((one) => (one.name === name ? change(one) : one)),
});

const unique = (wanted: string, taken: readonly string[]) => {
  let name = wanted;
  for (let n = 2; taken.includes(name); n += 1) name = `${wanted}_${n}`;
  return name;
};

// A new table takes the next free place of the grid, so nothing already drawn moves.
export const tableAdded = (schema: Schema, wanted: string): Altered => {
  const name = unique(
    wanted,
    schema.tables.map((one) => one.name),
  );
  const id: Column = { type: "int", name: "id", keys: ["PK"], required: true };
  return {
    schema: {
      ...schema,
      pieces: [...schema.pieces, { kind: "table", name }],
      tables: [...schema.tables, { name, columns: [id], extra: [] }],
    },
    picked: { kind: "table", name },
  };
};

// A hyphen is plain outside a class; escaping it would break a `u` pattern.
const escaped = (said: string) => said.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

// A name stands alone when nothing that could be part of a name touches it.
const nameIn = (name: string) =>
  new RegExp(`(?<![\\p{L}\\p{N}_*-])${escaped(name)}(?![\\p{L}\\p{N}_*-])`, "gu");

const renamedIn = (piece: Piece, from: string, to: string): Piece => {
  if (piece.kind === "table") return piece.name === from ? { ...piece, name: to } : piece;
  // Lines the window keeps as written (`style`, `class`, relations in words) name tables too; comments do not count.
  if (piece.kind === "raw")
    return piece.line.trim().startsWith("%%")
      ? piece
      : { ...piece, line: piece.line.replace(nameIn(from), to) };
  const { relation } = piece;
  if (relation.from !== from && relation.to !== from) return piece;
  const { line: _line, ...rest } = relation;
  return {
    kind: "relation",
    relation: {
      ...rest,
      from: rest.from === from ? to : rest.from,
      to: rest.to === from ? to : rest.to,
    },
  };
};

export const tableRenamed = (schema: Schema, from: string, wanted: string): Altered => {
  const to = wanted.trim().replace(/\s+/g, "_");
  const taken = schema.tables.map((one) => one.name).filter((one) => one !== from);
  if (!to || to === from || taken.includes(to))
    return { schema, picked: { kind: "table", name: from } };
  return {
    schema: {
      ...withTable(schema, from, (table) => touched(table, { name: to })),
      pieces: schema.pieces.map((one) => renamedIn(one, from, to)),
    },
    picked: { kind: "table", name: to },
  };
};

export const tableRemoved = (schema: Schema, name: string): Altered => ({
  schema: {
    ...schema,
    tables: schema.tables.filter((one) => one.name !== name),
    pieces: schema.pieces.filter((one) => {
      if (one.kind === "table") return one.name !== name;
      if (one.kind === "relation") return one.relation.from !== name && one.relation.to !== name;
      return true;
    }),
  },
  picked: null,
});

// Where a table sits is a `%% tisty:` line of its own, so moving it never rewrites its block.
export const tableMoved = (
  schema: Schema,
  name: string,
  at: { x: number; y: number },
): Altered => ({
  schema: withTable(schema, name, (table) => ({ ...table, at })),
  picked: { kind: "table", name },
});

// A table only named by relations has no block yet; giving it a column gives it one.
const blocked = (schema: Schema, name: string): Schema =>
  schema.pieces.some((one) => one.kind === "table" && one.name === name)
    ? schema
    : { ...schema, pieces: [...schema.pieces, { kind: "table", name }] };

export const columnAdded = (schema: Schema, table: string): Altered => {
  const columns = tableOf(schema, table)?.columns ?? [];
  const name = unique(
    "column",
    columns.map((one) => one.name),
  );
  const column: Column = { type: "string", name, keys: [], required: false };
  return {
    schema: withTable(blocked(schema, table), table, (one) =>
      touched(one, { columns: [...one.columns, column] }),
    ),
    picked: { kind: "column", table, index: columns.length },
  };
};

// A relation drawn from a column is named after it; it follows the column's new name.
const relabeled = (schema: Schema, table: string, from: string, to: string): Schema => ({
  ...schema,
  pieces: schema.pieces.map((one) => {
    if (one.kind !== "relation" || one.relation.label !== from) return one;
    if (one.relation.from !== table && one.relation.to !== table) return one;
    const { line: _line, ...rest } = one.relation;
    return { kind: "relation", relation: { ...rest, label: to } };
  }),
});

export const columnChanged = (
  schema: Schema,
  table: string,
  index: number,
  change: Partial<Omit<Column, "line">>,
): Altered => {
  const name = change.name?.trim().replace(/\s+/g, "_");
  const said = name === undefined ? change : { ...change, name: name || "column" };
  const only = Object.keys(said).length === 1 && "required" in said;
  const was = tableOf(schema, table)?.columns[index]?.name;
  const named =
    said.name !== undefined && was !== undefined && said.name !== was
      ? relabeled(schema, table, was, said.name)
      : schema;
  return {
    schema: withTable(named, table, (one) => {
      const columns = one.columns.map((column, at) => {
        if (at !== index) return column;
        const { line: _line, ...rest } = column;
        // Being required is the `%% tisty:` line's word, so the column's own line stays as written.
        return only ? { ...column, ...said } : { ...rest, ...said };
      });
      return only ? { ...one, columns } : touched(one, { columns });
    }),
    picked: { kind: "column", table, index },
  };
};

export const columnRemoved = (schema: Schema, table: string, index: number): Altered => ({
  schema: withTable(schema, table, (one) =>
    touched(one, { columns: one.columns.filter((_, at) => at !== index) }),
  ),
  picked: { kind: "table", name: table },
});

/** A column dragged onto another table: that table holds one, this one many, and the column becomes a foreign key. */
export const related = (
  schema: Schema,
  from: { table: string; index: number },
  to: string,
): Altered | null => {
  const column = tableOf(schema, from.table)?.columns[from.index];
  if (!column || !tableOf(schema, to) || to === from.table) return null;
  const relation: Relation = {
    from: to,
    to: from.table,
    fromEnd: "one",
    toEnd: "zero-many",
    identifying: true,
    label: column.name,
  };
  // Dragging the same column onto the same table again finds the relation it already made.
  const already = relationsOf(schema).findIndex(
    (one) => one.from === to && one.to === from.table && one.label === column.name,
  );
  if (already >= 0) return { schema, picked: { kind: "relation", index: already } };
  const keyed = column.keys.includes("FK")
    ? { schema, picked: null }
    : columnChanged(schema, from.table, from.index, { keys: [...column.keys, "FK"] });
  const pieces = [...keyed.schema.pieces, { kind: "relation" as const, relation }];
  return {
    schema: { ...keyed.schema, pieces },
    picked: { kind: "relation", index: relationsOf({ ...keyed.schema, pieces }).length - 1 },
  };
};

const nthRelation = (schema: Schema, index: number) =>
  schema.pieces
    .map((one, place) => (one.kind === "relation" ? place : -1))
    .filter((place) => place >= 0)[index] ?? -1;

export const relationChanged = (
  schema: Schema,
  index: number,
  change: Partial<Omit<Relation, "line" | "from" | "to">>,
): Altered => {
  const at = nthRelation(schema, index);
  return {
    schema: {
      ...schema,
      pieces: schema.pieces.map((one, place) => {
        if (place !== at || one.kind !== "relation") return one;
        const { line: _line, ...rest } = one.relation;
        return { kind: "relation", relation: { ...rest, ...change } };
      }),
    },
    picked: { kind: "relation", index },
  };
};

export const relationRemoved = (schema: Schema, index: number): Altered => {
  const at = nthRelation(schema, index);
  return {
    schema: { ...schema, pieces: schema.pieces.filter((_, place) => place !== at) },
    picked: null,
  };
};
