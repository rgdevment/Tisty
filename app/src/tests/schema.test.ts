import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { readSchema, type Schema, writeSchema } from "../ui/drawing/schema";
import { boxesOf } from "../ui/drawing/schemalayout";
import { toDbml, toSql } from "../ui/drawing/schemaout";
import {
  type Altered,
  columnAdded,
  columnChanged,
  columnRemoved,
  related,
  relationChanged,
  relationRemoved,
  relationsOf,
  tableAdded,
  tableMoved,
  tableRemoved,
  tableRenamed,
} from "../ui/drawing/scheming";

// `⏎` is a line break, so a whole schema fits on its line.
const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

// Schemas as people write them, one per line: each must come back byte for byte.
const WRITTEN = `
erDiagram⏎  personas ||--o{ compartidos : publica⏎  compartidos ||--|{ versiones : guarda⏎  personas {⏎    uuid id PK⏎    text huella UK⏎    text alias⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id FK "quién lo publicó"⏎    date caduca⏎  }⏎  %% tisty: personas 60 60 required huella⏎  %% tisty: compartidos 410 180
---⏎title: Pagos⏎---⏎erDiagram⏎    direction LR⏎    %% una nota⏎    CLIENTE }|..|{ DIRECCION : "vive en"⏎    CLIENTE {⏎        varchar(255) nombre⏎        decimal(10,2) saldo "en pesos"⏎        %% columna vieja⏎    }⏎    style CLIENTE fill:#f9f⏎
erDiagram⏎  A ||--o| B : tiene⏎  B only one to zero or more C : usa⏎
erDiagram
erDiagram⏎  A {⏎    int id PK⏎    text nombre⏎  }⏎  %% tisty: A required nombre
`;

// Schemas the window cannot keep without changing them, one per line.
const REFUSED = `
erDiagram⏎  A {⏎    int id⏎  }⏎  A {⏎    int otra⏎  }
erDiagram⏎  A {⏎    int id
graph TD⏎  a --> b
`;

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

describe("reading and writing a schema", () => {
  it("gives back every schema as it was written", () => {
    for (const case_ of lines(WRITTEN)) {
      const source = unfolded(case_);
      const schema = readSchema(source);
      expect(schema, case_).not.toBeNull();
      expect(writeSchema(schema as Schema), case_).toBe(source);
    }
  });

  it("refuses what it cannot keep, so the window leaves it as text", () => {
    for (const case_ of lines(REFUSED)) expect(readSchema(unfolded(case_)), case_).toBeNull();
  });

  it("reads tables, columns, keys, notes, places and what is required", () => {
    const schema = readSchema(unfolded(lines(WRITTEN)[0])) as Schema;
    const [personas, compartidos] = schema.tables;

    expect(schema.tables.map((one) => one.name)).toEqual(["personas", "compartidos", "versiones"]);
    expect(personas.at).toEqual({ x: 60, y: 60 });
    expect(personas.columns.map((one) => [one.type, one.name, one.keys, one.required])).toEqual([
      ["uuid", "id", ["PK"], false],
      ["text", "huella", ["UK"], true],
      ["text", "alias", [], false],
    ]);
    expect(compartidos.columns[1]).toMatchObject({ keys: ["FK"], comment: "quién lo publicó" });
    expect(
      relationsOf(schema).map((one) => [one.from, one.fromEnd, one.toEnd, one.to, one.label]),
    ).toEqual([
      ["personas", "one", "zero-many", "compartidos", "publica"],
      ["compartidos", "one", "one-many", "versiones", "guarda"],
    ]);
  });

  it("keeps a relation written in words as text it does not draw", () => {
    const schema = readSchema(unfolded(lines(WRITTEN)[2])) as Schema;
    expect(relationsOf(schema)).toHaveLength(1);
    expect(schema.pieces.some((one) => one.kind === "raw" && one.line.includes("only one"))).toBe(
      true,
    );
  });
});

const BASE = unfolded(
  "erDiagram⏎  personas ||--o{ compartidos : publica⏎  personas {⏎    uuid id PK⏎    text alias⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }",
);

