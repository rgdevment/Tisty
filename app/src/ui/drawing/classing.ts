import { type Classes, type ClassLink, linkLine } from "./classes";
import { renamedLine } from "./naming";

/** What is chosen: a class by its name, a member by its place in it, a relation by its place among them. */
export type ClassPicked =
  | { readonly kind: "class"; readonly name: string }
  | { readonly kind: "member"; readonly name: string; readonly index: number }
  | { readonly kind: "link"; readonly index: number }
  | null;

/** A change is new text for the body of the diagram, read again by whoever shows it. */
export interface ClassAltered {
  readonly body: readonly string[];
  readonly picked: ClassPicked;
}

const indentOf = (line: string | undefined) =>
  line?.slice(0, line.length - line.trimStart().length) ?? "";
const indentIn = (diagram: Classes) => indentOf(diagram.body.find((one) => one.trim())) || "  ";
const classOf = (diagram: Classes, name: string) =>
  diagram.classes.find((one) => one.name === name);
const plain = (said: string) => said.trim().replace(/\s+/g, "_");

export const classAdded = (diagram: Classes, wanted: string): ClassAltered => {
  const taken = diagram.classes.map((one) => one.name);
  let n = 1;
  while (taken.includes(`${wanted}${n}`)) n += 1;
  const name = `${wanted}${n}`;
  return {
    body: [...diagram.body, `${indentIn(diagram)}class ${name}`],
    picked: { kind: "class", name },
  };
};

/** A class renamed everywhere it stands as a word of its own: declaration, relations, members, styles. */
export const classRenamed = (diagram: Classes, from: string, wanted: string): ClassAltered => {
  const to = plain(wanted);
  const taken = diagram.classes.some((one) => one.name === to);
  if (!/^[\p{L}\p{N}_]+$/u.test(to) || to === from || taken)
    return { body: diagram.body, picked: { kind: "class", name: from } };
  return {
    body: diagram.body.map((line) => renamedLine(line, from, to)),
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
  return { body: diagram.body.filter((_, at) => !going.has(at)), picked: null };
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
