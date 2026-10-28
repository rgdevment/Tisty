import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Classes, readClasses, writeClasses } from "../src/classes";
import {
  classAdded,
  classesLinked,
  classRemoved,
  classRenamed,
  linkChanged,
  linkRemoved,
  memberAdded,
  memberChanged,
  memberRemoved,
} from "../src/classing";

// `⏎` is a line break, so a whole diagram fits on its line.
const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

// Class diagrams as people write them, one per line: each is read and given back as written.
const WRITTEN = `
classDiagram⏎  direction LR⏎  class Animal {⏎    <<abstract>>⏎    +String name⏎    +makeSound() void⏎  }⏎  class Duck⏎  Animal <|-- Duck : hereda⏎  Animal "1" *-- "many" Leg : tiene⏎  Animal : +isMammal()⏎  Duck ..|> Swimmer⏎  <<interface>> Swimmer
classDiagram⏎  Animal o-- Zoo⏎  Animal --> Food⏎  Animal ..> Water⏎  Animal -- Owner⏎  Animal .. Ghost⏎  Zoo --* Animal⏎  %% una nota⏎  style Zoo fill:#f9f⏎  note for Zoo "abre a las 9"⏎
---⏎title: Tienda⏎---⏎classDiagram⏎  class Pedido~T~ {⏎    -List~T~ items⏎  }⏎  class Cliente:::vip
`;

// Diagrams the window cannot keep without changing them, one per line.
const REFUSED = `
classDiagram⏎  class A {⏎    +int x
classDiagram⏎  class A⏎  class A
classDiagram⏎  namespace Tienda {⏎    class A⏎  }
graph TD⏎  a --> b
`;

describe("reading a class diagram", () => {
  it("reads every diagram people write, Mermaid agrees, and gives it back as written", async () => {
    for (const case_ of lines(WRITTEN)) {
      const source = unfolded(case_);
      const diagram = readClasses(source);
      expect(diagram, case_).not.toBeNull();
      expect(writeClasses(diagram as Classes), case_).toBe(source);
      expect(await valid(source), case_).toBe(true);
    }
  });

  it("refuses what it cannot keep, so the window leaves it as text", () => {
    for (const case_ of lines(REFUSED)) expect(readClasses(unfolded(case_)), case_).toBeNull();
  });

  it("reads classes, members in braces or after a colon, notes and relations", () => {
    const diagram = readClasses(unfolded(lines(WRITTEN)[0])) as Classes;

    expect(diagram.direction).toBe("LR");
    expect(
      diagram.classes.map((one) => [one.name, one.note, one.members.map((m) => m.text)]),
    ).toEqual([
      ["Animal", "abstract", ["+String name", "+makeSound() void", "+isMammal()"]],
      ["Duck", null, []],
      ["Leg", null, []],
      ["Swimmer", "interface", []],
    ]);
    expect(
      diagram.links.map((one) => [
        one.left,
        one.leftCard,
        one.arrow,
        one.rightCard,
        one.right,
        one.label,
      ]),
    ).toEqual([
      ["Animal", "", "<|--", "", "Duck", "hereda"],
      ["Animal", "1", "*--", "many", "Leg", "tiene"],
      ["Duck", "", "..|>", "", "Swimmer", ""],
    ]);
  });
});

const BASE = unfolded(
  "classDiagram⏎  class Animal {⏎    +String name⏎  }⏎  class Duck⏎  Animal <|-- Duck : hereda⏎  Animal : +isMammal()⏎  style Duck fill:#ff0",
);
const of = () => readClasses(BASE) as Classes;
const bodyText = (body: readonly string[]) =>
  writeClasses({ head: of().head, body, places: [], tail: [] });

// One per row: what is done → the text it leaves.
const DONE: [string, () => readonly string[], string][] = [
  ["a new class goes last", () => classAdded(of(), "Clase").body, `${BASE}\n  class Clase1`],
  [
    "a rename reaches every line that names it alone",
    () => classRenamed(of(), "Animal", "Ser").body,
    BASE.replace(/Animal/g, "Ser"),
  ],
  [
    "a member goes inside the braces",
    () => memberAdded(of(), "Animal", "+int age").body,
    BASE.replace("    +String name\n", "    +String name\n    +int age\n"),
  ],
  [
    "a class without braces gets them",
    () => memberAdded(of(), "Duck", "+swim()").body,
    BASE.replace("  class Duck\n", "  class Duck {\n    +swim()\n  }\n"),
  ],
  [
    "a member after a colon stays after a colon",
    () => memberChanged(of(), "Animal", 1, "+isAlive() bool").body,
    BASE.replace("Animal : +isMammal()", "Animal : +isAlive() bool"),
  ],
  [
    "removing a member takes only its line",
    () => memberRemoved(of(), "Animal", 0).body,
    BASE.replace("    +String name\n", ""),
  ],
  ["two classes are related once", () => classesLinked(of(), "Duck", "Animal").body, BASE],
  [
    "a relation's kind, ends and words are written in its line",
    () =>
      linkChanged(of(), 0, { arrow: "*--", leftCard: "1", rightCard: "0..*", label: "tiene" }).body,
    BASE.replace("Animal <|-- Duck : hereda", 'Animal "1" *-- "0..*" Duck : tiene'),
  ],
  [
    "removing a relation takes its line",
    () => linkRemoved(of(), 0).body,
    BASE.replace("  Animal <|-- Duck : hereda\n", ""),
  ],
  [
    "removing a class takes its declaration, members, relations and style",
    () => classRemoved(of(), "Duck").body,
    "classDiagram\n  class Animal {\n    +String name\n  }\n  Animal : +isMammal()",
  ],
];

describe("changing a class diagram", () => {
  it("does each change, keeps the rest as written and leaves valid Mermaid", async () => {
    for (const [said, change, expected] of DONE) {
      const written = bodyText(change());
      expect(written, said).toBe(expected);
      expect(await valid(written), said).toBe(true);
    }
  });

  it("removes only the class named, whatever letters end its name", async () => {
    const source = unfolded(
      'classDiagram⏎  class Camión⏎  class CamiónGrande⏎  note for Camión "vieja"⏎  note for CamiónGrande "nueva"',
    );
    const classes = readClasses(source) as Classes;
    const written = writeClasses({ ...classes, body: classRemoved(classes, "Camión").body });

    expect(written).toBe('classDiagram\n  class CamiónGrande\n  note for CamiónGrande "nueva"');
    expect(await valid(written)).toBe(true);
  });

  it("gives braces to a class written with its style class", async () => {
    const classes = readClasses("classDiagram\n  class Cliente:::vip") as Classes;
    const written = writeClasses({ ...classes, body: memberAdded(classes, "Cliente", "+id").body });

    expect(written).toBe("classDiagram\n  class Cliente:::vip {\n    +id\n  }");
    expect(await valid(written)).toBe(true);
  });

  it("refuses a name already taken or not one Mermaid reads", () => {
    expect(classRenamed(of(), "Animal", "Duck").body).toEqual(of().body);
    expect(classRenamed(of(), "Animal", "con-guion").body).toEqual(of().body);
    expect(classRenamed(of(), "Animal", "dos palabras").picked).toEqual({
      kind: "class",
      name: "dos_palabras",
    });
    expect(classesLinked(of(), "Duck", "Animal").picked).toEqual({ kind: "link", index: 0 });
  });
});
