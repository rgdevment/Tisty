import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Flow, readFlow, writeFlow } from "../src/flow";
import { nodeAddedFrom, nodesPlaced, pinned } from "../src/flowing";
import { outlineOf, sizeOf } from "../src/flowoutlines";
import { routeOf, sidesOf } from "../src/flowroutes";
import { type Bracketed, NAMED, nameOfShape, type Shape, shapeNamed } from "../src/flowshapes";
import { PALETTE, SHAPES, shapeName } from "../src/paletteshapes";
import { PALETTES, svgOf } from "../src/picture";
import { pictureOf } from "../src/pictures";
import { flowLaid, flowScene } from "../src/scenes";
import { sayIn } from "../src/words";

const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

const BRACKETED: readonly Bracketed[] = [
  "process",
  "rounded",
  "stadium",
  "subroutine",
  "database",
  "circle",
  "double",
  "decision",
  "hexagon",
  "lean",
  "leanLeft",
  "trapezoid",
  "trapezoidDown",
  "flag",
];

// One per line, at 120 × 60: a shape → whether it has a body → how many lines go over it → how far its words drop.
const OUTLINES = `
process → body → 0 → 0
subroutine → body → 2 → 0
database → body → 1 → 4.8
double → body → 1 → 0
decision → body → 0 → 0
person → body → 1 → 12.2
cloud → body → 0 → 0
doc → body → 0 → -3
docs → body → 2 → 0
datastore → body → 2 → 0
browser → body → 1 → 6
console → body → 2 → 6
brace → none → 1 → 0
braces → none → 2 → 0
text → none → 0 → 0
bolt → body → 0 → 0
`;

describe("drawing a shape", () => {
  it("draws each shape from its outline: a body, lines over it, and where its words sit", () => {
    for (const case_ of lines(OUTLINES)) {
      const [shape, body, over, drop] = case_.split(" → ").map((one) => one.trim());
      const outline = outlineOf(shape as Shape, 120, 60);
      expect([outline.body ? "body" : "none", outline.lines.length], case_).toEqual([
        body,
        Number(over),
      ]);
      expect(outline.drop, case_).toBeCloseTo(Number(drop), 1);
    }
  });

  it("draws every shape Mermaid knows with plain numbers, at any size", () => {
    for (const shape of [...BRACKETED, ...NAMED])
      for (const [wide, tall] of [
        [120, 60],
        [28, 28],
      ]) {
        const said = [outlineOf(shape, wide, tall).body, ...outlineOf(shape, wide, tall).lines];
        expect(said.join(" "), shape).not.toMatch(/NaN|Infinity|undefined/);
      }
  });

  it("sizes a shape for its words, keeping circles round and small dots small", () => {
    expect(sizeOf("Uno", "process")).toEqual({ wide: 90, tall: 42 });
    expect(sizeOf("Una palabra larga", "decision")).toEqual({ wide: 204, tall: 64 });
    expect(sizeOf("Fin", "circle")).toEqual({ wide: 90, tall: 90 });
    expect(sizeOf("", "sm-circ")).toEqual({ wide: 28, tall: 28 });
    expect(sizeOf("Cliente", "person")).toEqual({ wide: 90, tall: 84 });
    expect(sizeOf("Proveedor", "cloud")).toEqual({ wide: 134, tall: 64 });
  });

  it("names every shape of the palette in both languages", () => {
    for (const locale of ["en", "es"])
      for (const shape of SHAPES)
        expect(shapeName(shape, sayIn(locale)), shape).not.toMatch(/^flow/);
    expect(shapeName("bolt", sayIn("es"))).toBe("Enlace de comunicación");
    expect(PALETTE.map((one) => one.shapes.length)).toEqual([13, 12, 5, 9, 10, 4]);
  });

  it("offers every shape Mermaid draws, once each", () => {
    expect(new Set(SHAPES).size).toBe(SHAPES.length);
    expect(SHAPES).toHaveLength(53);
    for (const shape of SHAPES) expect(shapeNamed(nameOfShape(shape)), shape).toBe(shape);
  });
});

// One per line: the shape an arrow leaves (x y wide tall) | the one it enters → the two sides.
const SIDES = `
0 0 100 40 | 200 0 100 40 → right left
200 0 100 40 | 0 0 100 40 → left right
0 0 100 40 | 0 200 100 40 → bottom top
0 200 100 40 | 0 0 100 40 → top bottom
0 0 100 40 | 90 100 100 40 → bottom top
0 0 100 40 | 150 100 100 40 → right left
0 0 100 40 | 0 0 100 40 → right top
`;

