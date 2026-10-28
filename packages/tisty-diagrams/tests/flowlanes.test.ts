import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Flow, readFlow, writeFlow } from "../src/flow";
import type { FlowAltered } from "../src/flowing";
import { laneAdded, laneMoved, laneResized, regrouped } from "../src/flowlanes";
import { looksOf, nodeStyled } from "../src/flowstyles";
import {
  alignedMoves,
  nodeDuplicated,
  nodesRemoved,
  pickedRemoved,
  relaid,
} from "../src/flowtools";
import { PALETTES, svgOf } from "../src/picture";
import { pictureOf } from "../src/pictures";
import { flowLaid, flowScene, lanesOf } from "../src/scenes";

const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());
const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));
const written = (flow: Flow, made: FlowAltered | null) =>
  writeFlow({ ...flow, body: made?.body ?? flow.body, places: made?.places ?? flow.places });

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

const LANED = unfolded(
  "flowchart LR⏎  subgraph cli [Cliente]⏎    A[Pide]⏎  end⏎  subgraph caja [Caja]⏎    B[Cobra]⏎  end⏎  A --> B⏎  C[Fuera]⏎  %% tisty: cli 0 0 260 480⏎  %% tisty: caja 300 0 260 480⏎  %% tisty: A 40 60⏎  %% tisty: B 340 60⏎  %% tisty: C 700 60",
);
const laned = () => readFlow(LANED) as Flow;

describe("lanes", () => {
  it("reads a lane's size from its place and gives it back as written", async () => {
    const flow = laned();

    expect(lanesOf(flow).map((one) => [one.id, one.rect, one.across])).toEqual([
      ["cli", { x: 0, y: 0, wide: 260, tall: 480 }, false],
      ["caja", { x: 300, y: 0, wide: 260, tall: 480 }, false],
    ]);
    expect(writeFlow(flow)).toBe(LANED);
    expect(await valid(LANED)).toBe(true);
  });

  it("adds a lane where it is put, empty, as a group with its size", async () => {
    const made = laneAdded(laned(), "Envío", { x: 600, y: 0, wide: 640, tall: 160 });
    const text = written(laned(), made);

    expect(text).toContain("  subgraph g1 [Envío]\n  end\n");
    expect(text.endsWith("  %% tisty: g1 600 0 640 160")).toBe(true);
    expect(made.picked).toEqual({ kind: "group", id: "g1" });
    expect(await valid(text)).toBe(true);
  });

  it("moves a lane with the shapes in it, and sizes it leaving them where they are", () => {
    const moved = written(laned(), laneMoved(laned(), "cli", { x: 10, y: 20 }));
    const sized = written(
      laned(),
      laneResized(laned(), "caja", { x: 300, y: 0, wide: 400, tall: 200 }),
    );

    expect(moved).toContain("%% tisty: cli 10 20 260 480");
    expect(moved).toContain("%% tisty: A 50 80");
    expect(moved).toContain("%% tisty: B 340 60");
    expect(sized).toContain("%% tisty: caja 300 0 400 200");
    expect(laneMoved(laned(), "nada", { x: 1, y: 1 }).places).toBeUndefined();
  });

  it("draws a lane at its size in the picture, wherever the shapes are", () => {
    const scene = flowScene(laned());
    const picture = pictureOf(LANED);

    expect(scene.groups.map((one) => [one.key, one.rect])).toEqual([
      ["cli", { x: 40, y: 40, wide: 260, tall: 480 }],
      ["caja", { x: 340, y: 40, wide: 260, tall: 480 }],
    ]);
    expect(picture ? svgOf(picture, PALETTES.dark, null, "sans-serif") : "").toContain(">Caja<");
  });
});

// One per line: a shape → where its middle is let go → the group it ends in (`-` for none).
const REGROUPED = `
A → 400 100 → caja
A → 900 100 → -
A → 100 100 → cli
C → 100 100 → cli
C → 900 300 → -
`;

describe("joining and leaving lanes", () => {
  it("puts a shape let go in a lane in it, and takes one let go outside every lane out", () => {
    for (const case_ of lines(REGROUPED)) {
      const [id, at, group] = case_.split(" → ").map((one) => one.trim());
      const [x, y] = at.split(" ").map(Number);
      const flow = regrouped(
        laned(),
        new Map([[id, { x: x - 45, y: y - 21, wide: 90, tall: 42 }]]),
      );
      expect(flow.nodes.find((one) => one.id === id)?.group ?? "-", case_).toBe(group);
    }
  });

  it("puts a shape back where it was when its lane names it in an arrow and it cannot leave", () => {
    const flow = readFlow(
      unfolded(
        "flowchart LR⏎  subgraph cli [Cliente]⏎    A --> B⏎  end⏎  %% tisty: cli 0 0 260 480⏎  %% tisty: A 40 60⏎  %% tisty: B 40 200",
      ),
    ) as Flow;
    const out = new Map([["A", { x: 900, y: 900, wide: 90, tall: 42 }]]);
    const back = regrouped(flow, out, new Map([["A", { x: 40, y: 60, wide: 90, tall: 42 }]]));

    expect(back.nodes.find((one) => one.id === "A")?.group).toBe("cli");
    expect(back.places.find((one) => one.id === "A")).toMatchObject({ x: 40, y: 60 });
    expect(regrouped(flow, out)).toBe(flow);
  });

  it("leaves a plain group alone, as only a lane is left by dragging", () => {
    const flow = readFlow(unfolded("flowchart LR⏎  subgraph g [Grupo]⏎    A⏎  end")) as Flow;

    expect(regrouped(flow, new Map([["A", { x: 900, y: 900, wide: 90, tall: 42 }]]))).toBe(flow);
  });
});

