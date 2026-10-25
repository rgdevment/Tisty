import { common, createLowlight } from "lowlight";
import { t } from "../locales";
import { DOC, TASK } from "../markdown";
import bar from "./bar.css?raw";
import { diagramShot } from "./diagrams";
import type { Choice } from "./Menu";
import sheet from "./page.css?raw";
import task from "./taskpage.css?raw";

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

const darkNow = () => document.documentElement.dataset.theme === "dark";

const worn = (): string => {
  const theirs = getComputedStyle(document.documentElement);
  return TOKENS.map((token) => `${token}:${theirs.getPropertyValue(token).trim()}`)
    .filter((one) => !/[<>{}]/.test(one))
    .join(";");
};

export const escaped = (said: string): string =>
  said.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

export const shell = (html: string): string =>
  `<!doctype html><html lang="${document.documentElement.lang}" class="${darkNow() ? "dark" : ""}"><head><meta charset="utf-8"><style>:root{${worn()}}</style><style>${sheet}${bar}${task}</style></head><body><article class="page" inert>${html}</article></body></html>`;

export const dressed = (page: Document) => {
  const theirs = getComputedStyle(document.documentElement);
  const root = page.documentElement;
  for (const token of TOKENS) root.style.setProperty(token, theirs.getPropertyValue(token));
  root.classList.toggle("dark", darkNow());
};

// The page names blocks by their bytes in the file; whoever edits holds the text as characters.
export const charsAt = (body: string, bytes: number): number =>
  new TextDecoder().decode(new TextEncoder().encode(body).subarray(0, bytes)).length;

export type Went =
  | { kind: "doc"; id: string }
  | { kind: "task"; id: string }
  | { kind: "web"; href: string }
  | { kind: "here"; id: string }
  | { kind: "attachment"; reference: string }
  | { kind: "block"; at: [number, number] }
  | { kind: "tag"; name: string }
  | { kind: "tick"; at: [number, number] }
  | { kind: "part"; at: [number, number] }
  | { kind: "apart"; at: [number, number]; nth?: number }
  | { kind: "more"; at: [number, number]; target: string; x: number; y: number }
  | { kind: "end" }
  | { kind: "title" }
  | { kind: "hint"; at: [number, number]; seed: string; from: "hint" | "add-step" | "add-section" }
  | { kind: "section"; at: [number, number] }
  | { kind: "cut"; at: [number, number] }
  | { kind: "picture"; nth: number }
  | null;

// A picture or a diagram, as the viewer shows it; only a kept file can be saved or opened.
export type Shot = { readonly src: string; readonly kept: string | null; readonly alt: string };

// What a click leaves alone, and so no picture inside it is one to look at.
const UNTOUCHED = "summary, figure.widget, .media, .col-grip, .picture-away";

const SHOTS = ".page img[src], .page figure.diagram.drawn";

const shotsIn = (page: Document): Element[] =>
  Array.from(page.querySelectorAll(SHOTS)).filter(
    (one) => !one.closest(`.doc-card, details:not([open]), ${UNTOUCHED}`),
  );

export const shotAt = (page: Document, target: Element | null): number | null => {
  const shot = target?.closest(SHOTS);
  const nth = shot ? shotsIn(page).indexOf(shot) : -1;
  return nth >= 0 ? nth : null;
};

export type PictureWays = {
  readonly look: (nth: number) => void;
  readonly save: (kept: string) => void;
  readonly open: (kept: string) => void;
};

// What a picture's own menu offers above writing it: a kept file can also leave or open.
export const pictureChoices = (shots: Shot[], nth: number | null, ways: PictureWays): Choice[] => {
  const kept = nth === null ? null : (shots[nth]?.kept ?? null);
  if (nth === null || !shots[nth]) return [];
  const look: Choice = { key: "look", label: t("pictureLook"), onPick: () => ways.look(nth) };
  if (!kept) return [look];
  return [
    look,
    { key: "save", label: t("pictureSave"), onPick: () => ways.save(kept) },
    { key: "open", label: t("pictureOpen"), onPick: () => ways.open(kept) },
  ];
};

