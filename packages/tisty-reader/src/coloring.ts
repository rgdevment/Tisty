import { common, createLowlight } from "lowlight";

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
