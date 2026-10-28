export type Key = "PK" | "FK" | "UK";
export type End = "zero-one" | "one" | "zero-many" | "one-many";

export const KEYS: readonly Key[] = ["PK", "FK", "UK"];
export const ENDS: readonly End[] = ["one", "zero-one", "zero-many", "one-many"];

export interface Column {
  readonly type: string;
  readonly name: string;
  readonly keys: readonly Key[];
  readonly comment?: string;
  // Mermaid has no word for it; it lives in the table's `%% tisty:` line and only SQL and DBML use it.
  readonly required: boolean;
  readonly line?: string;
}

export interface Table {
  readonly name: string;
  readonly columns: readonly Column[];
  readonly at?: { readonly x: number; readonly y: number };
  // The block as read, written back as is while the table is untouched.
  readonly block?: readonly string[];
  readonly opener?: string;
  // Comments and lines inside the block that are not columns, kept when the block is written anew.
  readonly extra: readonly string[];
}

export interface Relation {
  readonly from: string;
  readonly to: string;
  readonly fromEnd: End;
  readonly toEnd: End;
  readonly identifying: boolean;
  readonly label: string;
  readonly line?: string;
}

export type Piece =
  | { readonly kind: "raw"; readonly line: string }
  | { readonly kind: "table"; readonly name: string }
  | { readonly kind: "relation"; readonly relation: Relation };

export interface Schema {
  readonly head: readonly string[];
  readonly pieces: readonly Piece[];
  readonly tables: readonly Table[];
  readonly tail: readonly string[];
}

const LEFT: Record<string, End> = {
  "|o": "zero-one",
  "||": "one",
  "}o": "zero-many",
  "}|": "one-many",
};
const RIGHT: Record<string, End> = {
  "o|": "zero-one",
  "||": "one",
  "o{": "zero-many",
  "|{": "one-many",
};
const leftOf = (end: End) => Object.keys(LEFT).find((one) => LEFT[one] === end) as string;
const rightOf = (end: End) => Object.keys(RIGHT).find((one) => RIGHT[one] === end) as string;

const NAME = String.raw`[\p{L}\p{N}_*-]+`;
const RELATION = new RegExp(
  String.raw`^\s*(${NAME})\s*(\|o|\|\||\}o|\}\|)(--|\.\.)(o\||\|\||o\{|\|\{)\s*(${NAME})\s*:\s*(.*?)\s*$`,
  "u",
);
const OPENER = new RegExp(String.raw`^\s*(${NAME})\s*\{\s*$`, "u");
// Mermaid reads keys in any case; the window holds them as `PK`, `FK`, `UK`.
const COLUMN =
  /^\s*(\S+)\s+(\S+?)(?:\s+((?:PK|FK|UK)(?:\s*,\s*(?:PK|FK|UK))*))?(?:\s+"([^"]*)")?\s*$/i;
const TISTY = new RegExp(
  String.raw`^\s*%% tisty: (${NAME})(?: (-?\d+) (-?\d+))?(?: required((?: [^\s]+)+))?\s*$`,
  "u",
);

const labelOf = (said: string) =>
  said.startsWith('"') && said.endsWith('"') ? said.slice(1, -1) : said;

const relationOf = (line: string): Relation | null => {
  const found = RELATION.exec(line);
  if (!found) return null;
  const [, from, left, dash, right, to, label] = found;
  return {
    from,
    to,
    fromEnd: LEFT[left],
    toEnd: RIGHT[right],
    identifying: dash === "--",
    label: labelOf(label),
    line,
  };
};

const columnOf = (line: string): Column | null => {
  const found = COLUMN.exec(line);
  if (!found || found[1].startsWith("%%")) return null;
  const keys = (found[3] ?? "")
    .split(",")
    .map((one) => one.trim().toUpperCase())
    .filter(Boolean) as Key[];
  const comment = found[4];
  return {
    type: found[1],
    name: found[2],
    keys,
    required: false,
    line,
    ...(comment !== undefined && { comment }),
  };
};

type Building = Omit<Table, "columns" | "extra" | "block" | "opener"> & {
  opener?: string;
  columns: Column[];
  extra: string[];
  block?: string[];
};

// A table placed by hand says where; one only some of whose columns are required says just that.
type Place = { at?: { x: number; y: number }; required: string[] };

const placeOf = (found: RegExpExecArray): Place => ({
  ...(found[2] !== undefined && { at: { x: Number(found[2]), y: Number(found[3]) } }),
  required: (found[4] ?? "").trim().split(" ").filter(Boolean),
});

