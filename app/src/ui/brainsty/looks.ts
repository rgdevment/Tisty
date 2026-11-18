import type { Neuron, NeuronKind, Tie } from "../../brainsty";

export type Shape = "square" | "circle";

export type Standing = "live" | "done" | "archived";

export interface Palette {
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
}

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

const SIZE: Record<NeuronKind, number> = { doc: 7, page: 4.5, task: 5, tag: 2.5, file: 3 };

const STANDING: Record<Standing, { alpha: number; scale: number }> = {
  live: { alpha: 1, scale: 1 },
  done: { alpha: 0.45, scale: 0.9 },
  archived: { alpha: 0.22, scale: 0.7 },
};

const LINE: Record<
  Tie,
  { size: number; alpha: number; gap: number; tone: "soft" | "tag" | "file" }
> = {
  cite: { size: 1.2, alpha: 0.55, gap: 0, tone: "soft" },
  mention: { size: 1, alpha: 0.5, gap: 4, tone: "soft" },
  structure: { size: 0.7, alpha: 0.4, gap: 0, tone: "soft" },
  tag: { size: 0.5, alpha: 0.22, gap: 0, tone: "tag" },
  uses: { size: 0.6, alpha: 0.35, gap: 0, tone: "file" },
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
    return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
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
  return {
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

export function nodeLook(neuron: Neuron, hue: string | undefined, palette: Palette): NodeLook {
  const standing = STANDING[standingOf(neuron)];
  const tinted = hue ? palette.hues[hue] : undefined;
  return {
    shape: shapeOf(neuron.kind),
    size: SIZE[neuron.kind] * standing.scale,
    color: rgba(tinted ?? baseOf(neuron.kind, palette), standing.alpha),
  };
}

export function lineLook(tie: Tie, palette: Palette): LineLook {
  const line = LINE[tie];
  return { size: line.size, color: rgba(palette[line.tone], line.alpha), gap: line.gap };
}
