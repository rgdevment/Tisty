import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(process.cwd(), "src/index.css"), "utf8");

type Rgb = [number, number, number];

const themed = (opener: string): string => {
  const at = css.indexOf(opener);
  expect(at, `${opener} must exist`).toBeGreaterThan(-1);
  return css.slice(at, css.indexOf("\n}", at));
};

const LIGHT = themed(":root {");
const DARK = themed('[data-theme="dark"] {');

const hexa = (hex: string): Rgb => [
  parseInt(hex.slice(1, 3), 16),
  parseInt(hex.slice(3, 5), 16),
  parseInt(hex.slice(5, 7), 16),
];

const declared = (block: string, name: string): { rgb: Rgb; alpha: number } => {
  const solid = block.match(new RegExp(`--tisty-${name}: (#[0-9a-fA-F]{6});`));
  if (solid) return { rgb: hexa(solid[1]), alpha: 1 };

  const soft = block.match(
    new RegExp(`--tisty-${name}: rgb\\((\\d+) (\\d+) (\\d+) / ([\\d.]+)\\);`),
  );
  expect(soft, `--tisty-${name} must be a hex or an rgb() with alpha`).toBeTruthy();
  const [, r, g, b, a] = soft as RegExpMatchArray;
  return { rgb: [+r, +g, +b], alpha: +a };
};

const lin = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const lum = ([r, g, b]: Rgb) =>
  0.2126 * lin(r / 255) + 0.7152 * lin(g / 255) + 0.0722 * lin(b / 255);

const ratio = (a: Rgb, b: Rgb) => {
  const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};

const over = (tint: Rgb, alpha: number, base: Rgb): Rgb =>
  base.map((c, i) => tint[i] * alpha + c * (1 - alpha)) as Rgb;

const groundsOf = (block: string, tints: string[]): Rgb[] => {
  const bases = ["bg", "rail", "panel"].map((name) => declared(block, name).rgb);
  const active = declared(block, "active");
  const out: Rgb[] = [];
  for (const base of bases) {
    out.push(base, over(active.rgb, active.alpha, base));
    for (const name of tints) {
      const tint = declared(block, name);
      out.push(over(tint.rgb, tint.alpha, base));
    }
  }
  return out;
};

const MARKS = [
  "mark-date",
  "mark-deadline",
  "mark-list",
  "mark-tag",
  "mark-priority",
  "mark-repeat",
];
const ROWS: string[] = ["accent-soft"];

const worst = (ink: Rgb, on: Rgb[]) => Math.min(...on.map((ground) => ratio(ink, ground)));
const AA = 4.5;
const BIG = 3;

describe("the palette holds up where it is actually painted", () => {
  it("soft clears AA on every tint it sits on", () => {
    for (const block of [LIGHT, DARK]) {
      expect(worst(declared(block, "soft").rgb, groundsOf(block, MARKS))).toBeGreaterThanOrEqual(AA);
    }
  });

  it("faint clears AA on plain and selected rows", () => {
    for (const block of [LIGHT, DARK]) {
      expect(worst(declared(block, "faint").rgb, groundsOf(block, ROWS))).toBeGreaterThanOrEqual(AA);
    }
  });

  it.each(["accent", "urgent", "high"])("%s clears AA in the meta line of a row", (name) => {
    for (const block of [LIGHT, DARK]) {
      expect(worst(declared(block, name).rgb, groundsOf(block, ROWS))).toBeGreaterThanOrEqual(AA);
    }
  });

  it("keeps high apart from urgent, not merely legible", () => {
    for (const block of [LIGHT, DARK]) {
      const apart = ratio(declared(block, "high").rgb, declared(block, "urgent").rgb);
      expect(apart).toBeGreaterThanOrEqual(1.35);
    }
  });

  it("keeps the three greys telling one another apart", () => {
    for (const block of [LIGHT, DARK]) {
      const [ink, soft, faint] = ["ink", "soft", "faint"].map((n) => declared(block, n).rgb);
      expect(ratio(ink, soft)).toBeGreaterThanOrEqual(1.35);
      expect(ratio(soft, faint)).toBeGreaterThanOrEqual(1.2);
    }
  });

  it.each(["accent", "urgent"])("carries its own ground as ink on %s", (name) => {
    for (const block of [LIGHT, DARK]) {
      const ink = declared(block, "bg").rgb;
      expect(ratio(ink, declared(block, name).rgb)).toBeGreaterThanOrEqual(BIG);
    }
  });

  it("keeps a disabled button readable", () => {
    for (const block of [LIGHT, DARK]) {
      expect(worst(declared(block, "soft").rgb, groundsOf(block, ["hair"]))).toBeGreaterThanOrEqual(
        AA,
      );
    }
  });

  it.each(["ink", "soft", "faint", "accent", "urgent", "high", "bg", "panel", "rail"])(
    "wires --color-%s to the token it was measured on",
    (name) => {
      expect(css).toContain(`--color-${name}: var(--tisty-${name});`);
    },
  );
});