// One per line: the style line a node has (`-` for none) → what is set → the line it is left with.
const STYLED = `
- → fill=#dbeafe color=#1e3a8a → style A fill:#dbeafe,color:#1e3a8a
style A fill:#f00,stroke-width:4px → fill=#dcfce7 → style A fill:#dcfce7,stroke-width:4px
style A fill:#f00,stroke:#000 → fill= → style A stroke:#000
style A fill:#f00 → fill= → -
- → stroke= → -
style A fill:rgb(255,0,0) → stroke=#dc2626 → style A fill:rgb(255,0,0),stroke:#dc2626
style A fill:#f00⏎  style A stroke:#000,fill:#0f0 → color=#111 → style A fill:#0f0,stroke:#000,color:#111
`;

describe("colouring a node", () => {
  it("sets or takes off a colour in the node's own style line, keeping the rest of it", async () => {
    for (const case_ of lines(STYLED)) {
      const [line, what, left] = case_.split(" → ").map((one) => one.trim());
      const source = unfolded(`flowchart LR⏎  A[Uno] --> B${line === "-" ? "" : `⏎  ${line}`}`);
      const flow = readFlow(source) as Flow;
      const change = Object.fromEntries(
        what.split(" ").map((one) => {
          const [key, value] = one.split("=");
          return [key, value || null];
        }),
      );
      const text = written(flow, nodeStyled(flow, "A", change));
      expect(
        text
          .split("\n")
          .filter((one) => one.includes("style"))
          .join("|") || "-",
        case_,
      ).toBe(left === "-" ? "-" : `  ${left}`);
      // What Mermaid did not read before, such as unescaped commas, it still does not; the text is kept.
      expect(await valid(text), case_).toBe(await valid(source));
    }
  });

  it("reads the colours each node's style line gives it", () => {
    const flow = readFlow(
      "flowchart LR\n  A --> B\n  style A fill:#fff,stroke:#000,color:#111\n  style B stroke-width:2px",
    ) as Flow;

    expect(looksOf(flow).get("A")).toEqual({ fill: "#fff", stroke: "#000", color: "#111" });
    expect(looksOf(flow).get("B")).toEqual({});
    const twice = readFlow("flowchart LR\n  A\n  style A fill:#f00\n  style A fill:#0f0") as Flow;
    expect(looksOf(twice).get("A")).toEqual({ fill: "#0f0" });
  });
});

const TOOLED = unfolded(
  "flowchart LR⏎  A[Uno] --> B[Dos]⏎  B --> C⏎  C --> D --> E⏎  style A fill:#dbeafe,color:#1e3a8a⏎  %% tisty: A 0 0⏎  %% tisty: B 200 50",
);
const tooled = () => readFlow(TOOLED) as Flow;

describe("the toolbar's changes", () => {
  it("takes out several nodes at once, or none when one of them cannot go", () => {
    const gone = readFlow(written(tooled(), nodesRemoved(tooled(), ["A", "B"]))) as Flow;

    expect(gone.nodes.map((one) => one.id)).toEqual(["C", "D", "E"]);
    expect(nodesRemoved(tooled(), ["A", "D"])).toBeNull();
    expect(pickedRemoved(tooled(), { kind: "nodes", ids: ["A"] })?.picked).toBeNull();
    expect(pickedRemoved(tooled(), { kind: "edge", index: 2 })).toBeNull();
    expect(pickedRemoved(tooled(), null)).toBeNull();
  });

  it("copies a node beside it, with its shape, its colours and its place", async () => {
    const made = nodeDuplicated(tooled(), "A", { x: 0, y: 0, wide: 90, tall: 42 });
    const text = written(tooled(), made);

    expect(text).toContain("  n1[Uno]\n");
    expect(text).toContain("  style n1 fill:#dbeafe,color:#1e3a8a");
    expect(text).toContain("  %% tisty: n1 24 24");
    expect(made?.picked).toEqual({ kind: "node", id: "n1" });
    expect(nodeDuplicated(tooled(), "Z", undefined)).toBeNull();
    const classed = readFlow("flowchart LR\n  A[Uno]:::rojo\n  classDef rojo fill:#f00") as Flow;
    expect(written(classed, nodeDuplicated(classed, "A", undefined))).toContain("  n1[Uno]:::rojo");
    expect(await valid(text)).toBe(true);
  });

  it("lays everything out again, each lane around its shapes", () => {
    const flow = laned();
    const text = written(flow, relaid(flow));
    const again = readFlow(text) as Flow;
    const lanes = lanesOf(again);
    const inCli = flowLaid(again).get("A");

    expect(again.places.filter((one) => one.wide === undefined)).toHaveLength(3);
    expect(lanes.map((one) => one.id)).toEqual(["cli", "caja"]);
    expect(inCli && inCli.x > lanes[0].rect.x && inCli.y > lanes[0].rect.y).toBe(true);
  });
});

const RECTS = new Map([
  ["A", { x: 0, y: 0, wide: 100, tall: 40 }],
  ["B", { x: 200, y: 100, wide: 60, tall: 60 }],
]);

// One per line: how they line up → where A and B go.
const ALIGNED = `
left → 0 0 | 0 100
center → 80 0 | 100 100
right → 160 0 | 200 100
top → 0 0 | 200 0
middle → 0 60 | 200 50
bottom → 0 120 | 200 100
`;

describe("lining up shapes", () => {
  it("moves each shape to the others' edge or middle, along one way only", () => {
    for (const case_ of lines(ALIGNED)) {
      const [how, where] = case_.split(" → ").map((one) => one.trim());
      const moves = alignedMoves(RECTS, ["A", "B", "Z"], how as "left");
      expect(moves.map((one) => `${one.x} ${one.y}`).join(" | "), case_).toBe(where);
    }
  });
});
