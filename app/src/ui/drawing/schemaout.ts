import type { Column, Relation, Schema, Table } from "./schema";
import { relationsOf } from "./scheming";

const SQL_TYPES: Record<string, string> = {
  string: "TEXT",
  text: "TEXT",
  int: "INTEGER",
  integer: "INTEGER",
  bigint: "BIGINT",
  float: "REAL",
  double: "DOUBLE PRECISION",
  bool: "BOOLEAN",
  boolean: "BOOLEAN",
  date: "DATE",
  datetime: "TIMESTAMP",
  timestamp: "TIMESTAMP",
  uuid: "UUID",
  blob: "BLOB",
};

const sqlType = (type: string) => SQL_TYPES[type.toLowerCase()] ?? type.toUpperCase();

// Words SQL keeps for itself; a table or column with one of these names needs quotes.
const RESERVED = new Set(
  "all and as asc by case check column create default delete desc distinct drop else end from grant group having in index insert is join key like limit not null offset on or order primary references select set table then to union unique update user using values when where with".split(
    " ",
  ),
);

export const quotedSql = (name: string): string =>
  /^[A-Za-z_][A-Za-z0-9_]*$/.test(name) && !RESERVED.has(name.toLowerCase())
    ? name
    : `"${name.replace(/"/g, '""')}"`;

const keyOf = (table: Table | undefined) =>
  table?.columns.find((one) => one.keys.includes("PK"))?.name ?? "id";

// A foreign key points where the relation drawn from it goes: the one it names, or the only one it has.
export const pointedAt = (schema: Schema, table: Table, column: Column): string | null => {
  // A table may point at itself (a parent_id), so a relation with both ends on it counts.
  const near = relationsOf(schema).filter(
    (one: Relation) => one.from === table.name || one.to === table.name,
  );
  const named = near.filter((one) => one.label === column.name);
  const only = near.length === 1 ? near : [];
  const chosen = named[0] ?? only[0];
  if (!chosen) return null;
  return chosen.from === table.name ? chosen.to : chosen.from;
};

const tableSql = (schema: Schema, table: Table): string => {
  const tables = new Map(schema.tables.map((one) => [one.name, one]));
  const lines = table.columns.map((one) => {
    const required = one.required || one.keys.includes("PK") ? " NOT NULL" : "";
    const unique = one.keys.includes("UK") ? " UNIQUE" : "";
    return `  ${quotedSql(one.name)} ${sqlType(one.type)}${required}${unique}`;
  });
  const keys = table.columns
    .filter((one) => one.keys.includes("PK"))
    .map((one) => quotedSql(one.name));
  if (keys.length) lines.push(`  PRIMARY KEY (${keys.join(", ")})`);
  for (const column of table.columns.filter((one) => one.keys.includes("FK"))) {
    const target = pointedAt(schema, table, column);
    if (target)
      lines.push(
        `  FOREIGN KEY (${quotedSql(column.name)}) REFERENCES ${quotedSql(target)}(${quotedSql(keyOf(tables.get(target)))})`,
      );
  }
  const notes = table.columns
    .filter((one) => one.comment)
    .map((one) => `-- ${table.name}.${one.name}: ${one.comment}`);
  return [...notes, `CREATE TABLE ${quotedSql(table.name)} (`, lines.join(",\n"), ");"].join("\n");
};

/** `CREATE TABLE` for every table that has columns, in the order the schema names them. */
export const toSql = (schema: Schema): string =>
  `${schema.tables
    .filter((one) => one.columns.length)
    .map((one) => tableSql(schema, one))
    .join("\n\n")}\n`;

// DBML reads a bare name only when it is a plain word; anything else goes in double quotes.
const quotedDbml = (name: string) =>
  /^[A-Za-z_][A-Za-z0-9_]*$/.test(name) ? name : `"${name.replace(/"/g, '\\"')}"`;

const dbmlColumn = (one: Column): string => {
  const settings = [
    one.keys.includes("PK") && "pk",
    one.keys.includes("UK") && "unique",
    one.required && !one.keys.includes("PK") && "not null",
    one.comment && `note: '${one.comment.replace(/'/g, "\\'")}'`,
  ].filter(Boolean);
  return `  ${quotedDbml(one.name)} ${one.type}${settings.length ? ` [${settings.join(", ")}]` : ""}`;
};

const ARROW: Record<string, string> = {
  one: "1",
  "zero-one": "1",
  "zero-many": "n",
  "one-many": "n",
};

/** DBML: the tables with their columns, and a `Ref` for each foreign key whose table is known. */
export const toDbml = (schema: Schema): string => {
  const tables = new Map(schema.tables.map((one) => [one.name, one]));
  const blocks = schema.tables
    .filter((one) => one.columns.length)
    .map((one) =>
      [`Table ${quotedDbml(one.name)} {`, ...one.columns.map(dbmlColumn), "}"].join("\n"),
    );
  const refs = schema.tables.flatMap((table) =>
    table.columns
      .filter((one) => one.keys.includes("FK"))
      .flatMap((column) => {
        const target = pointedAt(schema, table, column);
        return target
          ? [
              `Ref: ${quotedDbml(table.name)}.${quotedDbml(column.name)} > ${quotedDbml(target)}.${quotedDbml(keyOf(tables.get(target)))}`,
            ]
          : [];
      }),
  );
  // Relations no foreign key stands for have no column to point from in DBML; they stay as notes.
  const loose = relationsOf(schema)
    .filter(
      (one) =>
        !one.label ||
        !tables.get(one.to)?.columns.some((c) => c.name === one.label && c.keys.includes("FK")),
    )
    .map(
      (one) => `// ${one.from} ${ARROW[one.fromEnd]}-${ARROW[one.toEnd]} ${one.to}: ${one.label}`,
    );
  return `${[...blocks, ...(refs.length ? [refs.join("\n")] : []), ...(loose.length ? [loose.join("\n")] : [])].join("\n\n")}\n`;
};
