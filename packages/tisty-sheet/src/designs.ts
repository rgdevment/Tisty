import {
  FONTS,
  HEADINGS,
  INKS,
  type Own,
  type OwnDesign,
  ownIn,
  ownSaid,
  SHEETS,
} from "./ownDesign";

export const DESIGNS = ["tisty", "bosque", "coral", "fiordo", "violeta"] as const;

type Factory = (typeof DESIGNS)[number];

// A design of its own is said whole, as own/sheet/heading/ink/font, so it compares as any other name.
export type Design = Factory | OwnDesign;

export const isOwn = (design: Design): design is OwnDesign => design.startsWith("own/");

export const designNamed = (said: string | undefined): Design =>
  said !== undefined && /^own(\/[a-z_]*){0,4}$/.test(said)
    ? ownSaid(ownIn(said))
    : (DESIGNS.find((one) => one === said) ?? "tisty");

const COLORS = ["sheet", "ink", "soft", "line", "accent", "heading"] as const;

type Colors = Readonly<Record<(typeof COLORS)[number], string>>;

type Look = {
  readonly light: Colors;
  readonly dark: Colors;
  readonly title: "rule" | "bar";
};

// What the contract publishes as designs, and a test holds these to it. Tisty wears what style says, so it is not here.
export const LOOKS: Readonly<Record<Exclude<Factory, "tisty">, Look>> = {
  bosque: {
    light: {
      sheet: "#f3f8f2",
      ink: "#1d2b1f",
      soft: "#51654f",
      line: "rgb(40 100 50 / 0.28)",
      accent: "#2f7d3b",
      heading: "#2d5a34",
    },
    dark: {
      sheet: "#18211a",
      ink: "#e3ece2",
      soft: "#a3b5a2",
      line: "rgb(166 217 172 / 0.25)",
      accent: "#7fc98a",
      heading: "#a6d9ac",
    },
    title: "rule",
  },
  coral: {
    light: {
      sheet: "#fff7f1",
      ink: "#2d1c17",
      soft: "#7a5a4e",
      line: "rgb(190 80 40 / 0.28)",
      accent: "#b9441f",
      heading: "#b5391a",
    },
    dark: {
      sheet: "#261a16",
      ink: "#f5e6df",
      soft: "#c9a99c",
      line: "rgb(255 181 154 / 0.25)",
      accent: "#ff9a76",
      heading: "#ffb59a",
    },
    title: "bar",
  },
  fiordo: {
    light: {
      sheet: "#f4f7fa",
      ink: "#242a35",
      soft: "#4f5b6d",
      line: "rgb(46 80 110 / 0.28)",
      accent: "#2e6f86",
      heading: "#2f5d62",
    },
    dark: {
      sheet: "#2b303b",
      ink: "#e5e9f0",
      soft: "#aeb7c6",
      line: "rgb(216 222 233 / 0.28)",
      accent: "#88c0d0",
      heading: "#8fbcbb",
    },
    title: "rule",
  },
  violeta: {
    light: {
      sheet: "#f8f5fd",
      ink: "#2a2438",
      soft: "#5f5675",
      line: "rgb(124 77 204 / 0.28)",
      accent: "#7c4dcc",
      heading: "#9a3b86",
    },
    dark: {
      sheet: "#1e1e2e",
      ink: "#cdd6f4",
      soft: "#a6adc8",
      line: "rgb(205 214 244 / 0.25)",
      accent: "#cba6f7",
      heading: "#f5c2e7",
    },
    title: "bar",
  },
};

// What Tisty itself wears, for what has colors of its own on any design.
export const PLAIN: Readonly<Record<"light" | "dark", Omit<Colors, "heading">>> = {
  light: {
    sheet: "#ffffff",
    ink: "#1d1d1f",
    soft: "#57575c",
    line: "rgb(0 0 0 / 0.16)",
    accent: "#0060e3",
  },
  dark: {
    sheet: "#232326",
    ink: "#f2f2f7",
    soft: "#adadb4",
    line: "rgb(255 255 255 / 0.2)",
    accent: "#439bff",
  },
};

export const SIZE = "14.5px";

export const TINT = { light: 8, dark: 11 };

const EDGES = { rule: "1px solid var(--tisty-line)", bar: "4px solid var(--tisty-accent)" };

const softened = (accent: string, theme: "light" | "dark") =>
  `color-mix(in srgb, ${accent} ${TINT[theme]}%, transparent)`;

const toned = (own: Own, theme: "light" | "dark"): Colors => {
  const sheet = SHEETS[own.sheet][theme];
  const ink = INKS[own.ink][theme];
  const heading = HEADINGS[own.heading][theme];
  return {
    sheet,
    ink,
    heading,
    accent: heading,
    soft: `color-mix(in srgb, ${ink} 78%, ${sheet})`,
    line: `color-mix(in srgb, ${heading} 28%, transparent)`,
  };
};

const lookOf = (design: Exclude<Design, "tisty">): Look => {
  if (!isOwn(design)) return LOOKS[design];
  const own = ownIn(design);
  return { light: toned(own, "light"), dark: toned(own, "dark"), title: "rule" };
};

const TEXT =
  '-apple-system, BlinkMacSystemFont, "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif';

const fontOf = (design: Design) => {
  const face = isOwn(design) ? FONTS[ownIn(design).font] : undefined;
  return face ? `"${face.family}", ${face.falls}` : TEXT;
};

// The color that tells a design apart at a glance, in each theme.
export const glimpseOf = (design: Design) => {
  const { light, dark } = design === "tisty" ? PLAIN : lookOf(design);
  return { light: light.accent, dark: dark.accent };
};

// What a design puts on the root of a drawn page, over the tokens its host lent it.
export const wornBy = (design: Design, dark: boolean): [string, string][] => {
  if (design === "tisty") return [];
  const look = lookOf(design);
  const theme = dark ? "dark" : "light";
  const colors = look[theme];
  return [
    ...COLORS.map((one): [string, string] => [`--tisty-${one}`, colors[one]]),
    ["--tisty-accent-soft", softened(colors.accent, theme)],
    ...Object.entries(PLAIN[theme]).map(([one, value]): [string, string] => [
      `--own-${one}`,
      value,
    ]),
    ["--own-accent-soft", softened(PLAIN[theme].accent, theme)],
    ["--design-ink", colors.ink],
    ["--design-size", SIZE],
    ["--design-font", fontOf(design)],
    ["--own-font", TEXT],
    ["--design-edge", EDGES[look.title]],
  ];
};

export const wearDesign = (root: HTMLElement, design: Design, dark: boolean) => {
  for (const one of Array.from(root.style))
    if (/^--(design|own)-/.test(one) || one === "--tisty-heading") root.style.removeProperty(one);
  for (const [name, value] of wornBy(design, dark)) root.style.setProperty(name, value);
  root.toggleAttribute("data-designed", design !== "tisty");
};
