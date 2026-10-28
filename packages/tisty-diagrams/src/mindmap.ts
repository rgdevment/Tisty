import { headerAt } from "./framing";
export type Shape = "plain" | "square" | "rounded" | "circle" | "cloud" | "bang" | "hexagon";

export const SHAPES: readonly Shape[] = [
  "plain",
  "square",
  "rounded",
  "circle",
  "cloud",
  "bang",
  "hexagon",
];

export interface Idea {
  readonly text: string;
  readonly shape: Shape;
  readonly id?: string;
  readonly children: readonly Idea[];
  // The line and its indent as read, written back as is while the idea is untouched.
  readonly line?: string;
  readonly indent?: string;
  // Icons, classes, comments and blank lines under it, kept as written.
  readonly after: readonly string[];
}

export interface Mind {
  readonly head: readonly string[];
  readonly root: Idea;
  // Blank lines that end the text belong to it, not to the last idea, which can move.
  readonly tail: readonly string[];
}

const STEP = "  ";

const FORMS: Record<Exclude<Shape, "plain">, readonly [string, string]> = {
  circle: ["((", "))"],
  bang: ["))", "(("],
  hexagon: ["{{", "}}"],
  cloud: [")", "("],
  rounded: ["(", ")"],
  square: ["[", "]"],
};

// Mermaid also reads `(-cloud-)`; longest openings first, so `((` is never read as `(`.
const OPENINGS = [
  ...(Object.entries(FORMS) as [Shape, readonly [string, string]][]),
  ["cloud", ["(-", "-)"]] as [Shape, readonly [string, string]],
].sort(([, a], [, b]) => b[0].length - a[0].length);

const UNSAFE = /[()[\]{}]/;

const NAMED: Record<string, string> = { quot: '"', amp: "&", lt: "<", gt: ">" };

// Mermaid's own escapes, `#quot;` or `#35;`, read as the character they stand for.
const unescaped = (said: string): string =>
  said.replace(/#(\w+);/g, (whole, name: string) => {
    if (/^\d+$/.test(name)) return String.fromCodePoint(Number(name));
    return NAMED[name] ?? whole;
  });

const unquoted = (inner: string): string => {
  if (inner.startsWith('"`') && inner.endsWith('`"')) return inner.slice(2, -2);
  if (inner.length > 1 && inner.startsWith('"') && inner.endsWith('"'))
    return unescaped(inner.slice(1, -1));
  return unescaped(inner);
};

const shaped = (said: string): Pick<Idea, "text" | "shape" | "id"> | null => {
  for (const [shape, [open, close]] of OPENINGS) {
    const at = said.indexOf(open);
    if (at < 0 || !said.endsWith(close) || said.length < at + open.length + close.length) continue;
    const id = said.slice(0, at);
    if (UNSAFE.test(id)) continue;
    const text = unquoted(said.slice(at + open.length, said.length - close.length));
    return id ? { text, shape, id } : { text, shape };
  }
  return UNSAFE.test(said) ? null : { text: unescaped(said), shape: "plain" };
};

// Lines Mermaid reads as a comment or a decoration of the idea above, never as an idea.
const DECORATION = /^(%%|::icon\(|:::)/;

const kept = (line: string) => {
  const said = line.trim();
  return !said || DECORATION.test(said);
};

type Reading = Omit<Idea, "children" | "after"> & { children: Reading[]; after: string[] };

/** The map a `mindmap` diagram draws, or `null` when it holds something the window cannot keep. */
export const readMind = (source: string): Mind | null => {
  const lines = source.replace(/\r/g, "").split("\n");
  const opening = headerAt(lines, /^\s*mindmap\b/);
  if (opening < 0) return null;
  const head = lines.slice(0, opening + 1);
  let root: Reading | null = null;
  const open: Reading[] = [];
  for (const line of lines.slice(opening + 1)) {
    if (kept(line)) {
      if (open.length) open[open.length - 1].after.push(line);
      else head.push(line);
      continue;
    }
    const form = shaped(line.trim());
    if (!form) return null;
    const indent = line.slice(0, line.length - line.trimStart().length);
    const idea: Reading = { ...form, line: line.trim(), indent, children: [], after: [] };
    // As Mermaid does: the parent is the last idea written with less indent.
    while (open.length && (open[open.length - 1].indent ?? "").length >= indent.length) open.pop();
    const parent = open[open.length - 1];
    if (parent) parent.children.push(idea);
    else if (root) return null;
    else root = idea;
    open.push(idea);
  }
  if (!root) return null;
  const last = open[open.length - 1].after;
  let end = last.length;
  while (end > 0 && !last[end - 1].trim()) end -= 1;
  return { head, root, tail: last.splice(end) };
};

// Mermaid has no way to put brackets in an idea without a border, so such an idea gets a rounded one.
export const formed = (idea: Pick<Idea, "text" | "shape" | "id">): string => {
  const text = idea.text.replace(/"/g, "#quot;").replace(/\s*\n\s*/g, " ");
  const bare = text && text.trim() === text && !UNSAFE.test(text) && !DECORATION.test(text);
  if (idea.shape === "plain" && bare) return text;
  const [open, close] = FORMS[idea.shape === "plain" ? "rounded" : idea.shape];
  return `${idea.id ?? ""}${open}${bare ? text : `"${text}"`}${close}`;
};

// Children share one indent, the one they were written with while it still nests: a lone odd one would change parents.
const childIndent = (idea: Idea, indent: string): string => {
  const written = idea.children.find((one) => one.line !== undefined)?.indent;
  return written !== undefined && written.length > indent.length ? written : indent + STEP;
};

/** The text of a map; what was read and not touched comes back byte for byte. */
export const writeMind = (mind: Mind): string => {
  const out = [...mind.head];
  const emit = (idea: Idea, indent: string) => {
    out.push(`${indent}${idea.line ?? formed(idea)}`, ...idea.after);
    const below = childIndent(idea, indent);
    for (const child of idea.children) emit(child, below);
  };
  emit(mind.root, mind.root.indent ?? STEP);
  return [...out, ...mind.tail].join("\n");
};
