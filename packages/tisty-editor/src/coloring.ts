type Lowlight = ReturnType<typeof import("lowlight").createLowlight>;

let engine: Promise<Lowlight> | undefined;
// The grammars weigh more than the rest of what draws a page, so they come only for a page with code to colour.
const lowlit = (): Promise<Lowlight> =>
  (engine ??= import("lowlight").then(({ common, createLowlight }) => createLowlight(common)));

const CODES = "figure.code > pre > code[class]";
const FRAME_MS = 12;
const PAINTED_AT_MOST = 64 * 1024;

type Lit = {
  type: string;
  value?: string;
  properties?: { className?: unknown };
  children?: Lit[];
};

// A part in another language is named after it too, which colours nothing and says nothing to whoever reads.
const colouring = (one: unknown) => typeof one === "string" && (one.startsWith("hljs-") || one.endsWith("_"));

const grown = (page: Document, node: Lit): Node => {
  if (node.type === "text") return page.createTextNode(node.value ?? "");
  const span = page.createElement("span");
  const named = node.properties?.className;
  const worn = Array.isArray(named) ? named.filter(colouring) : [];
  if (worn.length) span.className = worn.join(" ");
  for (const one of node.children ?? []) span.append(grown(page, one));
  return span;
};

const painted = (lowlight: Lowlight, page: Document, code: HTMLElement) => {
  const tongue = /language-(\S+)/.exec(code.className)?.[1];
  const said = code.textContent ?? "";
  if (!tongue || !lowlight.registered(tongue) || said.length > PAINTED_AT_MOST) return;
  const tree = lowlight.highlight(tongue, said) as { children: Lit[] };
  const nodes = tree.children.map((one) => grown(page, one));
  code.replaceChildren(...nodes);
};

export const coloredAll = async (page: Document) => {
  const codes = page.querySelectorAll<HTMLElement>(CODES);
  if (codes.length === 0) return;
  const lowlight = await lowlit();
  for (const code of codes) painted(lowlight, page, code);
};

export const colored = async (page: Document, live: () => boolean) => {
  const codes = Array.from(page.querySelectorAll<HTMLElement>(CODES));
  if (codes.length === 0) return;
  const lowlight = await lowlit();
  const step = (from: number) => {
    const until = performance.now() + FRAME_MS;
    let at = from;
    while (live() && at < codes.length && performance.now() < until) {
      painted(lowlight, page, codes[at]);
      at += 1;
    }
    if (live() && at < codes.length) requestAnimationFrame(() => step(at));
  };
  step(0);
};
