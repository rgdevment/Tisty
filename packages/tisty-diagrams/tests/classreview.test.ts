import { Position } from "@xyflow/system";
import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Classes, readClasses, writeClasses } from "../src/classes";
import { type ClassAltered, classRenamed } from "../src/classing";
import {
  classesRelaid,
  classLanesAround,
  namespaceMoved,
  namespaceRenamed,
} from "../src/classspaces";
import { endWordAt } from "../src/flowroutes";
import { nodeStyled } from "../src/flowstyles";
import { classLaid } from "../src/scenes";

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));
const written = (diagram: Classes, made: ClassAltered | null) =>
  writeClasses({
    ...diagram,
    body: made?.body ?? diagram.body,
    places: made?.places ?? diagram.places,
  });
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

describe("renaming packages and classes", () => {
  it("renames a package by its name only, never inside the word namespace", async () => {
    const diagram = readClasses("classDiagram\n  namespace name {\n    class A\n  }") as Classes;
    const text = written(diagram, namespaceRenamed(diagram, "name", "Zoo"));

    expect(text).toBe("classDiagram\n  namespace Zoo {\n    class A\n  }");
    expect(await valid(text)).toBe(true);
  });

  it("renames a class without renaming a package of the same name", () => {
    const diagram = readClasses(
      "classDiagram\n  namespace Animal {\n    class Animal\n  }\n  %% tisty: Animal 0 0 300 200",
    ) as Classes;
    const text = written(diagram, classRenamed(diagram, "Animal", "Ser"));

    expect(text).toContain("  namespace Animal {\n    class Ser\n  }");
  });
});

describe("packages written by hand", () => {
  const diagram = () =>
    readClasses(
      "classDiagram\n  namespace Zoo {\n    class A\n    class B\n  }\n  class C",
    ) as Classes;

  it("draws every package as a lane, around its classes when it has no size of its own", () => {
    const lanes = classLanesAround(diagram(), classLaid(diagram()));

    expect(lanes.map((one) => one.id)).toEqual(["Zoo"]);
    expect(lanes[0].rect.wide).toBeGreaterThan(150);
  });

  it("moves such a package from where it is drawn, writing its size for the first time", () => {
    const flow = diagram();
    const from = { x: 10, y: 10, wide: 200, tall: 300 };
    const text = written(flow, namespaceMoved(flow, "Zoo", { x: 50, y: 60 }, from));

    expect(text).toContain("%% tisty: Zoo 50 60 200 300");
  });

  it("lays everything out again, writing each package around its classes", () => {
    const text = written(diagram(), classesRelaid(diagram()));
    const again = readClasses(text) as Classes;

    expect(again.places.map((one) => one.id)).toEqual(["A", "B", "C", "Zoo"]);
    expect(again.places.find((one) => one.id === "Zoo")?.wide).toBeGreaterThan(150);
  });
});

describe("colours in a class diagram", () => {
  it("writes style lines Mermaid's class diagram reads", async () => {
    const diagram = readClasses("classDiagram\n  class Animal\n  class Duck") as Classes;
    const body = nodeStyled(diagram, "Animal", {
      fill: "#dbeafe",
      color: "#1e3a8a",
      stroke: "#2563eb",
    }).body;
    const text = writeClasses({ ...diagram, body });

    expect(text).toContain("  style Animal fill:#dbeafe,color:#1e3a8a,stroke:#2563eb");
    expect(await valid(text)).toBe(true);
  });
});

// One per line: the side an end meets its shape by → where its words go and which way they run.
const ENDS = `
right → 106 -10 start
left → 94 -10 end
top → 106 -10 start
bottom → 106 10 start
`;

describe("words at the ends of a line", () => {
  it("sits beside the line and off the shape, whatever side it meets", () => {
    for (const case_ of lines(ENDS)) {
      const [side, where] = case_.split(" → ").map((one) => one.trim());
      const at = endWordAt({ x: 100, y: 0 }, side as Position);
      expect(`${at.x} ${at.y} ${at.anchor}`, case_).toBe(where);
    }
    expect(Position.Left).toBe("left");
  });
});
