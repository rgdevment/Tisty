// What the contract publishes for a design of its own, and a test holds these to it.
type Tone = {
  readonly es: string;
  readonly en: string;
  readonly light: string;
  readonly dark: string;
};

export const SHEETS: Readonly<Record<string, Tone>> = {
  papel: { es: "Papel", en: "Paper", light: "#fbfaf7", dark: "#242321" },
  arena: { es: "Arena", en: "Sand", light: "#f8f2e6", dark: "#28241d" },
  durazno: { es: "Durazno", en: "Peach", light: "#fff3ec", dark: "#2a201c" },
  rosa: { es: "Rosa", en: "Rose", light: "#fdf1f4", dark: "#291f24" },
  lavanda: { es: "Lavanda", en: "Lavender", light: "#f5f2fc", dark: "#222030" },
  cielo: { es: "Cielo", en: "Sky", light: "#eff5fb", dark: "#1d2530" },
  menta: { es: "Menta", en: "Mint", light: "#eef7f2", dark: "#1b2622" },
  salvia: { es: "Salvia", en: "Sage", light: "#f3f6ec", dark: "#22261d" },
};

export const HEADINGS: Readonly<Record<string, Tone>> = {
  grafito: { es: "Grafito", en: "Graphite", light: "#3d4048", dark: "#c9ccd4" },
  vino: { es: "Vino", en: "Wine", light: "#8c2f45", dark: "#f2a3b4" },
  teja: { es: "Teja", en: "Brick", light: "#9a3f1c", dark: "#f5a885" },
  ocre: { es: "Ocre", en: "Ochre", light: "#785200", dark: "#e3bf72" },
  pino: { es: "Pino", en: "Pine", light: "#2f6438", dark: "#9fd3a6" },
  petroleo: { es: "Petróleo", en: "Teal", light: "#0f5f69", dark: "#86cdd6" },
  anil: { es: "Añil", en: "Indigo", light: "#2d52a0", dark: "#a3bff2" },
  ciruela: { es: "Ciruela", en: "Plum", light: "#70378c", dark: "#d8b0ec" },
};

export const INKS: Readonly<Record<string, Tone>> = {
  tinta: { es: "Tinta", en: "Ink", light: "#24262b", dark: "#e6e7ea" },
  cafe: { es: "Café", en: "Coffee", light: "#33261f", dark: "#ece1da" },
  noche: { es: "Noche", en: "Night", light: "#1f2a3d", dark: "#dfe6f2" },
  musgo: { es: "Musgo", en: "Moss", light: "#23301f", dark: "#e1eadd" },
  uva: { es: "Uva", en: "Grape", light: "#30233b", dark: "#e9e0f0" },
  humo: { es: "Humo", en: "Smoke", light: "#44474d", dark: "#cfd1d6" },
};

export type Face = { readonly name: string; readonly family: string; readonly falls: string };

export const SYSTEM = "sans";

export const FONTS: Readonly<Record<string, Face>> = {
  inter: { name: "Inter", family: "Inter Variable", falls: "sans-serif" },
  atkinson: {
    name: "Atkinson Hyperlegible",
    family: "Atkinson Hyperlegible Next Variable",
    falls: "sans-serif",
  },
  nunito: { name: "Nunito", family: "Nunito Variable", falls: "sans-serif" },
  grotesk: { name: "Space Grotesk", family: "Space Grotesk Variable", falls: "sans-serif" },
  source_serif: { name: "Source Serif", family: "Source Serif 4 Variable", falls: "serif" },
  lora: { name: "Lora", family: "Lora Variable", falls: "serif" },
  fraunces: { name: "Fraunces", family: "Fraunces Variable", falls: "serif" },
  caveat: { name: "Caveat", family: "Caveat Variable", falls: "cursive" },
};

export type Own = {
  readonly sheet: string;
  readonly heading: string;
  readonly ink: string;
  readonly font: string;
};

export const OWN: Own = { sheet: "papel", heading: "grafito", ink: "tinta", font: SYSTEM };

const named = (said: string | undefined, list: object, otherwise: string) =>
  said !== undefined && Object.keys(list).includes(said) ? said : otherwise;

// Each name is read alone, so one nobody knows leaves the others as they are.
export const ownIn = (said: string): Own => {
  const [, sheet, heading, ink, font] = said.split("/");
  return {
    sheet: named(sheet, SHEETS, OWN.sheet),
    heading: named(heading, HEADINGS, OWN.heading),
    ink: named(ink, INKS, OWN.ink),
    font: named(font, FONTS, OWN.font),
  };
};

export type OwnDesign = `own/${string}`;

export const ownSaid = ({ sheet, heading, ink, font }: Own): OwnDesign =>
  `own/${sheet}/${heading}/${ink}/${font}`;
