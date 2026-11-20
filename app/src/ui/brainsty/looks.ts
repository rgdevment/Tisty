import type { Neuron, NeuronKind, Tie } from "../../brainsty";

export type Shape = "square" | "circle";

export type Standing = "live" | "done" | "archived";

export interface Palette {
  bg: string;
  ink: string;
  soft: string;
  faint: string;
  accent: string;
  tag: string;
  file: string;
  hues: Record<string, string>;
}

export interface NodeLook {
  shape: Shape;
  size: number;
  color: string;
}

export interface LineLook {
  size: number;
  color: string;
  gap: number;
  tone: string;
  alpha: number;
}

export const AGENTS_HUE = "coral";

export const HUE_KEYS = [
  "red",
  "orange",
  "amber",
  "green",
  "teal",
  "blue",
  "indigo",
  "purple",
  "pink",
  "brown",
  "gray",
] as const;

const SIZE: Record<NeuronKind, number> = {
  doc: 7,
  page: 4.5,
  task: 5,
  tag: 2.5,
  file: 3,
};

const STANDING: Record<Standing, { alpha: number; scale: number }> = {
  live: { alpha: 1, scale: 1 },
  done: { alpha: 0.45, scale: 0.9 },
  archived: { alpha: 0.22, scale: 0.7 },
};

const LINE: Record<
  Tie,
  { size: number; alpha: number; gap: number; tone: "soft" | "tag" | "file" }
> = {
  cite: { size: 1.4, alpha: 0.7, gap: 0, tone: "soft" },
  mention: { size: 1.2, alpha: 0.65, gap: 4, tone: "soft" },
  structure: { size: 1, alpha: 0.6, gap: 0, tone: "soft" },
  tag: { size: 0.4, alpha: 0.15, gap: 0, tone: "tag" },
  uses: { size: 0.6, alpha: 0.3, gap: 0, tone: "file" },
};

const LIVE = new Set(["open", "said_done", "said_dropped"]);

export const DASH = 4;

export const standingOf = (neuron: Neuron): Standing => {
  if (!neuron.state || LIVE.has(neuron.state)) return "live";
  return neuron.state === "done" ? "done" : "archived";
};

export const shapeOf = (kind: NeuronKind): Shape =>
  kind === "doc" || kind === "page" ? "square" : "circle";

const alphaOf = (said?: string): number => {
  if (said === undefined) return 1;
  return said.endsWith("%") ? Number.parseFloat(said) / 100 : Number(said);
};

export function rgba(colour: string, alpha: number): string {
  const hex = /^#([0-9a-f]{6})$/i.exec(colour.trim());
  if (hex) {
    const n = Number.parseInt(hex[1], 16);
    return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${Math.round(alpha * 1000) / 1000})`;
  }
  const inside = /^rgba?\(([^)]*)\)$/i.exec(colour.trim());
  const parts = inside?.[1].split(/[\s,/]+/).filter(Boolean) ?? [];
  if (parts.length < 3) return colour;
  const [red, green, blue, own] = parts;
  return `rgba(${red}, ${green}, ${blue}, ${Math.round(alphaOf(own) * alpha * 1000) / 1000})`;
}

export function paletteFrom(read: (name: string) => string): Palette {
  const hues: Record<string, string> = {};
  for (const key of HUE_KEYS) hues[key] = read(`--tisty-hue-${key}`);
  // Agents' own hue, never one a folder can pick, so zones and choices keep the eleven.
  hues[AGENTS_HUE] = read(`--tisty-hue-${AGENTS_HUE}`);
  return {
    bg: read("--tisty-bg"),
    ink: read("--tisty-ink"),
    soft: read("--tisty-soft"),
    faint: read("--tisty-faint"),
    accent: read("--tisty-accent"),
    tag: hues.purple,
    file: hues.gray,
    hues,
  };
}

export const paletteNow = (): Palette => {
  const style = getComputedStyle(document.documentElement);
  return paletteFrom((name) => style.getPropertyValue(name).trim());
};

const TONE: Record<NeuronKind, keyof Omit<Palette, "hues">> = {
  doc: "soft",
  page: "soft",
  task: "accent",
  tag: "tag",
  file: "file",
};

const baseOf = (kind: NeuronKind, palette: Palette): string => palette[TONE[kind]];

export const weightOf = (degree: number): number => 1 + Math.min(2, 0.35 * Math.log2(1 + degree));

const ROOMY = 6000;

// Past the count the looks were drawn for, nodes shrink so the whole network covers the same share of the window.
export const crowding = (count: number): number =>
  Math.min(1, Math.sqrt(ROOMY / Math.max(count, 1)));

// What Tisty keeps in step with the agents' own files is theirs, so it stays small and quiet beside the person's.
export const OUTSIDE = { scale: 0.6, alpha: 0.5, lines: 0.4 } as const;

export const outsideTone = (palette: Palette): string => palette.hues[AGENTS_HUE] || palette.faint;

export function nodeLook(
  neuron: Neuron,
  hue: string | undefined,
  palette: Palette,
  degree = 0,
  outside = false,
): NodeLook {
  const standing = STANDING[standingOf(neuron)];
  const tinted = hue ? palette.hues[hue] : undefined;
  const size = SIZE[neuron.kind] * standing.scale * weightOf(degree);
  if (outside)
    return {
      shape: shapeOf(neuron.kind),
      size: size * OUTSIDE.scale,
      color: rgba(outsideTone(palette), standing.alpha * OUTSIDE.alpha),
    };
  return {
    shape: shapeOf(neuron.kind),
    size,
    color: rgba(
      tinted ?? baseOf(neuron.kind, palette),
      standing.alpha * belongingFade(neuron.kind),
    ),
  };
}

// A tag only says where something belongs, so it stays behind what was written.
export const belongingFade = (kind: NeuronKind): number => (kind === "tag" ? 0.6 : 1);

export function degreesOf(count: number, edges: readonly (readonly number[])[]): number[] {
  const degrees = new Array<number>(count).fill(0);
  for (const [from, to] of edges) {
    if (from < count) degrees[from]++;
    if (to < count) degrees[to]++;
  }
  return degrees;
}

// The few most tied nodes carry their titles at middle distance, so the map reads before every title shows.
export function bigOnes(degrees: readonly number[], share = 0.03, least = 3): boolean[] {
  const sorted = [...degrees].sort((one, other) => other - one);
  const bar = Math.max(least, sorted[Math.floor(sorted.length * share)] ?? least);
  return degrees.map((one) => one >= bar);
}

export function lineLook(tie: Tie, palette: Palette): LineLook {
  const line = LINE[tie];
  const tone = palette[line.tone];
  return { size: line.size, color: rgba(tone, line.alpha), gap: line.gap, tone, alpha: line.alpha };
}
