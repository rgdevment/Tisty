import type { MermaidConfig } from "mermaid";

type Mermaid = typeof import("mermaid")["default"];

const AT_MOST = 32 * 1024;
// Laying out runs on the window's own thread, and a short fence can still name thousands of links.
const MOST_PARTS = 1_000;
const KEPT_AT_MOST = 64;
const PENDING = "figure.diagram > pre > code";

let loading: Promise<Mermaid> | null = null;
let turns: Promise<unknown> = Promise.resolve();
let made = 0;
const drawings = new Map<string, string>();
// A fence half written while it is edited draws nothing, and keeping those beside the drawings would push them out.
const refused = new Set<string>();
const sources = new WeakMap<Element, string>();
const wanted = new WeakMap<Element, boolean>();

// What a figure was drawn from; before its drawing lands, the code the page carries.
export const sourceOf = (figure: Element): string =>
  sources.get(figure) ?? figure.querySelector("pre > code")?.textContent ?? "";

const keyOf = (source: string, dark: boolean) => `${dark ? "dark" : "light"}\n${source}`;

const kept = (key: string): string | null | undefined => {
  if (refused.has(key)) return null;
  const drawn = drawings.get(key);
  if (drawn === undefined) return undefined;
  drawings.delete(key);
  drawings.set(key, drawn);
  return drawn;
};

const capped = (held: Map<string, string> | Set<string>) => {
  if (held.size > KEPT_AT_MOST) held.delete(held.keys().next().value as string);
};

const keep = (key: string, drawn: string | null) => {
  if (drawn === null) {
    refused.add(key);
    capped(refused);
    return;
  }
  drawings.set(key, drawn);
  capped(drawings);
};

const parts = (source: string) =>
  source.split("\n").length + (source.match(/--|==|-\.|->/g)?.length ?? 0);

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

const mermaidNow = (): Promise<Mermaid> => {
  loading ??= import("mermaid")
    .then((one) => one.default)
    .catch((error: unknown) => {
      loading = null;
      throw error;
    });
  return loading;
};

// Mermaid keeps one configuration for the whole window, so drawings take turns.
export const drawing = (
  source: string,
  dark: boolean,
  still: () => boolean = () => true,
): Promise<string | null> => {
  const key = keyOf(source, dark);
  const ready = kept(key);
  if (ready !== undefined) return Promise.resolve(ready);
  if (!source.trim() || source.length > AT_MOST || parts(source) > MOST_PARTS)
    return Promise.resolve(null);
  const turn = turns.then(async () => {
    const again = kept(key);
    if (again !== undefined) return again;
    if (!still()) return null;
    const mermaid = await mermaidNow().catch(() => null);
    if (!mermaid) return null;
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
  return turn.catch(() => null);
};

const placed = (figure: Element, svg: string, dark: boolean) => {
  const sheet = figure.ownerDocument.createElement("div");
  sheet.className = "diagram-sheet";
  sheet.innerHTML = svg;
  figure.replaceChildren(sheet, ...figure.querySelectorAll(":scope > .diagram-bar"));
  figure.classList.add("drawn");
  figure.classList.remove("unreadable");
  figure.setAttribute("data-theme", dark ? "dark" : "light");
};

export const diagrams = (within: ParentNode, dark: boolean, live: () => boolean): Promise<void> => {
  const waiting: Promise<void>[] = [];
  const asked = new Set<Element>();
  for (const code of within.querySelectorAll(PENDING)) {
    const figure = code.closest("figure.diagram");
    if (!figure) continue;
    sources.set(figure, code.textContent ?? "");
    asked.add(figure);
  }
  for (const figure of within.querySelectorAll("figure.diagram.drawn")) asked.add(figure);
  for (const figure of asked) {
    const source = sources.get(figure);
    if (source === undefined) continue;
    wanted.set(figure, dark);
    if (figure.getAttribute("data-theme") === (dark ? "dark" : "light")) continue;
    const ready = kept(keyOf(source, dark));
    if (typeof ready === "string") {
      placed(figure, ready, dark);
      continue;
    }
    // Only the look last asked for lands, so a slow drawing never paints over a newer one.
    const still = () => live() && figure.isConnected && wanted.get(figure) === dark;
    waiting.push(
      drawing(source, dark, still).then((svg) => {
        if (!still()) return;
        if (svg === null) figure.classList.add("unreadable");
        else placed(figure, svg, dark);
      }),
    );
  }
  return Promise.all(waiting).then(() => {});
};

// The viewer lays a picture on white, so a dark drawing brings the dark sheet it was drawn for.
export const diagramShot = (figure: Element): string | null => {
  const svg = figure.querySelector("svg");
  if (!svg) return null;
  if (figure.getAttribute("data-theme") !== "dark")
    return `data:image/svg+xml,${encodeURIComponent(svg.outerHTML)}`;
  const box = svg.viewBox?.baseVal;
  const [wide, tall] = box?.width ? [box.width, box.height] : [800, 600];
  const ground = getComputedStyle(figure).getPropertyValue("--tisty-sheet").trim() || "#232326";
  const framed = `<svg xmlns="http://www.w3.org/2000/svg" width="${wide}" height="${tall}" viewBox="0 0 ${wide} ${tall}"><rect width="100%" height="100%" fill="${ground}"/>${svg.outerHTML}</svg>`;
  return `data:image/svg+xml,${encodeURIComponent(framed)}`;
};
