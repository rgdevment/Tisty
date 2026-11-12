import { HUES } from "./hues";
import { IDEA_HUES } from "./ideas";
import sheet from "./page.css?raw";
import tables from "./tables.css?raw";

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
  ...IDEA_HUES.map((one) => `idea-${one}`),
  "idea-ink",
  ...[0, 1, 2, 3].map((one) => `idea-pin-${one}`),
  "idea-tape",
  "idea-needle",
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

// The blocks page.css draws as the document's title, for a host that names its window after it.
export const TITLED = [
  ".page:where(:not(.as-written)) > :is(h1, h2, h3, p):first-child",
  ".page:where(:not(.as-written)) > :is(.front, .fronted):first-child + :is(h1, h2, h3, p)",
].join(", ");

export const shell = (html: string, dark: boolean, extra = ""): string =>
  `<!doctype html><html lang="${document.documentElement.lang}" class="${dark ? "dark" : ""}"><head><meta charset="utf-8"><style>:root{${worn()}}</style><style>${PAGE_CSS}${extra}</style></head><body><article class="page" inert>${html}</article></body></html>`;

export const dressed = (page: Document, dark: boolean) => {
  const theirs = getComputedStyle(document.documentElement);
  const root = page.documentElement;
  for (const token of TOKENS) root.style.setProperty(token, theirs.getPropertyValue(token));
  root.classList.toggle("dark", dark);
};
