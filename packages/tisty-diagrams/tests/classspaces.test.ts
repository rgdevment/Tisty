import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Classes, readClasses, writeClasses } from "../src/classes";
import {
  type ClassAltered,
  classKindChanged,
  classPickedRemoved,
  classRemoved,
} from "../src/classing";
import {
  classLanesOf,
  classMovedTo,
  namespaceAdded,
  namespaceMoved,
  namespaceRenamed,
  namespaceResized,
  reclassed,
} from "../src/classspaces";
import { classScene } from "../src/scenes";

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

const SPACED = unfolded(
  "classDiagram⏎  namespace Zoo {⏎    class Animal {⏎      +name⏎    }⏎    class Pez⏎  }⏎  class Duck⏎  Animal <|-- Duck⏎  %% tisty: Zoo 0 0 300 360⏎  %% tisty: Animal 30 50⏎  %% tisty: Pez 30 200⏎  %% tisty: Duck 500 50",
);
const spaced = () => readClasses(SPACED) as Classes;

describe("packages", () => {
  it("reads each class's package, and the package's lane, and gives it back as written", async () => {
    expect(spaced().classes.map((one) => [one.name, one.group])).toEqual([
      ["Animal", "Zoo"],
      ["Pez", "Zoo"],
      ["Duck", null],
    ]);
    expect(classLanesOf(spaced()).map((one) => [one.id, one.rect])).toEqual([
      ["Zoo", { x: 0, y: 0, wide: 300, tall: 360 }],
    ]);
    expect(writeClasses(spaced())).toBe(SPACED);
    expect(classScene(spaced()).groups.map((one) => one.key)).toEqual(["Zoo"]);
    expect(await valid(SPACED)).toBe(true);
  });

  it("adds a package with a class in it, as Mermaid takes no empty one", async () => {
    const made = namespaceAdded(spaced(), "Mi paquete", "Clase", {
      x: 600,
      y: 0,
      wide: 300,
      tall: 360,
    });
    const text = written(spaced(), made);

    expect(text).toContain("  namespace Mi_paquete1 {\n    class Clase1\n  }\n");
    expect(text).toContain("  %% tisty: Mi_paquete1 600 0 300 360");
    expect(text).toContain("  %% tisty: Clase1 624 48");
    expect(made.picked).toEqual({ kind: "class", name: "Clase1" });
    expect(await valid(text)).toBe(true);
  });

  it("moves a package with its classes, and sizes it leaving them where they are", () => {
    const moved = written(spaced(), namespaceMoved(spaced(), "Zoo", { x: 10, y: 20 }));
    const sized = written(
      spaced(),
      namespaceResized(spaced(), "Zoo", { x: 0, y: 0, wide: 400, tall: 200 }),
    );

    expect(moved).toContain("%% tisty: Zoo 10 20 300 360");
    expect(moved).toContain("%% tisty: Animal 40 70");
    expect(moved).toContain("%% tisty: Duck 500 50");
    expect(sized).toContain("%% tisty: Zoo 0 0 400 200");
    expect(namespaceMoved(spaced(), "Nada", { x: 1, y: 1 }).places).toBeUndefined();
  });

  it("renames a package where it opens and where it is placed", () => {
    const text = written(spaced(), namespaceRenamed(spaced(), "Zoo", "Granja"));

    expect(text).toContain("  namespace Granja {");
    expect(text).toContain("%% tisty: Granja 0 0 300 360");
    expect(namespaceRenamed(spaced(), "Zoo", "a/b").picked).toEqual({
      kind: "namespace",
      name: "Zoo",
    });
    expect(namespaceRenamed(spaced(), "Nada", "Otro").body).toEqual(spaced().body);
  });
});

// One per line: a class → the package it is moved to (`-` for none) → the diagram left.
const MOVED = `
Duck → Zoo → namespace Zoo {⏎    class Animal {⏎      +name⏎    }⏎    class Pez⏎    class Duck⏎  }⏎  Animal <|-- Duck
Animal → - → namespace Zoo {⏎    class Pez⏎  }⏎  class Duck⏎  Animal <|-- Duck⏎  class Animal {⏎    +name⏎  }
Pez → - → namespace Zoo {⏎    class Animal {⏎      +name⏎    }⏎  }⏎  class Duck⏎  Animal <|-- Duck⏎  class Pez
`;

describe("moving a class in and out of a package", () => {
  it("takes the class with its braces, and a package left empty goes", async () => {
    const plain = unfolded(
      "classDiagram⏎  namespace Zoo {⏎    class Animal {⏎      +name⏎    }⏎    class Pez⏎  }⏎  class Duck⏎  Animal <|-- Duck",
    );
    for (const case_ of lines(MOVED)) {
      const [name, into, expected] = case_.split(" → ").map((one) => one.trim());
      const diagram = readClasses(plain) as Classes;
      const text = written(diagram, classMovedTo(diagram, name, into === "-" ? null : into));
      expect(text, case_).toBe(unfolded(`classDiagram⏎  ${expected}`));
      expect(await valid(text), case_).toBe(true);
    }
    const alone = readClasses(
      "classDiagram\n  namespace Zoo {\n    class A\n  }\n  A --> B",
    ) as Classes;
    const out = written(alone, classMovedTo(alone, "A", null));
    expect(out).toBe("classDiagram\n  A --> B\n  class A");
    expect(classMovedTo(alone, "A", "Nada").body).toBe(alone.body);
  });

  it("puts a class let go in a package's lane in it, and one let go outside out", () => {
    const into = reclassed(spaced(), new Map([["Duck", { x: 100, y: 100, wide: 150, tall: 60 }]]));
    const out = reclassed(spaced(), new Map([["Pez", { x: 900, y: 900, wide: 150, tall: 60 }]]));
    const base = spaced();
    const stays = reclassed(base, new Map([["Duck", { x: 900, y: 900, wide: 150, tall: 60 }]]));

    expect(into.classes.find((one) => one.name === "Duck")?.group).toBe("Zoo");
    expect(out.classes.find((one) => one.name === "Pez")?.group).toBeNull();
    expect(stays).toBe(base);
  });

  it("keeps the package when its last class is deleted, by taking it out, and a kind said outside", async () => {
    const alone = readClasses(
      "classDiagram\n  namespace Zoo {\n    class A\n  }\n  class B",
    ) as Classes;
    const gone = written(alone, classRemoved(alone, "A"));
    const kinded = written(alone, classKindChanged(alone, "A", "interface"));

    expect(gone).toBe("classDiagram\n  class B");
    expect(kinded).toBe(
      "classDiagram\n  namespace Zoo {\n    class A\n  }\n  <<interface>> A\n  class B",
    );
    expect(written(alone, classPickedRemoved(alone, { kind: "namespace", name: "Zoo" }))).toBe(
      "classDiagram\n    class A\n  class B",
    );
    for (const text of [gone, kinded]) expect(await valid(text)).toBe(true);
  });
});
