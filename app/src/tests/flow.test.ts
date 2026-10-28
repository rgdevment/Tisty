import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Flow, formedNode, readFlow, type Shape, writeFlow } from "../ui/drawing/flow";
import {
  directionChanged,
  edgeChanged,
  edgeRemoved,
  groupAdded,
  groupRemoved,
  groupRenamed,
  linked,
  nodeAdded,
  nodeChanged,
  nodeMoved,
  nodeRemoved,
} from "../ui/drawing/flowing";

// `⏎` is a line break, so a whole flowchart fits on its line.
const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

// Flowcharts as people write them, one per line: each is read and given back as written.
const WRITTEN = String.raw`
flowchart LR⏎  A[Inicio] --> B{¿Pago?}⏎  B -->|sí| C(Proceso)⏎  B -- no --> D((Fin))
graph TD;⏎  A --> B --> C⏎  A & B --> D;⏎  style A fill:#f9f⏎  classDef rojo fill:#f00⏎  class C rojo
flowchart TB⏎  E([Estadio]) --- F[[Sub]]⏎  G[(Base)] -.-> H{{Hex}}⏎  I[/Para/] ==> J>Asim]⏎  K[\Inv\] -.- L[/Trap\]⏎  M(((Doble)))
flowchart LR⏎  subgraph g1 [Grupo uno]⏎    C⏎    C --> X⏎  end⏎  subgraph Solo⏎    D⏎  end⏎  %% una nota⏎  C:::rojo --> D⏎
---⏎config:⏎  flowchart:⏎    curve: basis⏎---⏎flowchart-elk RL⏎  A --> B
---⏎title: Pagos⏎---⏎flowchart TD⏎  A["con (paréntesis) y #quot;comillas#quot;"] --> B⏎  A -.->|tal vez| B⏎  A ==>|seguro| C⏎  A ---|enlace| D⏎  A == firme ==> E⏎  A -. suave .-> F
`;

// Flowcharts the window cannot keep without changing them, one per line.
const REFUSED = `
flowchart LR⏎  A --> B⏎  subgraph g⏎    A
flowchart LR⏎  end
flowchart LR⏎  A --> ⏎
flowchart LR⏎  A@{ shape: rect }
sequenceDiagram⏎  a->>b: hola
`;

describe("reading a flowchart", () => {
  it("reads every flowchart people write, Mermaid agrees, and gives it back as written", async () => {
    for (const case_ of lines(WRITTEN)) {
      const source = unfolded(case_);
      const flow = readFlow(source);
      expect(flow, case_).not.toBeNull();
      expect(writeFlow(flow as Flow), case_).toBe(source);
      expect(await valid(source), case_).toBe(true);
    }
  });

  it("refuses what it cannot keep, so the window leaves it as text", () => {
    for (const case_ of lines(REFUSED)) expect(readFlow(unfolded(case_)), case_).toBeNull();
  });

  it("reads nodes with their words and shapes, edges with their arrows and labels", () => {
    const flow = readFlow(unfolded(lines(WRITTEN)[0])) as Flow;

    expect(flow.direction).toBe("LR");
    expect(flow.nodes.map((one) => [one.id, one.text, one.shape])).toEqual([
      ["A", "Inicio", "process"],
      ["B", "¿Pago?", "decision"],
      ["C", "Proceso", "rounded"],
      ["D", "Fin", "circle"],
    ]);
    expect(flow.edges.map((one) => [one.from, one.arrow, one.label, one.to])).toEqual([
      ["A", "-->", "", "B"],
      ["B", "-->", "sí", "C"],
      ["B", "-->", "no", "D"],
    ]);
  });

  it("reads chains, groups joined by & and every kind of arrow", () => {
    const chained = readFlow(unfolded(lines(WRITTEN)[1])) as Flow;
    const worded = readFlow(unfolded(lines(WRITTEN)[5])) as Flow;

    expect(chained.direction).toBe("TB");
    expect(chained.edges.map((one) => `${one.from}${one.arrow}${one.to}:${one.alone}`)).toEqual([
      "A-->B:false",
      "B-->C:false",
      "A-->D:false",
      "B-->D:false",
    ]);
    expect(worded.nodes[0].text).toBe('con (paréntesis) y "comillas"');
    expect(worded.edges.map((one) => [one.arrow, one.label])).toEqual([
      ["-->", ""],
      ["-.->", "tal vez"],
      ["==>", "seguro"],
      ["---", "enlace"],
      ["==>", "firme"],
      ["-.->", "suave"],
    ]);
  });

  it("reads the direction after front matter and a layout word", () => {
    expect(readFlow(unfolded(lines(WRITTEN)[4]))?.direction).toBe("RL");
  });

  it("puts a node in the group that lists it, as Mermaid does, even when named before", () => {
    const flow = readFlow(unfolded(lines(WRITTEN)[3])) as Flow;
    const later = readFlow("flowchart LR\n  A --> B\n  subgraph g [G]\n    A\n  end") as Flow;

    expect(later.nodes.map((one) => [one.id, one.group])).toEqual([
      ["A", "g"],
      ["B", null],
    ]);

    expect(flow.groups.map((one) => [one.id, one.title])).toEqual([
      ["g1", "Grupo uno"],
      ["Solo", "Solo"],
    ]);
    expect(flow.nodes.map((one) => [one.id, one.group])).toEqual([
      ["C", "g1"],
      ["X", "g1"],
      ["D", "Solo"],
    ]);
  });
});

