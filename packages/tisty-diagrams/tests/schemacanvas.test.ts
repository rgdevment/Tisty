import { describe, expect, it } from "vitest";
import { columnHandle, columnOfHandle, tableHandles } from "../src/SchemaNodes";
import { readSchema, type Schema, writeSchema } from "../src/schema";
import { schemaJointsOf, schemaLaid, schemaScene } from "../src/schemalayout";
import { schemaRelaid, tableAddedAt, tablesPlaced } from "../src/schemaplaces";
import { type Picked, schemaPickedRemoved } from "../src/scheming";
import { chosenTables } from "../src/useSchemaCanvas";

const SCHEMA =
  "erDiagram\n  a ||--o{ b : tiene\n  b }|..o| c : usa\n  a {\n    int id PK\n  }\n  b {\n    int id PK\n    int a_id FK\n  }\n  c {\n    int id\n  }\n";

const read = (source = SCHEMA): Schema => {
  const schema = readSchema(source);
  if (!schema) throw new Error("not a schema");
  return schema;
};

describe("a schema on the canvas", () => {
  it("puts tables where they are let go, rounded and never off the sheet, writing only theirs", () => {
    const cases: [readonly { name: string; x: number; y: number }[], string][] = [
      [[{ name: "a", x: 10.4, y: 20.6 }], "%% tisty: a 10 21"],
      [[{ name: "b", x: -30, y: 5 }], "%% tisty: b 0 5"],
      [
        [
          { name: "a", x: 1, y: 2 },
          { name: "c", x: 300, y: 400 },
        ],
        "%% tisty: c 300 400",
      ],
    ];
    for (const [moves, line] of cases) {
      const made = tablesPlaced(read(), moves, null);
      const written = writeSchema(made.schema);
      expect(written, line).toContain(line);
      expect(written.match(/%% tisty: \w+ \d+/g)?.length, line).toBe(moves.length);
    }
  });

  it("lays tables back in the grid, keeping what is required", () => {
    const placed = read(`${SCHEMA}  %% tisty: a 400 400 required id\n  %% tisty: b 10 10\n`);

    const made = schemaRelaid(placed, { kind: "table", name: "a" });

    expect(writeSchema(made.schema)).toBe(`${SCHEMA}  %% tisty: a required id\n`);
    expect(made.picked).toEqual({ kind: "table", name: "a" });
  });

  it("adds a table centred where it is put", () => {
    const made = tableAddedAt(read(), "pagos", { x: 400, y: 300 });
    const rect = schemaLaid(made.schema).get("pagos");

    expect(made.picked).toEqual({ kind: "table", name: "pagos" });
    expect(rect && rect.x + rect.wide / 2).toBe(400);
    expect(rect && Math.abs(rect.y + rect.tall / 2 - 300)).toBeLessThanOrEqual(1);
  });

  it("takes out what is chosen, or nothing when nothing is", () => {
    const cases: [Picked, string | null][] = [
      [null, null],
      [{ kind: "table", name: "c" }, "erDiagram\n  a ||--o{ b : tiene\n"],
      [{ kind: "tables", names: ["a", "c"] }, "erDiagram\n  b {\n"],
      [{ kind: "column", table: "b", index: 1 }, "    int id PK\n  }\n  c {"],
      [{ kind: "relation", index: 0 }, "erDiagram\n  b }|..o| c : usa\n"],
    ];
    for (const [picked, starts] of cases) {
      const made = schemaPickedRemoved(read(), picked);
      if (starts === null) expect(made).toBeNull();
      else expect(writeSchema(made?.schema ?? read()), JSON.stringify(picked)).toContain(starts);
    }
  });

  it("draws each relation solid when identifying, with how many on each end", () => {
    expect(
      schemaJointsOf(read()).map((one) => [
        one.from,
        one.to,
        one.dashed,
        one.startLabel,
        one.endLabel,
      ]),
    ).toEqual([
      ["a", "b", false, "1", "0..N"],
      ["b", "c", true, "1..N", "0..1"],
    ]);
    expect(schemaScene(read()).laid.at.size).toBe(3);
  });

  it("tells a column's dot from a side's", () => {
    const cases: [string | null | undefined, number | null][] = [
      [columnHandle(3), 3],
      ["right", null],
      [null, null],
      [undefined, null],
    ];
    for (const [handle, column] of cases)
      expect(columnOfHandle(handle), String(handle)).toBe(column);
    expect(tableHandles(200, 100, 2).map((one) => one.id)).toEqual([
      "top",
      "right",
      "bottom",
      "left",
      "column-0",
      "column-1",
    ]);
  });

  it("names the tables chosen, one or several", () => {
    const cases: [Picked, readonly string[]][] = [
      [null, []],
      [{ kind: "table", name: "a" }, ["a"]],
      [{ kind: "tables", names: ["a", "b"] }, ["a", "b"]],
      [{ kind: "relation", index: 0 }, []],
    ];
    for (const [picked, names] of cases) expect(chosenTables(picked)).toEqual(names);
  });
});
