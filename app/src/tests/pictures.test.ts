import { describe, expect, it } from "vitest";
import { cut, fitted, PALETTES, svgOf } from "../ui/drawing/picture";
import { pictureOf } from "../ui/drawing/pictures";

// `⏎` is a line break, so a whole diagram fits on its line.
const unfolded = (said: string) => said.trim().replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

const tenth = (n: number) => Math.round(n * 10) / 10;
const parsed = (svg: string) => new DOMParser().parseFromString(svg, "image/svg+xml");

// One per line: a diagram the window draws → words its picture must show, as SVG writes them.
const PICTURED = `
mindmap⏎  root((Idea & <más>))⏎    Rama uno⏎      Hoja⏎    b)Nube(⏎    c{{Hex}}⏎    d))Bang(( → Idea &amp; &lt;más&gt; | Rama uno | Hoja | Nube | Hex | Bang
erDiagram⏎  cliente {⏎    int id PK⏎    string nombre⏎  }⏎  pedido {⏎    int id PK⏎  }⏎  cliente ||--o{ pedido : hace⏎  %% tisty: pedido 400 60 required id → cliente | pedido | nombre | id * | 1 · hace · 0..N | x="400" y="60"
flowchart LR⏎  A[Inicio] --> B{¿Sí?}⏎  B -->|sí| C((Fin))⏎  B -.-> D[(Base)]⏎  D ==> E[[Sub]]⏎  E --- F([Est])⏎  F --> G(((Doble)))⏎  G --> H{{Hex}}⏎  H --> I(Redondo)⏎  subgraph g1 [Grupo]⏎    A⏎  end → Inicio | ¿Sí? | Fin | Base | Sub | Est | Doble | Hex | Redondo | Grupo | >sí< | stroke-dasharray="6 4" | marker-end
classDiagram⏎  class Animal {⏎    <<abstract>>⏎    +String name⏎  }⏎  class Duck⏎  Animal <|-- Duck : hereda → Animal | Duck | +String name | «abstract» | hereda &lt;|--
`;

describe("the picture of a diagram the window draws", () => {
  it("is well-formed SVG of the window's size, with every word the window shows, in either theme", () => {
    for (const case_ of lines(PICTURED)) {
      const [source, said] = case_.split(" → ");
      const picture = pictureOf(unfolded(source));
      expect(picture, case_).not.toBeNull();
      if (!picture) continue;
      for (const palette of [PALETTES.light, PALETTES.dark]) {
        const svg = svgOf(picture, palette, palette.sheet, "Inter");
        const read = parsed(svg);
        expect(read.querySelector("parsererror"), case_).toBeNull();
        expect(read.documentElement.getAttribute("viewBox"), case_).toBe(
          `0 0 ${tenth(picture.size.wide)} ${tenth(picture.size.tall)}`,
        );
        expect(svg, case_).not.toMatch(/NaN|undefined/);
        for (const word of said.split(" | ")) expect(svg, `${case_} · ${word}`).toContain(word);
      }
    }
  });

  it("brings a ground only when one is asked for, in the colours of the theme asked", () => {
    const picture = pictureOf("flowchart TD\n  A --> B\n");
    if (!picture) throw new Error("no picture");

    const clear = svgOf(picture, PALETTES.light, null, "Inter");
    const dark = svgOf(picture, PALETTES.dark, PALETTES.dark.sheet, "Inter");

    expect(clear).not.toContain('width="100%"');
    expect(clear).toContain(PALETTES.light.accent);
    expect(dark).toContain(`<rect width="100%" height="100%" fill="${PALETTES.dark.sheet}"/>`);
    expect(dark).toContain(PALETTES.dark.panel);
  });

  it("leaves to Mermaid the kinds the window does not draw, and what it cannot read", () => {
    expect(pictureOf("sequenceDiagram\n  A->>B: hola\n")).toBeNull();
    expect(pictureOf("classDiagram\n  namespace Zoo {\n    class A\n  }\n")).toBeNull();
  });
});

// One per line: words → room → what fits, cut as the window cuts it.
const FITTED = `
Corto | 100 | Corto
Una frase bastante larga | 70 | Una frase…
X | 1 | X
`;

describe("drawing words and shapes as SVG", () => {
  it("cuts words that would not fit, with an ellipsis", () => {
    for (const case_ of lines(FITTED)) {
      const [text, room, fits] = case_.split(" | ").map((one) => one.trim());
      expect(fitted(text, Number(room), 7), case_).toBe(fits);
    }
  });

  it("lays a CSS polygon over its box as SVG points", () => {
    const rect = { x: 10, y: 20, wide: 100, tall: 40 };

    expect(cut("polygon(50% 0, 100% 50%, 50% 100%, 0 50%)", rect, { fill: "red" })).toBe(
      '<polygon points="60,20 110,40 60,60 10,40" fill="red"/>',
    );
  });
});