export const shotsOf = (page: Document): Shot[] =>
  shotsIn(page).map((one) => {
    if (one.tagName !== "IMG") return { src: diagramShot(one) ?? "", kept: null, alt: "" };
    const img = one as HTMLImageElement;
    return { src: img.currentSrc || img.src, kept: img.dataset.kept ?? null, alt: img.alt };
  });

// What a label holds runs to the next label, so writing under it starts past the last of it.
const sectionEnd = (label: Element): [number, number] | null => {
  let last = spanned(label.getAttribute("data-at"));
  for (let one = label.nextElementSibling; one; one = one.nextElementSibling) {
    if (/^H[1-6]$/.test(one.tagName)) break;
    last = spanned(one.getAttribute("data-at")) ?? last;
  }
  return last;
};

export const spanned = (said: string | null): [number, number] | null => {
  const found = /^(\d+)-(\d+)$/.exec(said ?? "");
  return found ? [Number(found[1]), Number(found[2])] : null;
};

const linked = (link: HTMLAnchorElement): Went => {
  const doc = link.dataset.doc;
  if (doc) return { kind: "doc", id: doc };
  const task = link.dataset.task;
  if (task) return { kind: "task", id: task };
  const href = (link.getAttribute("href") ?? "").trim();
  if (href.startsWith(DOC) && href.length > DOC.length)
    return { kind: "doc", id: href.slice(DOC.length) };
  if (href.startsWith(TASK) && href.length > TASK.length)
    return { kind: "task", id: href.slice(TASK.length) };
  if (/^(https?:\/\/|mailto:)\S+$/i.test(href)) return { kind: "web", href };
  if (href.startsWith("#")) return href.length > 1 ? { kind: "here", id: href.slice(1) } : null;
  if (href && !/^[a-z][a-z0-9+.-]*:/i.test(href)) return { kind: "attachment", reference: href };
  return null;
};

export const widgetsIn = (block: Element | null): Element[] =>
  block?.matches("figure.widget")
    ? [block]
    : Array.from(block?.querySelectorAll("figure.widget") ?? []);

const pressedIn = (asked: HTMLElement): Went => {
  if (asked.dataset.asked === "part") {
    const step = spanned(asked.closest<HTMLElement>("li.task[data-at]")?.dataset.at ?? null);
    return step ? { kind: "part", at: step } : null;
  }
  const at = spanned(asked.closest<HTMLElement>(".page > [data-at]")?.dataset.at ?? null);
  if (!at) return null;
  if (asked.dataset.asked === "more") {
    const box = asked.getBoundingClientRect();
    return { kind: "more", at, target: asked.dataset.target ?? "", x: box.left, y: box.bottom };
  }
  if (asked.dataset.asked !== "apart") return { kind: "block", at };
  const nth = widgetsIn(asked.closest(".page > [data-at]")).indexOf(
    asked.closest("figure.widget") as Element,
  );
  return nth > 0 ? { kind: "apart", at, nth } : { kind: "apart", at };
};

