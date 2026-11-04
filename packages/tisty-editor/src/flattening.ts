const NS = "http://www.w3.org/2000/svg";

// What a drawing keeps. What else it holds, a stylesheet, a filter or an HTML label, says nothing a reader can use.
const KEPT = new Set([
  "svg",
  "g",
  "path",
  "rect",
  "circle",
  "ellipse",
  "line",
  "polygon",
  "polyline",
  "text",
  "tspan",
  "marker",
  "defs",
  "clippath",
  "lineargradient",
  "stop",
]);
const BECOME_GROUPS = new Set(["a", "switch"]);

const SHAPE: Record<string, readonly string[]> = {
  svg: ["viewBox", "width", "height", "preserveAspectRatio", "role", "aria-label"],
  path: ["d"],
  rect: ["x", "y", "width", "height", "rx", "ry"],
  circle: ["cx", "cy", "r"],
  ellipse: ["cx", "cy", "rx", "ry"],
  line: ["x1", "y1", "x2", "y2"],
  polygon: ["points"],
  polyline: ["points"],
  text: ["x", "y", "dx", "dy"],
  tspan: ["x", "y", "dx", "dy"],
  marker: ["id", "viewBox", "markerWidth", "markerHeight", "markerUnits", "refX", "refY", "orient"],
  clippath: ["id"],
  lineargradient: ["id", "x1", "y1", "x2", "y2", "gradientUnits"],
};

// What a child inherits needs saying only where it differs from its parent.
const INHERITED: Record<string, string> = {
  fill: "rgb(0, 0, 0)",
  stroke: "none",
  "stroke-width": "1px",
  "stroke-dasharray": "none",
  "stroke-linecap": "butt",
  "stroke-linejoin": "miter",
  "stroke-opacity": "1",
  "fill-opacity": "1",
  "fill-rule": "nonzero",
  "font-family": "",
  "font-size": "",
  "font-weight": "400",
  "font-style": "normal",
  "text-anchor": "start",
};
const ITS_OWN: Record<string, string> = { opacity: "1", "dominant-baseline": "auto" };
const NAMING = ["marker-start", "marker-end", "marker-mid", "clip-path"];
const UNPAINTED = new Set(["defs", "marker", "clippath", "lineargradient", "stop"]);
const WORDS = new Set(["svg", "g", "text", "tspan"]);
// What a shape would only ignore, so a line or a box never carries it.
const OF_WORDS = /^(font-|text-anchor|dominant-baseline)/;
const paints = (tag: string, name: string) =>
  (WORDS.has(tag) || !OF_WORDS.test(name)) && !(tag === "line" && name.startsWith("fill"));

const namedIn = (value: string): string | null => /#([^"')]+)/.exec(value)?.[1] ?? null;

type Copying = { readonly view: Window; readonly named: Set<string> };

const said = (one: Element, name: string, value: string | null) => {
  if (value !== null && value !== "") one.setAttribute(name, value);
};

const painted = (
  made: Element,
  tag: string,
  style: CSSStyleDeclaration,
  parent: CSSStyleDeclaration | null,
  copying: Copying,
) => {
  for (const [name, initial] of Object.entries(INHERITED)) {
    const value = style.getPropertyValue(name);
    if (
      paints(tag, name) &&
      value &&
      value !== (parent ? parent.getPropertyValue(name) : initial)
    ) {
      said(made, name, value);
      const named = value.includes("url(") ? namedIn(value) : null;
      if (named) {
        copying.named.add(named);
        made.setAttribute(name, `url(#${named})`);
      }
    }
  }
  for (const [name, initial] of Object.entries(ITS_OWN)) {
    const value = style.getPropertyValue(name);
    if (paints(tag, name) && value && value !== initial) said(made, name, value);
  }
  for (const name of NAMING) {
    const named = namedIn(style.getPropertyValue(name));
    if (named) {
      copying.named.add(named);
      made.setAttribute(name, `url(#${named})`);
    }
  }
};

const copied = (
  live: Element,
  parent: CSSStyleDeclaration | null,
  copying: Copying,
): Element | null => {
  const tag = live.tagName.toLowerCase();
  if (!KEPT.has(tag) && !BECOME_GROUPS.has(tag)) return null;
  const style = copying.view.getComputedStyle(live);
  if (style.display === "none") return null;
  const made = live.ownerDocument.createElementNS(NS, BECOME_GROUPS.has(tag) ? "g" : live.tagName);
  for (const name of SHAPE[tag] ?? []) said(made, name, live.getAttribute(name));
  if (tag === "stop") {
    said(made, "offset", live.getAttribute("offset"));
    said(
      made,
      "stop-color",
      style.getPropertyValue("stop-color") || live.getAttribute("stop-color"),
    );
    said(
      made,
      "stop-opacity",
      style.getPropertyValue("stop-opacity") || live.getAttribute("stop-opacity"),
    );
  }
  if (tag !== "svg") {
    const turned = style.transform && style.transform !== "none" ? style.transform : null;
    said(made, "transform", live.getAttribute("transform") ?? turned);
  }
  if (!UNPAINTED.has(tag)) painted(made, tag, style, parent, copying);
  for (const child of live.childNodes) {
    if (child.nodeType === 3) {
      if (tag === "text" || tag === "tspan") made.append(child.textContent ?? "");
    } else if (child.nodeType === 1) {
      const one = copied(child as Element, style, copying);
      if (one) made.append(one);
    }
  }
  return made;
};

// Two drawings on one page would share the names of their markers, and a line reaches the first it finds.
const renamed = (svg: Element, prefix: string) => {
  const names = new Map<string, string>();
  for (const one of svg.querySelectorAll("[id]")) {
    const fresh = `${prefix}-${names.size}`;
    names.set(one.id, fresh);
    one.id = fresh;
  }
  for (const one of [svg, ...svg.querySelectorAll("*")]) {
    for (const { name, value } of Array.from(one.attributes)) {
      const old = value.startsWith("url(#") ? value.slice(5, -1) : null;
      const fresh = old === null ? undefined : names.get(old);
      if (fresh) one.setAttribute(name, `url(#${fresh})`);
    }
  }
};

/** A drawing that needs nothing else: all it wore from a stylesheet is written on it, and no class, style or script is left. It has to be in the page, since only there does it have a style to read. */
export const flattened = (live: SVGSVGElement, prefix: string): string | null => {
  const view = live.ownerDocument.defaultView;
  if (!view) return null;
  const copying: Copying = { view, named: new Set() };
  const made = copied(live, null, copying);
  if (!made) return null;
  made.setAttribute("xmlns", NS);
  const box = live.viewBox?.baseVal;
  const wide = box?.width || Number.parseFloat(live.getAttribute("width") ?? "");
  const tall = box?.height || Number.parseFloat(live.getAttribute("height") ?? "");
  if (wide && tall) {
    made.setAttribute("width", String(Math.round(wide)));
    made.setAttribute("height", String(Math.round(tall)));
  }
  for (const one of made.querySelectorAll("marker, clipPath, linearGradient")) {
    if (!copying.named.has(one.id)) one.remove();
  }
  for (const one of made.querySelectorAll("defs")) if (!one.children.length) one.remove();
  renamed(made, prefix);
  return made.outerHTML;
};