// One per line: words → shape → what the window writes, which Mermaid must read.
const FORMED = `
Inicio → process → n[Inicio]
¿Pagó? → decision → n{¿Pagó?}
Con (nota) → rounded → n("Con (nota)")
Dice "sí" → stadium → n(["Dice #quot;sí#quot;"])
Fin → circle → n((Fin))
Datos → database → n[(Datos)]
`;

describe("writing a node", () => {
  it("writes each shape so Mermaid reads it, quoting what would break it", async () => {
    for (const case_ of lines(FORMED)) {
      const [text, shape, written] = case_.split(" → ");
      const line = formedNode("n", text.trim(), shape as Shape);
      expect(line, case_).toBe(written.trim());
      expect(await valid(`flowchart LR\n  ${line}`), case_).toBe(true);
    }
  });
});

const MAP = unfolded(
  "flowchart LR⏎  A[Inicio] --> B{¿Pago?}⏎  B -->|sí| C(Proceso)⏎  subgraph g1 [Cobro]⏎    C⏎  end⏎  C --> D --> E⏎  style D fill:#f9f",
);

// One per line: a flowchart → what is removed → the text it leaves.
const REMOVED = `
flowchart LR⏎  A --> Z → edge 0 → flowchart LR⏎  A⏎  Z
flowchart LR⏎  A --> Z⏎  A --> Y → edge 0 → flowchart LR⏎  Z⏎  A --> Y
flowchart LR⏎  A[Uno]:::rojo --> Z⏎  Z --> A → edge 0 → flowchart LR⏎  A[Uno]:::rojo⏎  Z --> A
flowchart LR⏎  A --> B⏎  B --> C⏎  C --> D⏎  linkStyle 0,2 stroke:red⏎  linkStyle 1 stroke:blue → edge 1 → flowchart LR⏎  A --> B⏎  C --> D⏎  linkStyle 0,1 stroke:red
flowchart LR⏎  A --> B⏎  B --> C⏎  linkStyle default stroke:red → edge 0 → flowchart LR⏎  A⏎  B --> C⏎  linkStyle default stroke:red
flowchart LR⏎  A --> B⏎  B --> C⏎  linkStyle 1 stroke:red → node A → flowchart LR⏎  B --> C⏎  linkStyle 0 stroke:red
flowchart LR⏎  A --> B[Dos]⏎  B --> C → node A → flowchart LR⏎  B[Dos]⏎  B --> C
`;

const flowOf = (source = MAP) => readFlow(source) as Flow;
const bodyText = (body: readonly string[] | undefined) =>
  writeFlow({ head: flowOf().head, body: body ?? [], tail: [] });