const rect = (said: string) => {
  const [x, y, wide, tall] = said.trim().split(" ").map(Number);
  return { x, y, wide, tall };
};

describe("routing an arrow", () => {
  it("leaves and enters by the sides that face each other across the wider gap", () => {
    for (const case_ of lines(SIDES)) {
      const [shapes, sides] = case_.split(" → ");
      const [from, to] = shapes.split("|").map(rect);
      expect(sidesOf(from, to).join(" "), case_).toBe(sides.trim());
    }
  });

  it("runs at right angles from the middle of one side to the middle of the other", () => {
    const route = routeOf(rect("0 0 100 40"), rect("200 0 100 40"));

    expect(route.path.startsWith("M100 20")).toBe(true);
    expect(route.path.endsWith("200 20")).toBe(true);
    expect([route.labelX, route.labelY]).toEqual([150, 20]);
  });
});

const PLACED = unfolded(
  "flowchart LR⏎  A[Uno] --> B[Dos]⏎  B --> C[Tres]⏎  subgraph g1 [Caja]⏎    C⏎  end⏎  %% tisty: A -100 -50⏎  %% tisty: C 300 200",
);

describe("laying out a flowchart with places", () => {
  it("keeps what was placed where it was put and lays out the rest", () => {
    const laid = flowLaid(readFlow(PLACED) as Flow);

    expect(laid.get("A")).toMatchObject({ x: -100, y: -50 });
    expect(laid.get("C")).toMatchObject({ x: 300, y: 200 });
    expect(laid.get("B")?.x).toBeCloseTo(208.4, 1);
    expect(laid.get("B")?.y).toBe(40);
  });

  it("starts the picture at its margin wherever the places are", () => {
    const scene = flowScene(readFlow(PLACED) as Flow);

    expect(scene.laid.at.get("A")).toMatchObject({ x: 40, y: 40 });
    expect(scene.laid.at.get("C")).toMatchObject({ x: 440, y: 290 });
    expect(scene.laid.size).toEqual({ wide: 570, tall: 372 });
    expect(scene.groups[0].key).toBe("g1");
  });

  it("draws the picture of a flowchart with Mermaid's newer shapes and its places", () => {
    const source = `${PLACED}\n  D@{ shape: cloud, label: "Nube" }`;
    const picture = pictureOf(source);
    const svg = picture ? svgOf(picture, PALETTES.light, null, "sans-serif") : "";

    expect(svg).toContain(">Nube<");
    expect(svg).toContain("translate(40,40)");
    expect(svg).toMatch(/<path d="M[\d.]+ [\d.]+L/);
    expect(pictureOf('flowchart LR\n  I@{ icon: "fa:user", label: "Yo" }')).toBeNull();
  });
});

describe("placing from the canvas", () => {
  const flow = () => readFlow(PLACED) as Flow;

  it("writes where every node stands once one is moved, so nothing else jumps", () => {
    const all = pinned(flow(), flowLaid(flow()));

    expect(all.places.map((one) => one.line)).toEqual([
      "  %% tisty: A -100 -50",
      "  %% tisty: C 300 200",
      "  %% tisty: B 208 40",
    ]);
    expect(pinned(all, flowLaid(all))).toBe(all);
  });

  it("moves several nodes at once and keeps what was chosen", () => {
    const made = nodesPlaced(
      flow(),
      [
        { id: "A", x: 1, y: 2 },
        { id: "B", x: 3, y: 4 },
      ],
      { kind: "node", id: "B" },
    );

    expect(made.places?.map((one) => one.line)).toEqual([
      "  %% tisty: A 1 2",
      "  %% tisty: C 300 200",
      "  %% tisty: B 3 4",
    ]);
    expect(made.picked).toEqual({ kind: "node", id: "B" });
  });

  it("adds a node joined from another where an arrow was let go", async () => {
    const made = nodeAddedFrom(flow(), "C", "Paso nuevo", { x: 500, y: 210 });
    const written = writeFlow({ ...flow(), body: made.body, places: made.places ?? [] });

    expect(written).toBe(
      PLACED.replace("  end\n", "  end\n  C --> n1[Paso nuevo]\n").concat(
        "\n  %% tisty: n1 500 210",
      ),
    );
    expect(made.picked).toEqual({ kind: "node", id: "n1" });
    expect(await valid(written)).toBe(true);
  });
});
