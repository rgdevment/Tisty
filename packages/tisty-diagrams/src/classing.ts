import { type Classes, type ClassLink, linkLine, readClasses, writeClasses } from "./classes";
import { renamedLine } from "./naming";
import { type Place, placed, type Spot } from "./places";

/** What is chosen: a class by its name, a member by its place in it, a relation by its place among them. */
export type ClassPicked =
  | { readonly kind: "class"; readonly name: string }
  | { readonly kind: "member"; readonly name: string; readonly index: number }
  | { readonly kind: "link"; readonly index: number }
  | { readonly kind: "classes"; readonly names: readonly string[] }
  | { readonly kind: "namespace"; readonly name: string }
  | null;

/** A change is new text for the body of the diagram, read again by whoever shows it. */
export interface ClassAltered {
  readonly body: readonly string[];
  // Left out when the places stay as they were.
  readonly places?: readonly Place[];
  readonly picked: ClassPicked;
}

/** The diagram a change leaves, read again so the next change finds its lines where they now are. */
export const classReread = (diagram: Classes, made: ClassAltered): Classes =>
  readClasses(
    writeClasses({ ...diagram, body: made.body, places: made.places ?? diagram.places }),
  ) ?? diagram;

/** Mermaid takes no empty `namespace`: one a change leaves without classes goes with it. */
export const spacesKept = (diagram: Classes, made: ClassAltered): ClassAltered => {
  const now = classReread(diagram, made);
  const empty = now.groups.filter((group) => !now.classes.some((one) => one.group === group.id));
  if (!empty.length) return made;
  const gone = new Set(empty.flatMap((group) => [group.open, group.close]));
  return { ...made, body: now.body.filter((_, at) => !gone.has(at)), places: now.places };
};

export const indentOf = (line: string | undefined) =>
  line?.slice(0, line.length - line.trimStart().length) ?? "";
export const indentIn = (diagram: Classes) =>
  indentOf(diagram.body.find((one) => one.trim())) || "  ";
const classOf = (diagram: Classes, name: string) =>
  diagram.classes.find((one) => one.name === name);
export const plain = (said: string) => said.trim().replace(/\s+/g, "_");

const placedIn = (diagram: Classes, name: string, at: Spot) =>
  placed(diagram.places, indentIn(diagram), name, at);

/** A new class, of a kind such as `interface` when one is given, where it was put when it was put somewhere. */
export const classAdded = (
  diagram: Classes,
  wanted: string,
  note: string | null = null,
  at: Spot | null = null,
): ClassAltered => {
  const taken = diagram.classes.map((one) => one.name);
  let n = 1;
  while (taken.includes(`${wanted}${n}`)) n += 1;
  const name = `${wanted}${n}`;
  const indent = indentIn(diagram);
  return {
    body: [
      ...diagram.body,
      `${indent}class ${name}`,
      ...(note ? [`${indent}<<${note}>> ${name}`] : []),
    ],
    places: at ? placedIn(diagram, name, at) : diagram.places,
    picked: { kind: "class", name },
  };
};

/** Classes put where the window was told, without touching the lines that say what they are. */
export const classesPlaced = (
  diagram: Classes,
  moves: readonly { name: string; x: number; y: number }[],
  picked: ClassPicked,
): ClassAltered => ({
  body: diagram.body,
  places: moves.reduce<readonly Place[]>(
    (places, move) => placed(places, indentIn(diagram), move.name, move),
    diagram.places,
  ),
  picked,
});

/** The diagram with every class it laid out itself written where it stands, so nothing jumps when one moves. */
export const classesPinned = (
  diagram: Classes,
  at: ReadonlyMap<string, { readonly x: number; readonly y: number }>,
): Classes => {
  const placedOnes = new Set(diagram.places.map((one) => one.id));
  const moves = diagram.classes.flatMap((one) => {
    const point = at.get(one.name);
    return point && !placedOnes.has(one.name) ? [{ name: one.name, x: point.x, y: point.y }] : [];
  });
  return moves.length
    ? { ...diagram, places: classesPlaced(diagram, moves, null).places ?? diagram.places }
    : diagram;
};

