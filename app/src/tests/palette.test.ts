import { PALETTES, type Palette } from "tisty-diagrams/read";
import { describe, expect, it } from "vitest";
import { themeCss, themed } from "./theme";

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
  it("are the window's own, as its theme writes them for each", () => {
    const css = themeCss();
    const themes = [
      [PALETTES.light, tokensIn(themed(css, ":root {"))],
      [PALETTES.dark, tokensIn(themed(css, '[data-theme="dark"] {'))],
    ] as const;
    for (const [palette, written] of themes)
      for (const [key, token] of Object.entries(TOKENS))
        expect(numbersOf(palette[key as keyof Palette]), token).toBe(numbersOf(written[token]));
  });
});
