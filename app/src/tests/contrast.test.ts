import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

/// Read from disk on purpose: importing it would hand back what Tailwind
/// produced, and what has to hold is what the palette actually says.
const css = readFileSync(resolve(process.cwd(), "src/index.css"), "utf8");

type Rgb = [number, number, number];

const token = (name: string): [Rgb, Rgb] => {
  const found = [...css.matchAll(new RegExp(`--tisty-${name}: (#[0-9a-f]{6});`, "g"))];
  expect(found.length, `--tisty-${name} must be set in both themes`).toBe(2);
  return [hexa(found[0][1]), hexa(found[1][1])];
};

const hexa = (hex: string): Rgb => [
  parseInt(hex.slice(1, 3), 16),
  parseInt(hex.slice(3, 5), 16),
  parseInt(hex.slice(5, 7), 16),
];

const lin = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const lum = ([r, g, b]: Rgb) =>
  0.2126 * lin(r / 255) + 0.7152 * lin(g / 255) + 0.0722 * lin(b / 255);

const ratio = (a: Rgb, b: Rgb) => {
  const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};

/// What a translucent tint actually resolves to over a given base.
const over = (tint: Rgb, alpha: number, base: Rgb): Rgb =>
  base.map((c, i) => tint[i] * alpha + c * (1 - alpha)) as Rgb;

const LIGHT = { bg: hexa("#ffffff"), rail: hexa("#f6f6f8"), panel: hexa("#fbfbfd") };
const DARK = { bg: hexa("#1c1c1e"), rail: hexa("#191919"), panel: hexa("#202022") };

/// A selected row is the ordinary state of the screen, not an edge case — and
/// it was the ground missing from the first measurement.
const grounds = (bases: Record<string, Rgb>, veil: Rgb, alpha: number, extra: Rgb[]) => [
  ...Object.values(bases),
  ...Object.values(bases).map((base) => over(veil, alpha, base)),
  ...extra,
];

const lightOn = grounds(LIGHT, [0, 0, 0], 0.06, [
  over([124, 92, 222], 0.15, LIGHT.panel),
  over([10, 124, 255], 0.1, LIGHT.bg),
  over([0, 0, 0], 0.035, LIGHT.bg),
]);
const darkOn = grounds(DARK, [255, 255, 255], 0.08, [
  over([160, 130, 255], 0.24, DARK.panel),
  over([61, 149, 255], 0.14, DARK.bg),
  over([255, 255, 255], 0.045, DARK.bg),
]);
const lightRows = grounds(LIGHT, [0, 0, 0], 0.06, []);
const darkRows = grounds(DARK, [255, 255, 255], 0.08, []);

const worst = (ink: Rgb, on: Rgb[]) => Math.min(...on.map((ground) => ratio(ink, ground)));

const AA = 4.5;

describe("every text token clears AA on the grounds it lands on", () => {
  it.each([
    ["faint", lightOn, darkOn],
    ["soft", lightOn, darkOn],
  ])("%s, everywhere it is painted", (name, light, dark) => {
    const [inLight, inDark] = token(name);

    expect(worst(inLight, light)).toBeGreaterThanOrEqual(AA);
    expect(worst(inDark, dark)).toBeGreaterThanOrEqual(AA);
  });

  it.each([
    ["accent", lightRows, darkRows],
    ["urgent", lightRows, darkRows],
    ["high", lightRows, darkRows],
  ])("%s, in the meta line of a row", (name, light, dark) => {
    const [inLight, inDark] = token(name);

    expect(worst(inLight, light)).toBeGreaterThanOrEqual(AA);
    expect(worst(inDark, dark)).toBeGreaterThanOrEqual(AA);
  });

  /// Losing the difference between the three greys costs information.
  it("keeps ink, soft and faint apart from each other", () => {
    const [ink] = token("ink");
    const [soft] = token("soft");
    const [faint] = token("faint");

    expect(lum(ink)).toBeLessThan(lum(soft));
    expect(lum(soft)).toBeLessThan(lum(faint));
  });
});