/** The schema an `erDiagram` draws, or `null` when the window could not keep it as written. */
export const readSchema = (source: string): Schema | null => {
  const lines = source.replace(/\r/g, "").split("\n");
  const opening = lines.findIndex((line) => /^\s*erDiagram\b/.test(line));
  if (opening < 0) return null;
  const places = new Map<string, Place>();
  const tables = new Map<string, Building>();
  const pieces: Piece[] = [];
  const table = (name: string) => {
    if (!tables.has(name)) tables.set(name, { name, columns: [], extra: [] });
    return tables.get(name) as Building;
  };
  let open: Building | null = null;
  for (const line of lines.slice(opening + 1)) {
    if (open) {
      (open.block as string[]).push(line);
      if (line.trim() === "}") open = null;
      else {
        const column = columnOf(line);
        if (column) open.columns.push(column);
        else open.extra.push(line);
      }
      continue;
    }
    // Only a place line outside every block is one; inside a block it is a comment kept as written.
    const place = TISTY.exec(line);
    if (place) {
      places.set(place[1], placeOf(place));
      continue;
    }
    const opener = OPENER.exec(line);
    const relation = relationOf(line);
    if (opener) {
      const one = table(opener[1]);
      // Mermaid joins two blocks of one table; the window would not know which one to write.
      if (one.block) return null;
      one.block = [line];
      one.opener = line;
      open = one;
      pieces.push({ kind: "table", name: one.name });
    } else if (relation) {
      table(relation.from);
      table(relation.to);
      pieces.push({ kind: "relation", relation });
    } else pieces.push({ kind: "raw", line });
  }
  if (open) return null;
  let end = pieces.length;
  while (end > 0) {
    const last = pieces[end - 1];
    if (last.kind !== "raw" || last.line.trim()) break;
    end -= 1;
  }
  const tail = pieces.splice(end).map((one) => (one as { line: string }).line);
  return {
    head: lines.slice(0, opening + 1),
    pieces,
    tables: [...tables.values()].map((one) => placed(one, places.get(one.name))),
    tail,
  };
};

const placed = (table: Building, place?: Place): Table => {
  if (!place) return table;
  return {
    ...table,
    ...(place.at && { at: place.at }),
    columns: table.columns.map((one) =>
      place.required.includes(one.name) ? { ...one, required: true } : one,
    ),
  };
};

const quoted = (label: string) =>
  /^[\p{L}\p{N}_-]+$/u.test(label) ? label : `"${label.replace(/"/g, "'")}"`;

export const relationLine = (one: Relation): string =>
  `  ${one.from} ${leftOf(one.fromEnd)}${one.identifying ? "--" : ".."}${rightOf(one.toEnd)} ${one.to} : ${quoted(one.label)}`;

const bare = (said: string, fallback: string) => said.trim().replace(/\s+/g, "_") || fallback;

export const columnLine = (one: Column, indent: string): string => {
  const keys = one.keys.length ? ` ${one.keys.join(", ")}` : "";
  const comment = one.comment ? ` "${one.comment.replace(/"/g, "'")}"` : "";
  return `${indent}${bare(one.type, "string")} ${bare(one.name, "column")}${keys}${comment}`;
};

const blockOf = (table: Table): string[] => {
  if (table.block) return [...table.block];
  const opener = table.opener ?? `  ${table.name} {`;
  const indent = `${opener.slice(0, opener.length - opener.trimStart().length)}  `;
  const written = table.columns.find((one) => one.line)?.line;
  const inner = written ? written.slice(0, written.length - written.trimStart().length) : indent;
  return [
    opener.replace(/^(\s*)\S+/, `$1${table.name}`),
    ...table.columns.map((one) => one.line ?? columnLine(one, inner)),
    ...table.extra,
    `${opener.slice(0, opener.length - opener.trimStart().length)}}`,
  ];
};

const placeLine = (table: Table): string | null => {
  const required = table.columns.filter((one) => one.required).map((one) => one.name);
  if (!table.at && !required.length) return null;
  const at = table.at ? ` ${Math.round(table.at.x)} ${Math.round(table.at.y)}` : "";
  const said = `%% tisty: ${table.name}${at}`;
  return required.length ? `${said} required ${required.join(" ")}` : said;
};

/** The text of a schema: what was read and not touched comes back as it was, places last. */
export const writeSchema = (schema: Schema): string => {
  const tables = new Map(schema.tables.map((one) => [one.name, one]));
  const out = [...schema.head];
  for (const piece of schema.pieces) {
    if (piece.kind === "raw") out.push(piece.line);
    else if (piece.kind === "relation")
      out.push(piece.relation.line ?? relationLine(piece.relation));
    else {
      const table = tables.get(piece.name);
      if (table) out.push(...blockOf(table));
    }
  }
  for (const table of schema.tables) {
    const line = placeLine(table);
    if (line) out.push(`  ${line}`);
  }
  return [...out, ...schema.tail].join("\n");
};
