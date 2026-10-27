import mermaid from "mermaid";
import { beforeAll, describe, expect, it } from "vitest";
import {
  type Changed,
  ideaAt,
  moved,
  type Path,
  renamed,
  reshaped,
  withChild,
  without,
  withSibling,
} from "../ui/drawing/minding";
import { laidOut } from "../ui/drawing/mindlayout";
import { formed, type Mind, readMind, type Shape, writeMind } from "../ui/drawing/mindmap";

// `⏎` is a line break and `⇥` a tab, so a whole map fits on its line.
const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n").replace(/⇥/g, "\t");

const lines = (table: string) => table.split("\n").filter((line) => line.trim());

// Maps as people write them, one per line: each must come back byte for byte.
const WRITTEN = `
mindmap⏎  root((Tisty Share))⏎    Cifrado⏎      Frase de 24 palabras⏎      Llave por equipo⏎    Servidor⏎      Cloudflare Workers⏎      D1 y R2
mindmap⏎  root((Ideas))⏎    a[Cuadrada]⏎    b(Redondeada)⏎    c))Explosión((⏎    d)Nube(⏎    e{{Hexágono}}
mindmap⏎    Raíz⏎        Uno⏎            Dos⏎        Tres⏎
---⏎title: Con propiedades⏎---⏎%%{init: {"theme": "forest"}}%%⏎mindmap⏎  root⏎    %% una nota⏎    A⏎    ::icon(fa fa-book)⏎    B⏎    :::urgente grande⏎⏎    C
mindmap⏎⇥Raíz⏎⇥⇥Hija⏎⇥⇥⇥Nieta
mindmap⏎  root["Con (paréntesis)"]⏎    x("Comillas #quot;dentro#quot;")⏎    y["\`**negrita**\`"]
`;

// Maps the window cannot keep without changing them, one per line.
const REFUSED = `
mindmap⏎  Una⏎ Otra raíz
mindmap⏎  Raíz⏎    Texto con (paréntesis) suelto
graph TD⏎  a --> b
mindmap
`;

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

describe("reading and writing a mind map", () => {
  it("gives back every map as it was written", async () => {
    for (const case_ of lines(WRITTEN)) {
      const source = unfolded(case_);
      const mind = readMind(source);
      expect(mind, case_).not.toBeNull();
      expect(writeMind(mind as Mind), case_).toBe(source);
    }
  });

  it("refuses what it cannot keep, so the window leaves it as text", () => {
    for (const case_ of lines(REFUSED)) expect(readMind(unfolded(case_)), case_).toBeNull();
  });

  it("reads the shape, the words and the id of each idea", () => {
    const mind = readMind(unfolded(lines(WRITTEN)[1])) as Mind;
    const said = mind.root.children.map((one) => [one.id, one.shape, one.text]);
    expect(mind.root).toMatchObject({ shape: "circle", text: "Ideas", id: "root" });
    expect(said).toEqual([
      ["a", "square", "Cuadrada"],
      ["b", "rounded", "Redondeada"],
      ["c", "bang", "Explosión"],
      ["d", "cloud", "Nube"],
      ["e", "hexagon", "Hexágono"],
    ]);
  });
});

// One per line: words → shape → what the window writes, which Mermaid must read.
const FORMED = `
Idea → plain → Idea
Con (paréntesis) → plain → ("Con (paréntesis)")
Dice "hola" → square → [Dice #quot;hola#quot;]
 espacios  → circle → ((" espacios "))
Corte → bang → ))Corte((
Nube → cloud → )Nube(
Seis → hexagon → {{Seis}}
`;

describe("writing an idea", () => {
  it("writes each shape so Mermaid reads it, quoting what would break it", async () => {
    for (const case_ of lines(FORMED)) {
      const [text, shape, written] = case_.split(" → ");
      const line = formed({ text, shape: shape as Shape });
      expect(line, case_).toBe(written.trim());
      expect(await valid(`mindmap\n  root\n    ${line}`), case_).toBe(true);
    }
  });
});

const MAP = unfolded(
  "mindmap⏎  root((Share))⏎    Cifrado⏎      Frase⏎    ::icon(fa fa-key)⏎    Servidor⏎      Workers⏎      D1",
);

const at = (made: Changed | null) => made as Changed;

