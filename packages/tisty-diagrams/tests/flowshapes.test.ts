import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import { type Flow, readFlow, writeFlow } from "../src/flow";
import { nodeAdded, nodeChanged, nodeMoved, nodePlaced, nodeRemoved } from "../src/flowing";
import { formedNode, keysAt, type Shape, shapeNamed } from "../src/flowshapes";

// `⏎` is a line break, so a whole flowchart fits on its line.
const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

// Flowcharts with Mermaid's newer shapes and the window's places, one per line, kept as written.
const WRITTEN = `
flowchart LR⏎  A@{ shape: person, label: "Cliente" } --> B@{ shape: cloud, label: "Proveedor" }
flowchart TB⏎  D@{ shape: cyl, label: "Pedidos" }⏎  N@{ shape: braces, label: "El correo sale al cobrar" }
flowchart LR⏎  I@{ icon: "fa:user", form: "square", label: "Usuario", pos: "t", h: 60 }
flowchart LR⏎  A@{shape: rect} & B@{ shape: doc } --> C
flowchart LR⏎  A["con #quot;comillas#quot;"] --> B@{ shape: docs, label: "Dice #quot;sí#quot;" }
flowchart LR⏎  A --> B⏎  %% tisty: A 10 20⏎  %% tisty: B -5 300
flowchart LR⏎  A --> B⏎⏎  %% tisty: A 10 20⏎
`;

// One per line: how a node is written → its id, its shape (`-` for none) and its words.
const READ = `
A@{ shape: rect, label: "Uno" } → A process Uno
A@{ shape: diamond, label: "¿Sí?" } → A decision ¿Sí?
A@{ shape: odd } → A flag A
A@{ shape: flag } → A paper-tape A
A@{ shape: documents, label: Palabras sueltas } → A docs Palabras sueltas
A@{ label: "Solo texto" } → A - Solo texto
A[Antes] --> A@{ shape: cloud } → A cloud Antes
`;

// What Mermaid would not draw, or the window could not keep, one per line: left as text.
const REFUSED = `
flowchart LR⏎  A@{ shape: nada }
flowchart LR⏎  A@{ shape: rect
flowchart LR⏎  A@{ shape: }
flowchart LR⏎  A@{⏎    shape: rect⏎  }
flowchart LR⏎  A@{ label: "sin cerrar }
flowchart LR⏎  A[Uno]@{ shape: rect }
flowchart LR⏎  A@{ shape: rect }:::rojo --> B
`;

describe("reading Mermaid's newer shapes", () => {
  it("reads every flowchart with them, Mermaid agrees, and gives it back as written", async () => {
    for (const case_ of lines(WRITTEN)) {
      const source = unfolded(case_);
      const flow = readFlow(source);
      expect(flow, case_).not.toBeNull();
      expect(writeFlow(flow as Flow), case_).toBe(source);
      expect(await valid(source), case_).toBe(true);
    }
  });

  it("reads each node's shape by any of Mermaid's names for it, and its words", () => {
    for (const case_ of lines(READ)) {
      const [said, expected] = case_.split(" → ");
      const [id, shape, ...text] = expected.split(" ");
      const node = readFlow(`flowchart LR\n  ${said}`)?.nodes.find((one) => one.id === id);
      expect([node?.shape ?? "-", node?.text], case_).toEqual([shape, text.join(" ")]);
    }
  });

  it("keeps the keys a node was written with", () => {
    const flow = readFlow(unfolded(lines(WRITTEN)[2])) as Flow;

    expect(flow.nodes[0].keys).toEqual([
      ["icon", '"fa:user"'],
      ["form", '"square"'],
      ["label", '"Usuario"'],
      ["pos", '"t"'],
      ["h", "60"],
    ]);
    expect(flow.nodes[0].shape).toBeNull();
  });

  it("refuses what it cannot keep, so the window leaves it as text", () => {
    for (const case_ of lines(REFUSED)) expect(readFlow(unfolded(case_)), case_).toBeNull();
  });

  it("knows no shape by a name Mermaid does not take", () => {
    expect(shapeNamed("nada")).toBeNull();
    expect(keysAt("A[Uno]", 1)).toBeNull();
  });

  it("reads the window's places, and leaves other `%% tisty:` lines in the body", () => {
    const flow = readFlow(
      unfolded(
        "flowchart LR⏎  A --> B⏎  %% tisty: A 10 -20⏎  %% tisty: lo que sea⏎    %% tisty: B 3 4",
      ),
    ) as Flow;

    expect(flow.places.map((one) => [one.id, one.x, one.y])).toEqual([
      ["A", 10, -20],
      ["B", 3, 4],
    ]);
    expect(flow.body).toEqual(["  A --> B", "  %% tisty: lo que sea"]);
  });
});

