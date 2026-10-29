import sheet from "./page.css?raw";
import tables from "./tables.css?raw";

export const HUES = [
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

export type Hue = (typeof HUES)[number];

export const isHue = (one: string): one is Hue => (HUES as readonly string[]).includes(one);

const PENS = ["yellow", "green", "blue", "pink"];

export const TOKENS = [
  "sheet",
  "ink",
  "soft",
  "faint",
  "accent",
  "accent-soft",
  "line",
  "hair",
  "hover",
  "rail",
  "desk",
  "cast",
  "mark-tag",
  "mark-doc",
  "mark-doc-lit",
  ...HUES.map((one) => `hue-${one}`),
  ...PENS.map((one) => `pen-${one}`),
].map((one) => `--tisty-${one}`);

const worn = (): string => {
  const theirs = getComputedStyle(document.documentElement);
  return TOKENS.map((token) => `${token}:${theirs.getPropertyValue(token).trim()}`)
    .filter((one) => !/[<>{}]/.test(one))
    .join(";");
};

export const escaped = (said: string): string =>
  said.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

// Everything a drawn page needs to look like itself, wherever it is shown.
export const PAGE_CSS = `${sheet}${tables}`;

export const shell = (html: string, dark: boolean, extra = ""): string =>
  `<!doctype html><html lang="${document.documentElement.lang}" class="${dark ? "dark" : ""}"><head><meta charset="utf-8"><style>:root{${worn()}}</style><style>${PAGE_CSS}${extra}</style></head><body><article class="page" inert>${html}</article></body></html>`;

export const dressed = (page: Document, dark: boolean) => {
  const theirs = getComputedStyle(document.documentElement);
  const root = page.documentElement;
  for (const token of TOKENS) root.style.setProperty(token, theirs.getPropertyValue(token));
  root.classList.toggle("dark", dark);
};

export const saidKinds = (page: Document | HTMLElement, said: (kind: string) => string) => {
  for (const one of page.querySelectorAll<HTMLElement>("blockquote[data-callout]"))
    one.dataset.said = said(one.dataset.callout ?? "note");
};