describe("changing a flowchart", () => {
  it("rewrites only the words or shape of a node, where they are written", async () => {
    const renamed = bodyText(nodeChanged(flowOf(), "B", { text: "¿Pagó (todo)?" }).body);
    const reshaped = bodyText(nodeChanged(flowOf(), "A", { shape: "stadium" }).body);
    const bare = bodyText(nodeChanged(flowOf(), "E", { text: "Fin", shape: "circle" }).body);

    expect(renamed).toBe(MAP.replace("B{¿Pago?}", 'B{"¿Pagó (todo)?"}'));
    expect(reshaped).toBe(MAP.replace("A[Inicio]", "A([Inicio])"));
    expect(bare).toBe(MAP.replace("D --> E", "D --> E((Fin))"));
    for (const one of [renamed, reshaped, bare]) expect(await valid(one)).toBe(true);
  });

  it("adds a node at the end or inside a group, and joins two nodes once", async () => {
    const outside = bodyText(nodeAdded(flowOf(), "Nuevo", null).body);
    const inside = bodyText(nodeAdded(flowOf(), "Nuevo", "g1").body);
    const joined = linked(flowOf(), "A", "E");
    const again = linked(flowOf(), "A", "B");

    expect(outside).toBe(`${MAP}\n  n1[Nuevo]`);
    expect(inside).toBe(MAP.replace("    C\n  end", "    C\n    n1[Nuevo]\n  end"));
    expect(bodyText(joined.body)).toBe(`${MAP}\n  A --> E`);
    expect(joined.picked).toEqual({ kind: "edge", index: 4 });
    expect(again).toEqual({ body: flowOf().body, picked: { kind: "edge", index: 0 } });
    for (const one of [outside, inside, bodyText(joined.body)]) expect(await valid(one)).toBe(true);
  });

  it("rewrites an edge's arrow and label, even inside a chain", async () => {
    const labelled = bodyText(edgeChanged(flowOf(), 0, { label: "empieza" }).body);
    const dotted = bodyText(edgeChanged(flowOf(), 3, { arrow: "-.->" }).body);

    expect(labelled).toBe(MAP.replace("A[Inicio] --> B", "A[Inicio] -->|empieza| B"));
    expect(dotted).toBe(MAP.replace("D --> E", "D -.-> E"));
    for (const one of [labelled, dotted]) expect(await valid(one)).toBe(true);
  });

  it("takes out a lone edge, but never one link of a chain", () => {
    expect(bodyText(edgeRemoved(flowOf(), 0)?.body)).toBe(
      MAP.replace("  A[Inicio] --> B{¿Pago?}\n", "  A[Inicio]\n  B{¿Pago?}\n"),
    );
    expect(edgeRemoved(flowOf(), 2)).toBeNull();
  });

  it("takes out a node with its lone edges and its style, but not one a chain still names", () => {
    expect(bodyText(nodeRemoved(flowOf(), "A")?.body)).toBe(
      MAP.replace("  A[Inicio] --> B{¿Pago?}\n", "  B{¿Pago?}\n"),
    );
    expect(nodeRemoved(flowOf(), "D")).toBeNull();
  });

  it("keeps what a removed line also held, and renumbers the styles of the edges after it", async () => {
    for (const case_ of lines(REMOVED)) {
      const [source, what, expected] = case_.split(" → ").map(unfolded);
      const [kind, which] = what.split(" ");
      const flow = flowOf(source);
      const done = kind === "edge" ? edgeRemoved(flow, Number(which)) : nodeRemoved(flow, which);
      const written = writeFlow({ head: flow.head, body: done?.body ?? [], tail: [] });
      expect(written, case_).toBe(expected);
      expect(await valid(written), case_).toBe(true);
    }
  });

  it("adds, renames and takes apart a group, and moves a node in and out of one", async () => {
    const added = bodyText(groupAdded(flowOf(), "Envío").body);
    const renamed = bodyText(groupRenamed(flowOf(), "g1", "Cobro final").body);
    const apart = bodyText(groupRemoved(flowOf(), "g1").body);
    const outOf = bodyText(nodeMoved(flowOf(), "C", null)?.body);
    const into = bodyText(nodeMoved(flowOf(), "A", "g1")?.body);

    expect(added).toBe(`${MAP}\n  subgraph g2 [Envío]\n  end`);
    expect(renamed).toBe(MAP.replace("subgraph g1 [Cobro]", "subgraph g1 [Cobro final]"));
    expect(apart).toBe(
      MAP.replace("  subgraph g1 [Cobro]\n", "").replace("    C\n  end\n", "    C\n"),
    );
    expect(outOf).toBe(`${MAP.replace("    C\n", "")}\n  C`);
    expect(into).toBe(MAP.replace("    C\n  end", "    C\n    A\n  end"));
    for (const one of [added, renamed, apart, outOf, into]) expect(await valid(one)).toBe(true);
  });

  it("turns the whole chart another way, keeping its layout word", () => {
    expect(directionChanged(flowOf(), "TB")).toEqual(["flowchart TB"]);
    expect(directionChanged(flowOf("flowchart-elk TD\n  A --> B"), "LR")).toEqual([
      "flowchart-elk LR",
    ]);
  });
});
