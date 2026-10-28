import { headerAt } from "./framing";
import type { Direction } from "./graphlayout";
import { type Place, placesApart, tidied } from "./places";

export interface Member {
  readonly text: string;
  readonly line: number;
  // Written as `Class : member` rather than inside the class's braces.
  readonly colon: boolean;
}

export interface ClassBox {
  readonly name: string;
  readonly members: readonly Member[];
  readonly decl: number | null;
  readonly close: number | null;
  readonly note: string | null;
  // Where `<<interface>>` and the like is said, inside the braces or on a line of its own.
  readonly noteLine: number | null;
  // The `namespace` its declaration stands in.
  readonly group: string | null;
}

/** A `namespace`: the classes declared between its opening and closing lines belong to it. */
export interface ClassGroup {
  readonly id: string;
  readonly open: number;
  readonly close: number;
}

export interface ClassLink {
  readonly left: string;
  readonly right: string;
  readonly arrow: string;
  readonly leftCard: string;
  readonly rightCard: string;
  readonly label: string;
  readonly line: number;
}

export interface Classes {
  readonly head: readonly string[];
  readonly body: readonly string[];
  readonly places: readonly Place[];
  readonly tail: readonly string[];
  readonly direction: Direction;
  readonly classes: readonly ClassBox[];
  readonly links: readonly ClassLink[];
  readonly groups: readonly ClassGroup[];
}

const NAME = String.raw`[\p{L}\p{N}_]+`;
// Longest first, so `<|--` is never read as `<--`.
export const ARROWS = [
  "<|--",
  "<|..",
  "--|>",
  "..|>",
  "*--",
  "--*",
  "o--",
  "--o",
  "-->",
  "<--",
  "..>",
  "<..",
  "--",
  "..",
];
const ARROW = ARROWS.map((one) => one.replace(/[|*.]/g, "\\$&")).join("|");
const LINK = new RegExp(
  String.raw`^\s*(${NAME})\s*(?:"([^"]*)"\s*)?(${ARROW})\s*(?:"([^"]*)"\s*)?(${NAME})\s*(?::\s*(.*?))?\s*$`,
  "u",
);
const DECL = new RegExp(
  String.raw`^\s*class\s+(${NAME})(~[^~]*~)?(\["[^"]*"\])?(:::[\w-]+)?\s*(\{)?\s*$`,
  "u",
);
const COLON = new RegExp(String.raw`^\s*(${NAME})\s*:\s*(.+?)\s*$`, "u");
const ANNOTATED = new RegExp(String.raw`^\s*<<\s*([^>]+?)\s*>>\s*(${NAME})\s*$`, "u");
const KEPT =
  /^\s*(%%|style\b|classDef\b|cssClass\b|link\b|click\b|callback\b|note\b|accTitle\b|accDescr\b|direction\b)/;

type Reading = {
  classes: Map<string, ClassBox>;
  links: ClassLink[];
  open: string | null;
  direction: Direction;
  space: { id: string; line: number } | null;
  groups: ClassGroup[];
};

