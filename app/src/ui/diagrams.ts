import type { MermaidConfig } from "mermaid";

type Mermaid = typeof import("mermaid")["default"];

const AT_MOST = 32 * 1024;
const KEPT_AT_MOST = 64;
// Narrower than this the words stop reading, so a wide drawing scrolls instead of shrinking further.
const LEAST = 0.6;
const PENDING = "figure.diagram > pre > code";

let loading: Promise<Mermaid> | null = null;
let turns: Promise<unknown> = Promise.resolve();
let made = 0;
const drawings = new Map<string, string | null>();
const sources = new WeakMap<Element, string>();

const keyOf = (source: string, dark: boolean) => `${dark ? "dark" : "light"}\n${source}`;

const kept = (key: string): string | null | undefined => {
  if (!drawings.has(key)) return undefined;
  const drawn = drawings.get(key) ?? null;
  drawings.delete(key);
  drawings.set(key, drawn);
  return drawn;
};

const keep = (key: string, drawn: string | null) => {
  drawings.set(key, drawn);
  if (drawings.size > KEPT_AT_MOST) drawings.delete(drawings.keys().next().value as string);
};

const settings = (dark: boolean): MermaidConfig => ({
  startOnLoad: false,
  securityLevel: "strict",
  suppressErrorRendering: true,
  maxTextSize: AT_MOST,
  theme: dark ? "dark" : "default",
  darkMode: dark,
  htmlLabels: false,
  fontFamily: getComputedStyle(document.body).fontFamily,
  flowchart: { htmlLabels: false },
});

// Mermaid keeps one configuration for the whole window, so drawings take turns.
export const drawing = (source: string, dark: boolean): Promise<string | null> => {
  const key = keyOf(source, dark);
  const ready = kept(key);
  if (ready !== undefined) return Promise.resolve(ready);
  if (!source.trim() || source.length > AT_MOST) return Promise.resolve(null);
  const turn = turns.then(async () => {
    const again = kept(key);
    if (again !== undefined) return again;
    loading ??= import("mermaid").then((one) => one.default);
    const mermaid = await loading;
    mermaid.initialize(settings(dark));
    made += 1;
    const drawn = await mermaid
      .render(`tisty-diagram-${made}`, source)
      .then((one) => one.svg)
      .catch(() => null);
    keep(key, drawn);
    return drawn;
  });
  turns = turn.catch(() => null);
  return turn;
};

const placed = (figure: Element, svg: string, dark: boolean) => {
  const page = figure.ownerDocument;
  const sheet = page.createElement("div");
  sheet.className = "diagram-sheet";
  sheet.innerHTML = svg;
  const drawn = sheet.querySelector("svg");
  const wide = drawn?.viewBox?.baseVal?.width;
  if (drawn && wide) drawn.style.setProperty("--wide", `${Math.round(wide * LEAST)}px`);
  figure.replaceChildren(sheet);
  figure.classList.add("drawn");
  figure.classList.remove("unreadable");
  figure.setAttribute("data-theme", dark ? "dark" : "light");
};

// Each diagram is drawn for the sheet it sits on; one that cannot be drawn stays as the words it is made of.
export const diagrams = (within: ParentNode, dark: boolean, live: () => boolean): Promise<void> => {
  const waiting: Promise<void>[] = [];
  const asked = new Set<Element>();
  for (const code of within.querySelectorAll(PENDING)) {
    const figure = code.closest("figure.diagram");
    if (figure) {
      sources.set(figure, code.textContent ?? "");
      asked.add(figure);
    }
  }
  for (const figure of within.querySelectorAll("figure.diagram.drawn")) asked.add(figure);
  for (const figure of asked) {
    const source = sources.get(figure);
    if (source === undefined || figure.getAttribute("data-theme") === (dark ? "dark" : "light"))
      continue;
    const ready = kept(keyOf(source, dark));
    if (typeof ready === "string") {
      placed(figure, ready, dark);
      continue;
    }
    waiting.push(
      drawing(source, dark).then((svg) => {
        if (!live() || !figure.isConnected) return;
        if (svg === null) figure.classList.add("unreadable");
        else placed(figure, svg, dark);
        // The viewer always shows a diagram on white, so a dark page has its light drawing ready too.
        if (svg !== null && dark) void drawing(source, false);
      }),
    );
  }
  return Promise.all(waiting).then(() => {});
};

// A diagram is looked at on white, as the light drawing it was made from.
export const diagramShot = (figure: Element): string | null => {
  const source = sources.get(figure);
  const light = source === undefined ? undefined : kept(keyOf(source, false));
  const svg = typeof light === "string" ? light : figure.querySelector("svg")?.outerHTML;
  return svg ? `data:image/svg+xml,${encodeURIComponent(svg)}` : null;
};
