import { common, createLowlight } from "lowlight";
import { DOC } from "../markdown";
import sheet from "./page.css?raw";
import type { Head } from "./writing";

const HUES = [
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
];
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

export const shell = (html: string): string =>
  `<!doctype html><html><head><meta charset="utf-8"><style>${sheet}</style></head><body><article class="page">${html}</article></body></html>`;

const darkNow = () => document.documentElement.getAttribute("data-theme") === "dark";

export const dressed = (page: Document) => {
  const theirs = getComputedStyle(document.documentElement);
  const root = page.documentElement;
  for (const token of TOKENS) root.style.setProperty(token, theirs.getPropertyValue(token));
  root.classList.toggle("dark", darkNow());
};

export type Went =
  | { kind: "doc"; id: string }
  | { kind: "web"; href: string }
  | { kind: "here"; id: string }
  | { kind: "attachment"; reference: string }
  | { kind: "block"; at: [number, number] }
  | { kind: "tag"; name: string }
  | null;

const spanned = (said: string | null): [number, number] | null => {
  const found = /^(\d+)-(\d+)$/.exec(said ?? "");
  return found ? [Number(found[1]), Number(found[2])] : null;
};

export const went = (target: Element, selecting: boolean): Went => {
  const link = target.closest("a");
  if (link) {
    const doc = link.getAttribute("data-doc");
    if (doc) return { kind: "doc", id: doc };
    const href = (link.getAttribute("href") ?? "").trim();
    if (href.startsWith(DOC) && href.length > DOC.length)
      return { kind: "doc", id: href.slice(DOC.length) };
    if (/^(https?:\/\/|mailto:)\S+$/i.test(href)) return { kind: "web", href };
    if (href.startsWith("#")) return href.length > 1 ? { kind: "here", id: href.slice(1) } : null;
    if (href && !/^[a-z][a-z0-9+.-]*:/i.test(href)) return { kind: "attachment", reference: href };
    return null;
  }
  const ref = target.closest(".ref")?.getAttribute("data-ref");
  if (ref) return null;
  const tag = target.closest(".tag")?.getAttribute("data-tag");
  if (tag) return { kind: "tag", name: tag };
  if (selecting || target.closest("summary, figure.widget")) return null;
  const block = target.closest(".page > [data-at]");
  const at = spanned(block?.getAttribute("data-at") ?? null);
  return at ? { kind: "block", at } : null;
};

export const heads = (page: Document): Head[] =>
  Array.from(page.querySelectorAll<HTMLElement>(".page > :is(h1, h2, h3, h4, h5, h6)")).map(
    (one, at) => ({
      key: `${at}:${one.id}`,
      level: Number(one.tagName.slice(1)),
      text: one.textContent ?? "",
      go: () => one.scrollIntoView({ block: "start" }),
    }),
  );

export const saidKinds = (page: Document, said: (kind: string) => string) => {
  for (const one of page.querySelectorAll<HTMLElement>("blockquote[data-callout]"))
    one.setAttribute("data-said", said(one.getAttribute("data-callout") ?? "note"));
};

const lowlight = createLowlight(common);
const BATCH = 40;

type Lit = {
  type: string;
  value?: string;
  properties?: { className?: unknown };
  children?: Lit[];
};

const grown = (page: Document, node: Lit): Node => {
  if (node.type === "text") return page.createTextNode(node.value ?? "");
  const span = page.createElement("span");
  const named = node.properties?.className;
  if (Array.isArray(named)) span.className = named.join(" ");
  for (const one of node.children ?? []) span.append(grown(page, one));
  return span;
};

const painted = (page: Document, code: HTMLElement) => {
  const tongue = /language-(\S+)/.exec(code.className)?.[1];
  if (!tongue || !lowlight.registered(tongue)) return;
  const tree = lowlight.highlight(tongue, code.textContent ?? "") as { children: Lit[] };
  const nodes = tree.children.map((one) => grown(page, one));
  code.replaceChildren(...nodes);
};

export const colored = (page: Document, live: () => boolean) => {
  const codes = Array.from(page.querySelectorAll<HTMLElement>("figure.code > pre > code[class]"));
  const step = (from: number) => {
    if (!live() || from >= codes.length) return;
    for (const code of codes.slice(from, from + BATCH)) painted(page, code);
    requestAnimationFrame(() => step(from + BATCH));
  };
  step(0);
};
