import { describe, expect, it } from "vitest";
import { asRead, fitted, kindOf, LEAST, MOST, sizeOf, zoomedAt } from "../src/framing";

// One case per line: what the diagram says (`⏎` is a line break) → the kind the window names.
const KINDS = `
mindmap⏎  root((Tisty)) → mindmap
erDiagram⏎  a ||--o{ b : has → erDiagram
flowchart LR⏎  a --> b → flowchart
graph TD;⏎  a --> b → flowchart
flowchart-elk TD⏎  a --> b → flowchart
stateDiagram-v2⏎  [*] --> a → other
classDiagram⏎  class A → classDiagram
sequenceDiagram⏎  a->>b: hi → other
%% a note first⏎⏎mindmap⏎  root → mindmap
%%{init: {"theme": "dark"}}%%⏎erDiagram → erDiagram
---⏎title: Pagos⏎---⏎erDiagram → erDiagram
  ERDIAGRAM → erDiagram
⏎⏎ → other
`;

// One case per line: what the window holds → what it reads back as once saved.
const READ_BACK = `
graph TD⏎A-->B → graph TD⏎A-->B⏎
graph TD⏎A-->B⏎ → graph TD⏎A-->B⏎
graph TD␍⏎A-->B␍⏎ → graph TD⏎A-->B⏎
a⏎⏎b → a⏎⏎b⏎
 →
`;

const unfolded = (said: string) => said.replace(/⏎/g, "\n").replace(/␍/g, "\r");

const cases = (table: string) =>
  table
    .split("\n")
    .filter((line) => line.includes(" → "))
    .map((line) => line.split(" → ").map(unfolded) as [string, string]);

describe("what the window reads from a diagram's text", () => {
  it("names each kind by the word that opens it, past comments and front matter", () => {
    for (const [source, kind] of cases(KINDS)) expect(kindOf(source), source).toBe(kind);
  });

  it("holds what it saved the way the document reads it back", () => {
    for (const [held, back] of cases(READ_BACK)) expect(asRead(held), held).toBe(back);
  });

  it("reads the size Mermaid drew at from the viewBox, and nothing from a broken one", () => {
    expect(sizeOf('<svg viewBox="0 0 640.5 320" width="100%">')).toEqual({
      wide: 640.5,
      tall: 320,
    });
    expect(sizeOf('<svg viewBox="-8, -8, 100, 50">')).toEqual({ wide: 100, tall: 50 });
    expect(sizeOf('<svg viewBox="0 0 0 50">')).toBeNull();
    expect(sizeOf("<svg>")).toBeNull();
  });
});

describe("how the canvas frames a drawing", () => {
  it("zooms around the pointer, so the point under it stays put", () => {
    const view = zoomedAt({ scale: 1, x: 10, y: 20 }, 2, { x: 110, y: 120 });

    expect(view).toEqual({ scale: 2, x: -90, y: -80 });
    expect((110 - view.x) / view.scale).toBe(100);
  });

  it("never zooms past its bounds", () => {
    expect(zoomedAt({ scale: 3, x: 0, y: 0 }, 10, { x: 0, y: 0 }).scale).toBe(MOST);
    expect(zoomedAt({ scale: 0.3, x: 0, y: 0 }, 0.1, { x: 0, y: 0 }).scale).toBe(LEAST);
  });

  it("fits a big drawing in the room and centres a small one at its own size", () => {
    expect(fitted({ wide: 1000, tall: 500 }, { wide: 564, tall: 564 })).toEqual({
      scale: 0.5,
      x: 32,
      y: 157,
    });
    expect(fitted({ wide: 100, tall: 50 }, { wide: 500, tall: 300 })).toEqual({
      scale: 1,
      x: 200,
      y: 125,
    });
    expect(fitted({ wide: 0, tall: 0 }, { wide: 500, tall: 300 })).toEqual({
      scale: 1,
      x: 32,
      y: 32,
    });
  });
});