/** A class made an interface, an abstract class or an enumeration, or a plain class again. */
export const classKindChanged = (
  diagram: Classes,
  name: string,
  note: string | null,
): ClassAltered => {
  const own = classOf(diagram, name);
  const picked: ClassPicked = { kind: "class", name };
  if (!own || own.note === note) return { body: diagram.body, picked };
  if (own.noteLine !== null) {
    const line = diagram.body[own.noteLine];
    const inside = own.decl !== null && own.noteLine > own.decl && own.noteLine < (own.close ?? 0);
    const said = inside ? `${indentOf(line)}<<${note}>>` : `${indentOf(line)}<<${note}>> ${name}`;
    return {
      body: note
        ? diagram.body.map((one, at) => (at === own.noteLine ? said : one))
        : diagram.body.filter((_, at) => at !== own.noteLine),
      picked,
    };
  }
  // Mermaid takes nothing but classes inside a namespace, so the kind is said after it.
  const space = diagram.groups.find((one) => one.id === own.group);
  const at = space?.close ?? own.close ?? own.decl ?? diagram.body.length - 1;
  return {
    body: [
      ...diagram.body.slice(0, at + 1),
      `${space ? indentOf(diagram.body[space.open]) : indentOf(diagram.body[at]) || indentIn(diagram)}<<${note}>> ${name}`,
      ...diagram.body.slice(at + 1),
    ],
    picked,
  };
};

export const VISIBILITIES = ["+", "-", "#", "~"] as const;