// One per row: what is done → the text it leaves.
const DONE: [string, (schema: Schema) => Altered | null, string][] = [
  [
    "a new table comes with an id and takes the next free place of the grid",
    (schema) => tableAdded(schema, "versiones"),
    "erDiagram⏎  personas ||--o{ compartidos : publica⏎  personas {⏎    uuid id PK⏎    text alias⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }⏎  versiones {⏎    int id PK⏎  }⏎  %% tisty: versiones required id",
  ],
  [
    "renaming a table renames it in its relations",
    (schema) => tableRenamed(schema, "personas", "gente"),
    "erDiagram⏎  gente ||--o{ compartidos : publica⏎  gente {⏎    uuid id PK⏎    text alias⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }",
  ],
  [
    "removing a table takes its relations",
    (schema) => tableRemoved(schema, "personas"),
    "erDiagram⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }",
  ],
  [
    "moving a table writes only its place",
    (schema) => tableMoved(schema, "compartidos", { x: 410.4, y: 180 }),
    `${BASE.replace(/⏎/g, "\n")}\n  %% tisty: compartidos 410 180`,
  ],
  [
    "a new column goes last in its table",
    (schema) => columnAdded(schema, "personas"),
    "erDiagram⏎  personas ||--o{ compartidos : publica⏎  personas {⏎    uuid id PK⏎    text alias⏎    string column⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }",
  ],
  [
    "a column's type, keys and note are written in its line",
    (schema) =>
      columnChanged(schema, "personas", 1, {
        type: "varchar(80)",
        keys: ["UK"],
        comment: "visible",
      }),
    'erDiagram⏎  personas ||--o{ compartidos : publica⏎  personas {⏎    uuid id PK⏎    varchar(80) alias UK "visible"⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }',
  ],
  [
    "being required changes only the place line",
    (schema) => columnChanged(schema, "personas", 1, { required: true }),
    `${BASE}\n  %% tisty: personas required alias`,
  ],
  [
    "removing a column leaves the rest as written",
    (schema) => columnRemoved(schema, "personas", 1),
    "erDiagram⏎  personas ||--o{ compartidos : publica⏎  personas {⏎    uuid id PK⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }",
  ],
  [
    "a column dragged onto a table becomes a foreign key and a relation",
    (schema) => related(schema, { table: "compartidos", index: 1 }, "personas"),
    "erDiagram⏎  personas ||--o{ compartidos : publica⏎  personas {⏎    uuid id PK⏎    text alias⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id FK⏎  }⏎  personas ||--o{ compartidos : persona_id",
  ],
  [
    "a relation's ends, kind and words are written in its line",
    (schema) =>
      relationChanged(schema, 0, {
        fromEnd: "zero-one",
        toEnd: "one-many",
        identifying: false,
        label: "lo publica",
      }),
    'erDiagram⏎  personas |o..|{ compartidos : "lo publica"⏎  personas {⏎    uuid id PK⏎    text alias⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }',
  ],
  [
    "removing a relation leaves the tables",
    (schema) => relationRemoved(schema, 0),
    "erDiagram⏎  personas {⏎    uuid id PK⏎    text alias⏎  }⏎  compartidos {⏎    text id PK⏎    uuid persona_id⏎  }",
  ],
];

describe("changing a schema", () => {
  it("does each change, keeps the rest as written and leaves valid Mermaid", async () => {
    for (const [said, change, after] of DONE) {
      const made = change(readSchema(BASE) as Schema) as Altered;
      const written = writeSchema(made.schema);
      expect(written, said).toBe(unfolded(after));
      expect(await valid(written), said).toBe(true);
      expect(writeSchema(readSchema(written) as Schema), said).toBe(written);
    }
  });

  it("refuses names already taken, empty ones and relations to nowhere", () => {
    const schema = readSchema(BASE) as Schema;
    expect(tableRenamed(schema, "personas", "compartidos").schema).toBe(schema);
    expect(tableRenamed(schema, "personas", "  ").schema).toBe(schema);
    expect(related(schema, { table: "compartidos", index: 1 }, "nadie")).toBeNull();
    expect(related(schema, { table: "compartidos", index: 1 }, "compartidos")).toBeNull();
    expect(tableAdded(schema, "personas").picked).toEqual({
      kind: "table",
      name: "personas_2",
    });
  });
});

const SHOP = unfolded(
  'erDiagram⏎  clientes ||--o{ pedidos : cliente_id⏎  clientes {⏎    int id PK⏎    string correo UK "para avisos"⏎  }⏎  pedidos {⏎    int id PK⏎    int cliente_id FK⏎    datetime hecho⏎  }⏎  %% tisty: pedidos 300 0 required hecho',
);

describe("leaving a schema as SQL or DBML", () => {
  it("writes CREATE TABLE with keys, references, what is required and the notes", () => {
    expect(toSql(readSchema(SHOP) as Schema)).toBe(
      [
        "-- clientes.correo: para avisos",
        "CREATE TABLE clientes (",
        "  id INTEGER NOT NULL,",
        "  correo TEXT UNIQUE,",
        "  PRIMARY KEY (id)",
        ");",
        "",
        "CREATE TABLE pedidos (",
        "  id INTEGER NOT NULL,",
        "  cliente_id INTEGER,",
        "  hecho TIMESTAMP NOT NULL,",
        "  PRIMARY KEY (id),",
        "  FOREIGN KEY (cliente_id) REFERENCES clientes(id)",
        ");",
        "",
      ].join("\n"),
    );
  });

  it("writes DBML tables and a Ref for each foreign key", () => {
    expect(toDbml(readSchema(SHOP) as Schema)).toBe(
      [
        "Table clientes {",
        "  id int [pk]",
        "  correo string [unique, note: 'para avisos']",
        "}",
        "",
        "Table pedidos {",
        "  id int [pk]",
        "  cliente_id int",
        "  hecho datetime [not null]",
        "}",
        "",
        "Ref: pedidos.cliente_id > clientes.id",
        "",
      ].join("\n"),
    );
  });
});

