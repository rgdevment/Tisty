import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { PALETTES, type Palette } from "tisty-diagrams/read";
import { describe, expect, it } from "vitest";

const TOKENS: Record<keyof Palette, string> = {
  sheet: "sheet",
  ink: "ink",
  soft: "soft",
  faint: "faint",
  line: "line",
  hover: "hover",
  panel: "panel",
  desk: "desk",
  accent: "accent",
  accentSoft: "accent-soft",
};

// `rgb(0 0 0 / 0.16)` and `rgba(0,0,0,0.16)` are the same colour written two ways.
const numbersOf = (color = "") => (color.match(/#[0-9a-f]+|[\d.]+/gi) ?? []).join(" ");
const tokensIn = (block = "") =>
  Object.fromEntries(
    [...block.matchAll(/--tisty-([\w-]+):\s*([^;]+);/g)].map(([, name, value]) => [name, value]),
  );

describe("the colours of a picture", () => {
  it("are the window's own, as index.css writes them for each theme", () => {
    const css = readFileSync(resolve("src/index.css"), "utf8");
    const themes = [
      [PALETTES.light, tokensIn(/:root\s*\{([^}]*)\}/.exec(css)?.[1])],
      [PALETTES.dark, tokensIn(/\[data-theme="dark"\]\s*\{([^}]*)\}/.exec(css)?.[1])],
    ] as const;
    for (const [palette, written] of themes)
      for (const [key, token] of Object.entries(TOKENS))
        expect(numbersOf(palette[key as keyof Palette]), token).toBe(numbersOf(written[token]));
  });
});