/** The mark a member's visibility is written with, `+`, `-`, `#` or `~`, or none. */
export const visibilityOf = (text: string): string => (/^[+\-#~]/.test(text) ? text[0] : "");

export const withVisibility = (text: string, mark: string): string =>
  `${mark}${text.replace(/^[+\-#~]\s*/, "")}`;

/** A class renamed everywhere it stands as a word of its own: declaration, relations, members, styles. */
export const classRenamed = (diagram: Classes, from: string, wanted: string): ClassAltered => {
  const to = plain(wanted);
  const taken = diagram.classes.some((one) => one.name === to);
  if (!/^[\p{L}\p{N}_]+$/u.test(to) || to === from || taken)
    return { body: diagram.body, picked: { kind: "class", name: from } };
  return {
    // A package of the same name is not the class; its opening line stays as it is.
    body: diagram.body.map((line, at) =>
      diagram.groups.some((one) => one.open === at) ? line : renamedLine(line, from, to),
    ),
    // Its place goes with it, or the class would be laid out anew under its new name.
    places: diagram.places.map((one) =>
      one.id === from
        ? { ...one, id: to, line: one.line.replace(`tisty: ${from} `, `tisty: ${to} `) }
        : one,
    ),
    picked: { kind: "class", name: to },
  };
};

// A class goes with its declaration and members, its relations, and the lines that style or note it alone.
const goesWith = (diagram: Classes, name: string) => {
  const own = classOf(diagram, name);
  const lines = new Set<number>();
  if (own?.decl !== null && own?.decl !== undefined)
    for (let at = own.decl; at <= (own.close ?? own.decl); at += 1) lines.add(at);
  for (const member of own?.members ?? []) lines.add(member.line);
  for (const link of diagram.links)
    if (link.left === name || link.right === name) lines.add(link.line);
  diagram.body.forEach((line, at) => {
    if (
      new RegExp(
        `^\\s*(<<.+>>\\s*${name}|(style|click|link|callback)\\s+${name}|note\\s+for\\s+${name})(?![\\p{L}\\p{N}_])`,
        "u",
      ).test(line)
    )
      lines.add(at);
  });
  return lines;
};

export const classRemoved = (diagram: Classes, name: string): ClassAltered => {
  const going = goesWith(diagram, name);
  return spacesKept(diagram, {
    body: diagram.body.filter((_, at) => !going.has(at)),
    places: diagram.places.filter((one) => one.id !== name),
    picked: null,
  });
};

/** A member goes inside the class's braces; a class without them gets them. */
export const memberAdded = (diagram: Classes, name: string, text: string): ClassAltered => {
  const own = classOf(diagram, name);
  const count = own?.members.length ?? 0;
  const picked: ClassPicked = { kind: "member", name, index: count };
  if (own?.close !== null && own?.close !== undefined && own.decl !== null) {
    const inner = `${indentOf(diagram.body[own.decl])}  `;
    const body = [
      ...diagram.body.slice(0, own.close),
      `${inner}${text}`,
      ...diagram.body.slice(own.close),
    ];
    return { body, picked };
  }
  if (own?.decl !== null && own?.decl !== undefined) {
    const decl = diagram.body[own.decl];
    const indent = indentOf(decl);
    const body = [
      ...diagram.body.slice(0, own.decl),
      `${decl.trimEnd()} {`,
      `${indent}  ${text}`,
      `${indent}}`,
      ...diagram.body.slice(own.decl + 1),
    ];
    return { body, picked };
  }
  const indent = indentIn(diagram);
  return {
    body: [...diagram.body, `${indent}class ${name} {`, `${indent}  ${text}`, `${indent}}`],
    picked,
  };
};

export const memberChanged = (
  diagram: Classes,
  name: string,
  index: number,
  text: string,
): ClassAltered => {
  const member = classOf(diagram, name)?.members[index];
  const said = text.trim();
  if (!member || !said) return { body: diagram.body, picked: { kind: "member", name, index } };
  const line = diagram.body[member.line];
  const written = member.colon ? `${indentOf(line)}${name} : ${said}` : `${indentOf(line)}${said}`;
  return {
    body: diagram.body.map((one, at) => (at === member.line ? written : one)),
    picked: { kind: "member", name, index },
  };
};

export const memberRemoved = (diagram: Classes, name: string, index: number): ClassAltered => {
  const member = classOf(diagram, name)?.members[index];
  if (!member) return { body: diagram.body, picked: { kind: "class", name } };
  return {
    body: diagram.body.filter((_, at) => at !== member.line),
    picked: { kind: "class", name },
  };
};

/** A relation from one class to another, or the one already between them. */
export const classesLinked = (diagram: Classes, from: string, to: string): ClassAltered => {
  const already = diagram.links.findIndex(
    (one) => (one.left === from && one.right === to) || (one.left === to && one.right === from),
  );
  if (already >= 0) return { body: diagram.body, picked: { kind: "link", index: already } };
  return {
    body: [...diagram.body, `${indentIn(diagram)}${from} --> ${to}`],
    picked: { kind: "link", index: diagram.links.length },
  };
};

export const linkChanged = (
  diagram: Classes,
  index: number,
  change: Partial<Pick<ClassLink, "arrow" | "leftCard" | "rightCard" | "label">>,
): ClassAltered => {
  const link = diagram.links[index];
  if (!link) return { body: diagram.body, picked: null };
  const clean = (said: string | undefined, was: string) => (said ?? was).replace(/"/g, "'").trim();
  const said = linkLine(indentOf(diagram.body[link.line]), {
    ...link,
    arrow: change.arrow ?? link.arrow,
    leftCard: clean(change.leftCard, link.leftCard),
    rightCard: clean(change.rightCard, link.rightCard),
    label: clean(change.label, link.label),
  });
  return {
    body: diagram.body.map((one, at) => (at === link.line ? said : one)),
    picked: { kind: "link", index },
  };
};

export const linkRemoved = (diagram: Classes, index: number): ClassAltered => {
  const link = diagram.links[index];
  if (!link) return { body: diagram.body, picked: null };
  return { body: diagram.body.filter((_, at) => at !== link.line), picked: null };
};

/** A namespace taken apart: its opening and closing lines go, its classes stay. */
export const namespaceRemoved = (diagram: Classes, id: string): ClassAltered => {
  const group = diagram.groups.find((one) => one.id === id);
  if (!group) return { body: diagram.body, picked: null };
  const inner = `${indentOf(diagram.body[group.open])}  `;
  // What was inside goes back to the indent of the rest of the body.
  return {
    body: diagram.body.flatMap((line, at) => {
      if (at === group.open || at === group.close) return [];
      const inside = at > group.open && at < group.close && line.startsWith(inner);
      return [inside ? line.slice(2) : line];
    }),
    picked: null,
  };
};

/** What taking out the chosen thing leaves; several classes go one after another. */
export const classPickedRemoved = (diagram: Classes, picked: ClassPicked): ClassAltered | null => {
  if (picked?.kind === "class") return classRemoved(diagram, picked.name);
  if (picked?.kind === "member") return memberRemoved(diagram, picked.name, picked.index);
  if (picked?.kind === "link") return linkRemoved(diagram, picked.index);
  if (picked?.kind === "namespace") return namespaceRemoved(diagram, picked.name);
  if (picked?.kind !== "classes") return null;
  const left = picked.names.reduce(
    (now, name) => classReread(now, classRemoved(now, name)),
    diagram,
  );
  return { body: left.body, places: left.places, picked: null };
};

// The same kind of relation written the other way round, its head at the other class.
const MIRRORED: Record<string, string> = {
  "<|--": "--|>",
  "<|..": "..|>",
  "*--": "--*",
  "o--": "--o",
  "<--": "-->",
  "<..": "..>",
};
const mirrorOf = (arrow: string) =>
  MIRRORED[arrow] ?? Object.keys(MIRRORED).find((one) => MIRRORED[one] === arrow) ?? arrow;
const headFirst = (arrow: string) => /^(<|\*|o)/.test(arrow);

/** Whether a relation is of a kind, written either way round. */
export const sameKind = (arrow: string, kind: string) => arrow === kind || arrow === mirrorOf(kind);

/** A new kind for a relation, its head kept at the class it was at so its meaning does not turn. */
export const kindKept = (arrow: string, kind: string): string => {
  const headed = /[<>*o]/.test(arrow.replace(/\|/g, ""));
  return headed && headFirst(arrow) !== headFirst(kind) ? mirrorOf(kind) : kind;
};