const NAMESPACE = /^\s*namespace\s+([\p{L}\p{N}_.]+)\s*\{\s*$/u;

const classIn = (reading: Reading, name: string): ClassBox => {
  const known = reading.classes.get(name) ?? {
    name,
    members: [],
    decl: null,
    close: null,
    note: null,
    noteLine: null,
    group: null,
  };
  reading.classes.set(name, known);
  return known;
};

const updated = (reading: Reading, name: string, change: Partial<ClassBox>) =>
  reading.classes.set(name, { ...classIn(reading, name), ...change });

const insideBlock = (reading: Reading, said: string, line: number): boolean => {
  const name = reading.open as string;
  const trimmed = said.trim();
  if (trimmed === "}") {
    updated(reading, name, { close: line });
    reading.open = null;
  } else if (/^<<.+>>$/.test(trimmed))
    updated(reading, name, { note: trimmed.slice(2, -2).trim(), noteLine: line });
  else if (trimmed && !trimmed.startsWith("%%")) {
    const known = classIn(reading, name);
    updated(reading, name, { members: [...known.members, { text: trimmed, line, colon: false }] });
  }
  return true;
};

const declared = (reading: Reading, decl: RegExpExecArray, line: number): boolean => {
  // Mermaid joins two declarations of one class; the window would not know which one to write.
  if (classIn(reading, decl[1]).decl !== null) return false;
  updated(reading, decl[1], { decl: line, group: reading.space?.id ?? null });
  if (decl[5]) reading.open = decl[1];
  return true;
};

// Mermaid takes nothing but classes inside a namespace, and the window keeps it that way.
const insideSpace = (reading: Reading, said: string, line: number): boolean => {
  const space = reading.space as { id: string; line: number };
  const trimmed = said.trim();
  if (trimmed === "}") {
    reading.groups.push({ id: space.id, open: space.line, close: line });
    reading.space = null;
    return true;
  }
  if (!trimmed || trimmed.startsWith("%%")) return true;
  const decl = DECL.exec(said);
  return decl ? declared(reading, decl, line) : false;
};

const readLine = (reading: Reading, said: string, line: number): boolean => {
  if (reading.open) return insideBlock(reading, said, line);
  if (reading.space) return insideSpace(reading, said, line);
  const space = NAMESPACE.exec(said);
  if (space) {
    reading.space = { id: space[1], line };
    return true;
  }
  const direction = /^\s*direction\s+(TB|BT|LR|RL)\b/.exec(said);
  if (direction) reading.direction = direction[1] as Direction;
  if (!said.trim() || KEPT.test(said)) return true;
  const link = LINK.exec(said);
  if (link) {
    classIn(reading, link[1]);
    classIn(reading, link[5]);
    const [, left, leftCard = "", arrow, rightCard = "", right, label = ""] = link;
    reading.links.push({ left, right, arrow, leftCard, rightCard, label, line });
    return true;
  }
  const decl = DECL.exec(said);
  if (decl) return declared(reading, decl, line);
  const annotated = ANNOTATED.exec(said);
  if (annotated) {
    updated(reading, annotated[2], { note: annotated[1], noteLine: line });
    return true;
  }
  const colon = COLON.exec(said);
  if (!colon) return false;
  const known = classIn(reading, colon[1]);
  updated(reading, colon[1], {
    members: [...known.members, { text: colon[2], line, colon: true }],
  });
  return true;
};

/** The class diagram a diagram draws, or `null` when it says something the window could not keep as written. */
export const readClasses = (source: string): Classes | null => {
  const lines = source.replace(/\r/g, "").split("\n");
  const opening = headerAt(lines, /^\s*classDiagram(-v2)?\b/);
  if (opening < 0) return null;
  let end = lines.length;
  while (end > opening + 1 && !lines[end - 1].trim()) end -= 1;
  const { body, places } = placesApart(lines.slice(opening + 1, end));
  const reading: Reading = {
    classes: new Map(),
    links: [],
    open: null,
    direction: "TB",
    space: null,
    groups: [],
  };
  for (const [line, said] of body.entries()) if (!readLine(reading, said, line)) return null;
  if (reading.open || reading.space) return null;
  return {
    head: lines.slice(0, opening + 1),
    body,
    places: tidied(
      places,
      new Set([...reading.classes.keys(), ...reading.groups.map((one) => one.id)]),
    ),
    tail: lines.slice(end),
    direction: reading.direction,
    classes: [...reading.classes.values()],
    links: reading.links,
    groups: reading.groups,
  };
};

/** The text of a class diagram, its places last. */
export const writeClasses = (diagram: Pick<Classes, "head" | "body" | "places" | "tail">): string =>
  [
    ...diagram.head,
    ...diagram.body,
    ...diagram.places.map((one) => one.line),
    ...diagram.tail,
  ].join("\n");

export const linkLine = (indent: string, link: Omit<ClassLink, "line">): string => {
  const left = link.leftCard ? ` "${link.leftCard}"` : "";
  const right = link.rightCard ? `"${link.rightCard}" ` : "";
  const label = link.label ? ` : ${link.label}` : "";
  return `${indent}${link.left}${left} ${link.arrow} ${right}${link.right}${label}`;
};