export const went = (target: Element, selecting: boolean): Went => {
  const asked = target.closest<HTMLElement>("button[data-asked]");
  if (asked) return pressedIn(asked);
  const link = target.closest("a");
  if (link) return linked(link);
  const card = target.closest<HTMLElement>(".doc-card[data-doc]")?.dataset.doc;
  if (card) return { kind: "doc", id: card };
  const part = target.closest<HTMLElement>(".doc-card[data-task]")?.dataset.task;
  if (part) return { kind: "task", id: part };
  const ref = target.closest<HTMLElement>(".ref");
  if (ref?.dataset.stage) return { kind: "here", id: ref.dataset.stage };
  if (ref?.dataset.ref) return null;
  const tag = target.closest<HTMLElement>(".tag")?.dataset.tag;
  if (tag) return { kind: "tag", name: tag };
  if (selecting || target.closest(UNTOUCHED)) return null;
  // A picture or a diagram is looked at; it is written only from the block's own menu.
  const shot = shotAt(target.ownerDocument, target);
  if (shot !== null) return { kind: "picture", nth: shot };
  // A stage's box ticks where it is read; the words beside it still open the block.
  const box = target.closest(".check")?.closest<HTMLElement>("li.task[data-at]");
  const ticked = spanned(box?.dataset.at ?? null);
  if (ticked) return { kind: "tick", at: ticked };
  if (target.closest(".page > h1[data-title]")) return { kind: "title" };
  const look = target.ownerDocument.documentElement.classList;
  const shaping = look.contains("untitled") && look.contains("shaping");
  const drop = spanned(target.closest<HTMLElement>(".drop-section")?.dataset.at ?? null);
  if (drop) return shaping ? { kind: "cut", at: drop } : null;
  const hinted = target.closest<HTMLElement>(
    ".page > :is(.hint, .hint-gone, .add-step, .add-section)",
  );
  const hint = spanned(hinted?.dataset.at ?? hinted?.dataset.after ?? null);
  // While the template is edited, a hint is its words to mend, not a place to write.
  if (hint && shaping && hinted?.matches(".hint, .hint-gone")) return { kind: "block", at: hint };
  if (shaping && hinted?.matches(".add-step")) return null;
  if (hint) {
    const from = hinted?.classList.contains("add-section")
      ? "add-section"
      : hinted?.classList.contains("add-step")
        ? "add-step"
        : "hint";
    return { kind: "hint", at: hint, seed: hinted?.dataset.seed ?? "", from };
  }
  // A task's sections are filled in, not renamed, until its structure is asked for; an empty one is still mended.
  const label = target.closest(".page > :is(h1, h2, h3, h4, h5, h6)");
  if (look.contains("untitled") && !look.contains("shaping") && label?.textContent?.trim()) {
    const end = sectionEnd(label);
    return end ? { kind: "section", at: end } : null;
  }
  if (shaping && !label) return null;
  const at = spanned(target.closest<HTMLElement>(".page > [data-at]")?.dataset.at ?? null);
  return at ? { kind: "block", at } : null;
};

export interface Head {
  key: string;
  level: number;
  text: string;
  go: () => void;
}

// Level 0 is no heading: it opens the properties, which the page keeps out of sight.
export const heads = (page: Document): Head[] => {
  const front = page.querySelector<HTMLElement>(".page > .front:first-child");
  const opening: Head[] = front
    ? [{ key: "front", level: 0, text: "", go: () => front.click() }]
    : [];
  return opening.concat(listedHeads(page));
};

const listedHeads = (page: Document): Head[] =>
  Array.from(
    page.querySelectorAll<HTMLElement>(".page > :is(h1, h2, h3, h4, h5, h6):not([data-title])"),
  ).map((one, at) => ({
    key: `${at}:${one.id}`,
    level: Number(one.tagName.slice(1)),
    text: one.textContent ?? "",
    go: () => one.scrollIntoView({ block: "start" }),
  }));

export const saidKinds = (page: Document | HTMLElement, said: (kind: string) => string) => {
  for (const one of page.querySelectorAll<HTMLElement>("blockquote[data-callout]"))
    one.dataset.said = said(one.dataset.callout ?? "note");
};

const lowlight = createLowlight(common);
const FRAME_MS = 12;
const PAINTED_AT_MOST = 64 * 1024;

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
  const said = code.textContent ?? "";
  if (!tongue || !lowlight.registered(tongue) || said.length > PAINTED_AT_MOST) return;
  const tree = lowlight.highlight(tongue, said) as { children: Lit[] };
  const nodes = tree.children.map((one) => grown(page, one));
  code.replaceChildren(...nodes);
};

export const coloredAll = (page: Document) => {
  for (const code of page.querySelectorAll<HTMLElement>("figure.code > pre > code[class]"))
    painted(page, code);
};

export const colored = (page: Document, live: () => boolean) => {
  const codes = Array.from(page.querySelectorAll<HTMLElement>("figure.code > pre > code[class]"));
  const step = (from: number) => {
    const until = performance.now() + FRAME_MS;
    let at = from;
    while (live() && at < codes.length && performance.now() < until) {
      painted(page, codes[at]);
      at += 1;
    }
    if (live() && at < codes.length) requestAnimationFrame(() => step(at));
  };
  step(0);
};
