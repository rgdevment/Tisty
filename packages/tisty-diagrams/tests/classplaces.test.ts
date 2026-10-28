import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Classes, readClasses, writeClasses } from "../src/classes";
import {
  type ClassAltered,
  classAdded,
  classesPinned,
  classKindChanged,
  classPickedRemoved,
  classRenamed,
  kindKept,
  sameKind,
  visibilityOf,
  withVisibility,
} from "../src/classing";
import { PALETTES, svgOf } from "../src/picture";
import { pictureOf } from "../src/pictures";
import { classJointsOf, classLaid, classScene } from "../src/scenes";

const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());
const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));
const written = (diagram: Classes, made: ClassAltered | null) =>
  writeClasses({
    ...diagram,
    body: made?.body ?? diagram.body,
    places: made?.places ?? diagram.places,
  });

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

const PLACED = unfolded(
  'classDiagram⏎  class Animal {⏎    +String name⏎  }⏎  class Duck⏎  Animal "1" <|-- "many" Duck : hereda⏎  %% tisty: Animal 10 20⏎  %% tisty: Duck -40 300',
);
const placed = () => readClasses(PLACED) as Classes;

describe("classes where they were put", () => {
  it("reads each class's place and gives the diagram back as written", async () => {
    expect(placed().places.map((one) => [one.id, one.x, one.y])).toEqual([
      ["Animal", 10, 20],
      ["Duck", -40, 300],
    ]);
    expect(writeClasses(placed())).toBe(PLACED);
    expect(classLaid(placed()).get("Duck")).toMatchObject({ x: -40, y: 300 });
    expect(await valid(PLACED)).toBe(true);
  });

  it("writes where every class stands once one is moved, and a new one where it is put", () => {
    const flow = readClasses("classDiagram\n  class A\n  class B\n  A --> B") as Classes;
    const all = classesPinned(flow, classLaid(flow));
    const made = classAdded(all, "Clase", null, { x: 5, y: 6 });

    expect(all.places.map((one) => one.id)).toEqual(["A", "B"]);
    expect(classesPinned(all, classLaid(all))).toBe(all);
    expect(written(all, made)).toContain("  class Clase1\n  %% tisty: A");
    expect(written(all, made).endsWith("  %% tisty: Clase1 5 6")).toBe(true);
  });

  it("keeps a class's place when it is renamed, and drops it when the class goes", () => {
    const renamed = written(placed(), classRenamed(placed(), "Animal", "Ser"));
    const gone = written(
      placed(),
      classPickedRemoved(placed(), { kind: "classes", names: ["Duck"] }),
    );

    expect(renamed).toContain("  %% tisty: Ser 10 20");
    expect(gone).not.toMatch(/Duck/);
    expect(classPickedRemoved(placed(), null)).toBeNull();
  });

  it("draws the picture from the places, with each relation's head and cardinalities", () => {
    const scene = classScene(placed());
    const picture = pictureOf(PLACED);
    const svg = picture ? svgOf(picture, PALETTES.light, null, "sans-serif") : "";

    expect(scene.laid.at.get("Duck")).toMatchObject({ x: 40, y: 320 });
    expect(classJointsOf(placed())[0]).toMatchObject({
      from: "Duck",
      to: "Animal",
      end: "triangle",
      startLabel: "many",
      endLabel: "1",
    });
    expect(svg).toContain('marker-end="url(#triangle)"');
    expect(svg).toContain(">many<");
  });
});

// One per line: how the class is written → the kind it is made → how it is written then.
const SORTED = `
class A → interface → class A⏎  <<interface>> A
class A {⏎    <<abstract>>⏎  } → enumeration → class A {⏎    <<enumeration>>⏎  }
class A⏎  <<interface>> A → - → class A
class A {⏎    +id⏎  } → abstract → class A {⏎    +id⏎  }⏎  <<abstract>> A
class A⏎  <<interface>> A → interface → class A⏎  <<interface>> A
`;

describe("the kind of a class", () => {
  it("writes `<<…>>` where the class says it, or after it, and takes it off", async () => {
    for (const case_ of lines(SORTED)) {
      const [source, sort, expected] = case_.split(" → ").map((one) => one.trim());
      const diagram = readClasses(unfolded(`classDiagram⏎  ${source}`)) as Classes;
      const text = written(diagram, classKindChanged(diagram, "A", sort === "-" ? null : sort));
      expect(text, case_).toBe(unfolded(`classDiagram⏎  ${expected}`));
      expect(await valid(text), case_).toBe(true);
    }
  });
});

// One per line: the relation as written → the kind chosen → the arrow written, its head at the same class.
const KINDS = `
<|-- → *-- → *--
--|> → *-- → --*
--|> → --> → -->
<-- → <|-- → <|--
--> → <|-- → --|>
-- → o-- → o--
..|> → -- → --
`;

describe("the kind of a relation", () => {
  it("keeps the head at the class it was at when the kind changes", () => {
    for (const case_ of lines(KINDS)) {
      const [arrow, kind, written] = case_.split(" → ").map((one) => one.trim());
      expect(kindKept(arrow, kind), case_).toBe(written);
    }
    expect(sameKind("--|>", "<|--")).toBe(true);
    expect(sameKind("-->", "<|--")).toBe(false);
  });

  it("reads and sets a member's visibility", () => {
    expect([visibilityOf("+name"), visibilityOf("#id"), visibilityOf("name")]).toEqual([
      "+",
      "#",
      "",
    ]);
    expect(withVisibility("+name", "-")).toBe("-name");
    expect(withVisibility("~ name", "")).toBe("name");
  });
});