// One per line: words → shape (`-` for none) → the keys it was written with (`-` for none) → the node.
const FORMED = `
Cliente → person → - → n@{ shape: person, label: "Cliente" }
Dice "sí" → doc → - → n@{ shape: doc, label: "Dice #quot;sí#quot;" }
Cinta → paper-tape → - → n@{ shape: paper-tape, label: "Cinta" }
Uno → process → shape: diamond, label: "Uno" → n@{ shape: rect, label: "Uno" }
Uno → decision → shape: diamond, label: Uno → n@{ shape: diamond, label: Uno }
Otro → - → icon: "fa:user", label: "Usuario", h: 60 → n@{ icon: "fa:user", label: "Otro", h: 60 }
n → cloud → icon: "fa:user" → n@{ icon: "fa:user", shape: cloud }
Nube → cloud → icon: "fa:user" → n@{ icon: "fa:user", shape: cloud, label: "Nube" }
Fin → - → - → n[Fin]
`;

describe("writing Mermaid's newer shapes", () => {
  it("writes a shape with no brackets as `@{ … }`, changing only what changed", async () => {
    for (const case_ of lines(FORMED)) {
      const [text, shape, keys, written] = case_.split(" → ").map((one) => one.trim());
      const said = keys === "-" ? null : (keysAt(`@{ ${keys} }`, 0)?.keys ?? null);
      const line = formedNode("n", text, shape === "-" ? null : (shape as Shape), said);
      expect(line, case_).toBe(written);
      expect(await valid(`flowchart LR\n  ${line}`), case_).toBe(true);
    }
  });
});

const PLACED = unfolded(
  'flowchart LR⏎  A@{ shape: person, label: "Cliente" } --> B⏎  subgraph g1 [Cobro]⏎    C⏎  end⏎  classDef rojo fill:#f00⏎    %% tisty: A 10 20',
);

const written = (flow: Flow, made: { body: readonly string[]; places?: Flow["places"] } | null) =>
  writeFlow({ ...flow, body: made?.body ?? [], places: made?.places ?? flow.places });

describe("placing nodes", () => {
  it("rewrites only the numbers of a node already placed, rounded, and adds a line for a new one", () => {
    const flow = readFlow(PLACED) as Flow;

    expect(written(flow, nodePlaced(flow, "A", { x: 40.4, y: -7.6 }))).toBe(
      PLACED.replace("%% tisty: A 10 20", "%% tisty: A 40 -8"),
    );
    expect(written(flow, nodePlaced(flow, "B", { x: 1, y: 2 }))).toBe(
      `${PLACED}\n  %% tisty: B 1 2`,
    );
  });

  it("adds a node of any shape where it is dropped", async () => {
    const flow = readFlow(PLACED) as Flow;
    const added = written(flow, nodeAdded(flow, "Pedidos", "g1", "database", { x: 5, y: 6 }));

    expect(added).toBe(
      `${PLACED.replace("    C\n  end", "    C\n    n1[(Pedidos)]\n  end")}\n  %% tisty: n1 5 6`,
    );
    expect(written(flow, nodeAdded(flow, "Cliente", null, "person"))).toBe(
      PLACED.replace(
        "  classDef rojo fill:#f00\n",
        '  classDef rojo fill:#f00\n  n1@{ shape: person, label: "Cliente" }\n',
      ),
    );
    expect(await valid(added)).toBe(true);
  });

  it("takes a node's place away with it, and keeps its keys when it changes or moves", async () => {
    const flow = readFlow(PLACED) as Flow;
    const renamed = written(flow, nodeChanged(flow, "A", { text: "Comprador" }));
    const apart = readFlow(unfolded("flowchart LR⏎  A[Uno]:::rojo⏎  subgraph g1⏎  end")) as Flow;
    const moved = written(apart, nodeMoved(apart, "A", "g1"));
    const gone = readFlow(written(flow, nodeRemoved(flow, "A"))) as Flow;

    expect(renamed).toBe(PLACED.replace('label: "Cliente"', 'label: "Comprador"'));
    expect(moved).toContain("    A[Uno]:::rojo\n  end");
    expect(gone.places).toEqual([]);
    expect(gone.nodes.map((one) => one.id)).toEqual(["B", "C"]);
    expect(await valid(renamed)).toBe(true);
  });
});