// One per line: what is done → the map it leaves → the idea chosen after.
const DONE: [string, (mind: Mind) => Changed | null, string, Path][] = [
  [
    "a child of a branch goes last, under its siblings' indent",
    (mind) => withChild(mind, [1], "Nueva"),
    "mindmap⏎  root((Share))⏎    Cifrado⏎      Frase⏎    ::icon(fa fa-key)⏎    Servidor⏎      Workers⏎      D1⏎      Nueva",
    [1, 2],
  ],
  [
    "a sibling goes right after the chosen idea",
    (mind) => withSibling(mind, [0], "Visor"),
    "mindmap⏎  root((Share))⏎    Cifrado⏎      Frase⏎    ::icon(fa fa-key)⏎    Visor⏎    Servidor⏎      Workers⏎      D1",
    [1],
  ],
  [
    "a sibling of the root is a child of it",
    (mind) => withSibling(mind, [], "Otra"),
    "mindmap⏎  root((Share))⏎    Cifrado⏎      Frase⏎    ::icon(fa fa-key)⏎    Servidor⏎      Workers⏎      D1⏎    Otra",
    [2],
  ],
  [
    "removing takes its branch, icon and all, and chooses the one before",
    (mind) => without(mind, [1, 1]),
    "mindmap⏎  root((Share))⏎    Cifrado⏎      Frase⏎    ::icon(fa fa-key)⏎    Servidor⏎      Workers",
    [1, 0],
  ],
  [
    "renaming writes only that idea anew",
    (mind) => renamed(mind, [0], "Cifrado (24 palabras)"),
    'mindmap⏎  root((Share))⏎    ("Cifrado (24 palabras)")⏎      Frase⏎    ::icon(fa fa-key)⏎    Servidor⏎      Workers⏎      D1',
    [0],
  ],
  [
    "a new shape keeps the words and the id",
    (mind) => reshaped(mind, [], "hexagon"),
    "mindmap⏎  root{{Share}}⏎    Cifrado⏎      Frase⏎    ::icon(fa fa-key)⏎    Servidor⏎      Workers⏎      D1",
    [],
  ],
  [
    "moving takes the branch to the end of another, at its depth",
    (mind) => moved(mind, [0], [1, 0]),
    "mindmap⏎  root((Share))⏎    Servidor⏎      Workers⏎        Cifrado⏎          Frase⏎    ::icon(fa fa-key)⏎      D1",
    [0, 0, 0],
  ],
  [
    "moving a later sibling's child up finds where it lands",
    (mind) => moved(mind, [1, 1], []),
    "mindmap⏎  root((Share))⏎    Cifrado⏎      Frase⏎    ::icon(fa fa-key)⏎    Servidor⏎      Workers⏎    D1",
    [2],
  ],
];

describe("changing a mind map", () => {
  it("does each change, keeps the rest as written and leaves valid Mermaid", async () => {
    for (const [said, change, after, chosen] of DONE) {
      const made = at(change(readMind(MAP) as Mind));
      const written = writeMind(made.mind);
      expect(written, said).toBe(unfolded(after));
      expect(made.chosen, said).toEqual(chosen);
      expect(await valid(written), said).toBe(true);
    }
  });

  it("never moves an idea into itself or its own branch, nor the root anywhere", () => {
    const mind = readMind(MAP) as Mind;
    expect(moved(mind, [1], [1, 0])).toBeNull();
    expect(moved(mind, [1], [1])).toBeNull();
    expect(moved(mind, [], [0])).toBeNull();
    expect(moved(mind, [0], [9])).toBeNull();
  });

  it("leaves the root and missing ideas alone", () => {
    const mind = readMind(MAP) as Mind;
    expect(without(mind, []).mind).toBe(mind);
    expect(without(mind, [7]).mind).toBe(mind);
    expect(withChild(mind, [7], "x").mind).toBe(mind);
    expect(ideaAt(mind, [1, 1])?.text).toBe("D1");
    expect(renamed(mind, [0], "Cifrado").mind.root).toBe(mind.root);
  });
});

describe("laying out a mind map", () => {
  it("puts the root in the middle and first branches right and left in turn", () => {
    const laid = laidOut(readMind(MAP) as Mind);
    const [root, ...rest] = laid.ideas;
    const first = rest.find((one) => one.path.join() === "0");
    const second = rest.find((one) => one.path.join() === "1");

    expect(laid.ideas).toHaveLength(6);
    expect(laid.lines).toHaveLength(5);
    expect(first && first.x > root.x + root.wide).toBe(true);
    expect(second && second.x + second.wide < root.x).toBe(true);
    for (const one of laid.ideas) {
      expect(one.x).toBeGreaterThanOrEqual(0);
      expect(one.y).toBeGreaterThanOrEqual(0);
      expect(one.x + one.wide).toBeLessThanOrEqual(laid.size.wide);
      expect(one.y + one.tall).toBeLessThanOrEqual(laid.size.tall);
    }
  });
});
