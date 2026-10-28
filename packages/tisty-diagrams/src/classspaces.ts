import type { Classes } from "./classes";
import { type ClassAltered, classReread, indentIn, indentOf, plain, spacesKept } from "./classing";
import type { Rect } from "./graphlayout";
import { placed, type Spot } from "./places";
import { laneHolding, lanesIn } from "./scenes";

/** The namespaces drawn as lanes: those whose place carries a size. */
export const classLanesOf = (diagram: Classes) =>
  lanesIn(
    diagram.groups.map((one) => ({ id: one.id, title: one.id })),
    diagram.places,
  );

const unique = (wanted: string, taken: readonly string[]) => {
  let n = 1;
  while (taken.includes(`${wanted}${n}`)) n += 1;
  return `${wanted}${n}`;
};

/** A class moved into a namespace, or out of every one, with its braces and all it says inside them. */
export const classMovedTo = (
  diagram: Classes,
  name: string,
  space: string | null,
): ClassAltered => {
  const own = diagram.classes.find((one) => one.name === name);
  const picked = { kind: "class", name } as const;
  if (!own || own.group === space || (space && !diagram.groups.some((one) => one.id === space)))
    return { body: diagram.body, picked };
  const from = own.decl;
  const to = own.close ?? own.decl;
  const lines = from === null ? [`class ${name}`] : diagram.body.slice(from, (to ?? from) + 1);
  const base = indentOf(lines[0]);
  const left =
    from === null ? diagram.body : diagram.body.filter((_, at) => at < from || at > (to ?? from));
  const now = classReread(diagram, spacesKept(diagram, { body: left, picked }));
  const target = now.groups.find((one) => one.id === space);
  const indent = target ? `${indentOf(now.body[target.open])}  ` : indentIn(now);
  const moved = lines.map((line) =>
    line.startsWith(base) ? `${indent}${line.slice(base.length)}` : `${indent}${line.trimStart()}`,
  );
  const body = target
    ? [...now.body.slice(0, target.close), ...moved, ...now.body.slice(target.close)]
    : [...now.body, ...moved];
  return { body, places: now.places, picked };
};

/** A namespace put where it was asked, with a new class in it, as Mermaid takes none empty. */
export const namespaceAdded = (
  diagram: Classes,
  title: string,
  wanted: string,
  box: Rect,
): ClassAltered => {
  const space = unique(
    plain(title),
    diagram.groups.map((one) => one.id),
  );
  const name = unique(
    wanted,
    diagram.classes.map((one) => one.name),
  );
  const indent = indentIn(diagram);
  const body = [
    ...diagram.body,
    `${indent}namespace ${space} {`,
    `${indent}  class ${name}`,
    `${indent}}`,
  ];
  const boxed = placed(diagram.places, indent, space, box);
  const places = placed(boxed, indent, name, { x: box.x + 24, y: box.y + 48 });
  return { body, places, picked: { kind: "class", name } };
};

/** A namespace renamed where it opens and where it is placed. */
export const namespaceRenamed = (diagram: Classes, from: string, wanted: string): ClassAltered => {
  const to = plain(wanted);
  const group = diagram.groups.find((one) => one.id === from);
  const taken = diagram.groups.some((one) => one.id === to);
  if (!group || !/^[\p{L}\p{N}_.]+$/u.test(to) || to === from || taken)
    return { body: diagram.body, picked: { kind: "namespace", name: from } };
  return {
    body: diagram.body.map((line, at) => (at === group.open ? line.replace(from, to) : line)),
    places: diagram.places.map((one) =>
      one.id === from
        ? { ...one, id: to, line: one.line.replace(`tisty: ${from} `, `tisty: ${to} `) }
        : one,
    ),
    picked: { kind: "namespace", name: to },
  };
};

/** A namespace moved with every class in it, so they keep their places inside it. */
export const namespaceMoved = (diagram: Classes, id: string, to: Spot): ClassAltered => {
  const lane = classLanesOf(diagram).find((one) => one.id === id);
  const picked = { kind: "namespace", name: id } as const;
  if (!lane) return { body: diagram.body, picked };
  const indent = indentIn(diagram);
  const shifted = diagram.classes
    .filter((one) => one.group === id)
    .reduce((places, one) => {
      const place = places.find((at) => at.id === one.name);
      if (!place) return places;
      return placed(places, indent, one.name, {
        x: place.x + to.x - lane.rect.x,
        y: place.y + to.y - lane.rect.y,
      });
    }, diagram.places);
  return { body: diagram.body, places: placed(shifted, indent, id, to), picked };
};

/** A namespace's new size, its classes left where they are. */
export const namespaceResized = (diagram: Classes, id: string, box: Rect): ClassAltered => ({
  body: diagram.body,
  places: placed(diagram.places, indentIn(diagram), id, box),
  picked: { kind: "namespace", name: id },
});

/** Classes let go in a namespace's lane join it, and let go outside every lane leave theirs. */
export const reclassed = (diagram: Classes, at: ReadonlyMap<string, Rect>): Classes => {
  const lanes = classLanesOf(diagram);
  let now = diagram;
  for (const [name, rect] of at) {
    const own = now.classes.find((one) => one.name === name);
    if (!own) continue;
    const into = laneHolding(lanes, rect);
    const inLane = lanes.some((one) => one.id === own.group);
    if (into === own.group || (into === null && !inLane)) continue;
    now = classReread(now, classMovedTo(now, name, into));
  }
  return now;
};