describe("what the review of the schema found", () => {
  it("reads keys in any case, as Mermaid does, and gives the schema back as written", () => {
    const source = "erDiagram\n  A {\n    int id pk\n    int b_id Fk\n  }";
    const schema = readSchema(source) as Schema;

    expect(schema.tables[0].columns.map((one) => one.keys)).toEqual([["PK"], ["FK"]]);
    expect(writeSchema(schema)).toBe(source);
  });

  it("keeps a place line written inside a block as a comment, never as a second place", () => {
    const source = "erDiagram\n  A {\n    int id PK\n    %% tisty: A 10 10\n  }";
    const schema = readSchema(source) as Schema;
    const written = writeSchema(columnAdded(schema, "A").schema);

    expect(schema.tables[0].at).toBeUndefined();
    expect(written.match(/%% tisty: A/g)).toHaveLength(1);
  });

  it("renames a table in the lines it keeps as written, but not in comments", () => {
    const source =
      "erDiagram\n  personas {\n    int id PK\n  }\n  style personas fill:#f9f\n  personas only one to zero or more pedidos : hace\n  %% personas vieja\n  personas_viejas ||--o{ pedidos : x";
    const written = writeSchema(
      tableRenamed(readSchema(source) as Schema, "personas", "gente").schema,
    );

    expect(written).toBe(
      "erDiagram\n  gente {\n    int id PK\n  }\n  style gente fill:#f9f\n  gente only one to zero or more pedidos : hace\n  %% personas vieja\n  personas_viejas ||--o{ pedidos : x",
    );
  });

  it("finds the relation a column already made instead of drawing it twice", () => {
    const first = related(
      readSchema(BASE) as Schema,
      { table: "compartidos", index: 1 },
      "personas",
    ) as Altered;
    const again = related(first.schema, { table: "compartidos", index: 1 }, "personas") as Altered;

    expect(again.schema).toBe(first.schema);
    expect(again.picked).toEqual(first.picked);
  });

  it("renames the relation drawn from a column along with the column", () => {
    const linked = related(
      readSchema(BASE) as Schema,
      { table: "compartidos", index: 1 },
      "personas",
    ) as Altered;
    const written = writeSchema(
      columnChanged(linked.schema, "compartidos", 1, { name: "autor_id" }).schema,
    );

    expect(written).toContain("  personas ||--o{ compartidos : autor_id");
    expect(written).toContain("  personas ||--o{ compartidos : publica");
  });

  it("quotes in SQL the names SQL keeps or cannot read, and points a table at itself", () => {
    const schema = readSchema(
      "erDiagram\n  categorias ||--o{ categorias : padre_id\n  categorias {\n    int id PK\n    int padre_id FK\n  }\n  user {\n    int order PK\n    text correo_electrónico\n  }",
    ) as Schema;
    const sql = toSql(schema);

    expect(sql).toContain("FOREIGN KEY (padre_id) REFERENCES categorias(id)");
    expect(sql).toContain('CREATE TABLE "user" (');
    expect(sql).toContain('  "order" INTEGER NOT NULL,');
    expect(sql).toContain('  "correo_electrónico" TEXT');
    expect(sql).toContain('  PRIMARY KEY ("order")');
    expect(toDbml(schema)).toContain("Ref: categorias.padre_id > categorias.id");
  });
});

describe("laying out a schema", () => {
  it("never moves the other tables when one is placed by hand or added", () => {
    const schema = readSchema(
      "erDiagram\n  a {\n    int id\n  }\n  b {\n    int id\n  }\n  c {\n    int id\n  }\n  d {\n    int id\n  }",
    ) as Schema;
    const before = boxesOf(schema).map((one) => [one.x, one.y]);
    const placed = boxesOf(tableMoved(schema, "b", { x: 900, y: 500 }).schema).map((one) => [
      one.x,
      one.y,
    ]);
    const grown = boxesOf(tableAdded(schema, "e").schema).map((one) => [one.x, one.y]);

    expect(placed).toEqual([before[0], [900, 500], before[2], before[3]]);
    expect(grown.slice(0, 4)).toEqual(before);
  });
});
