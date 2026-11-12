import type { Edge, InternalNode } from "@xyflow/react";
import { describe, expect, it } from "vitest";
import { readClasses } from "../src/classes";
import { readFlow } from "../src/flow";
import { endWordBox, overlaps, routeOf, routesOf } from "../src/flowroutes";
import type { Rect } from "../src/graphlayout";
import { layered } from "../src/graphlayout";
import { pillSize } from "../src/picture";
import { classScene, flowScene, type Scene } from "../src/scenes";
import { routesIn, shapesSaid } from "../src/wiring";

const GUIDE = `flowchart TD
  A[Aparece algo] --> B{¿Lo vas a hacer,<br/>o a volver a leer?}
  B -->|Hacerlo| T[Una tarea]
  B -->|Volver a leerlo| D[Un documento]
  B -->|Todavía no lo sé| I[Una idea en la<br/>Lluvia de ideas]
  I -->|Convertir en tarea| T
  I -->|Anexar a un documento| D
  T --> P[Su página: notas, pasos,<br/>archivos, grabaciones]
  D --> G[Léelo, corrígelo y amplíalo<br/>durante años]`;

const SKIPPED = `flowchart TD
  A[Uno] --> B[Dos]
  B --> C[Tres]
  A -->|Saltar| C`;

const ACROSS = `flowchart LR
  A[Pide] -->|Enviar al cliente para revisar| B[Revisa]`;

const OWNED = `classDiagram
  class Animal
  class Leg
  Animal "1" *-- "many" Leg : tiene`;

const rect = (x: number, y: number, wide = 100, tall = 40): Rect => ({ x, y, wide, tall });

const runsOf = (way: readonly { x: number; y: number }[]) =>
  way.slice(1).map((one, at) => ({
    x: Math.min(way[at].x, one.x),
    y: Math.min(way[at].y, one.y),
    wide: Math.abs(one.x - way[at].x),
    tall: Math.abs(one.y - way[at].y),
  }));

const flowOf = (source: string) => {
  const flow = readFlow(source);
  if (!flow) throw new Error(`does not read: ${source}`);
  return flowScene(flow);
};

const classesOf = (source: string) => {
  const diagram = readClasses(source);
  if (!diagram) throw new Error(`does not read: ${source}`);
  return classScene(diagram);
};

const crowdingIn = (scene: Scene) => {
  const shapes = [...scene.laid.at.values()];
  const joints = scene.joints.map((one) => ({
    from: scene.laid.at.get(one.from) as Rect,
    to: scene.laid.at.get(one.to) as Rect,
    label: one.label,
    startLabel: one.startLabel,
    endLabel: one.endLabel,
  }));
  const routes = routesOf(joints, shapes);
  const crossed = routes.flatMap((route, at) => {
    const others = shapes.filter((one) => one !== joints[at].from && one !== joints[at].to);
    return runsOf(route.way).some((run) => others.some((one) => overlaps(run, one)))
      ? [joints[at].label || `${at}`]
      : [];
  });
  const ends = routes.flatMap((route, at) => [
    ...(joints[at].startLabel
      ? [endWordBox(route.from, route.fromSide, joints[at].startLabel as string)]
      : []),
    ...(joints[at].endLabel
      ? [endWordBox(route.to, route.toSide, joints[at].endLabel as string)]
      : []),
  ]);
  const words = routes.flatMap((route, at) => {
    const said = joints[at].label;
    if (!said) return [];
    const { wide, tall } = pillSize(said);
    return [{ said, box: { x: route.labelX - wide / 2, y: route.labelY - tall / 2, wide, tall } }];
  });
  const covering = words.flatMap((one) => [
    ...(shapes.some((shape) => overlaps(one.box, shape)) ? [`${one.said} / a shape`] : []),
    ...(ends.some((end) => overlaps(one.box, end)) ? [`${one.said} / an end word`] : []),
    ...words
      .filter((other) => other !== one && overlaps(one.box, other.box))
      .map((other) => `${one.said} / ${other.said}`),
  ]);
  return { crossed, covering };
};

const node = (id: string, x: number, y: number) =>
  ({
    id,
    type: "shape",
    internals: { positionAbsolute: { x, y } },
    measured: { width: 100, height: 40 },
  }) as unknown as InternalNode;

describe("a crowded drawing", () => {
  it("runs an arrow round a shape that stands in its way, and as before when none does", () => {
    const from = rect(100, 0);
    const to = rect(0, 240);
    const between = rect(100, 120);

    const round = routeOf(from, to, [from, to, between]);
    const open = routeOf(from, to, [from, to]);

    expect(runsOf(round.way).some((run) => overlaps(run, between))).toBe(false);
    expect(open.path).toBe(routeOf(from, to).path);
  });

  it("sets shapes apart, in a row and between rows, for the words on the arrows between them", () => {
    const sizes = new Map(["a", "b", "c"].map((id) => [id, { wide: 100, tall: 40 }]));
    const edges = [
      ["a", "b"],
      ["a", "c"],
    ] as const;
    const words = {
      aside: new Map([
        ["b", 200],
        ["c", 200],
      ]),
      along: new Map([["a", 150]]),
    };

    const tight = layered(["a", "b", "c"], edges, sizes, "TB");
    const roomy = layered(["a", "b", "c"], edges, sizes, "TB", words);
    const apart = (laid: typeof tight) =>
      (laid.at.get("c")?.x ?? 0) - ((laid.at.get("b")?.x ?? 0) + 100);
    const below = (laid: typeof tight) =>
      (laid.at.get("b")?.y ?? 0) - ((laid.at.get("a")?.y ?? 0) + 40);

    expect(apart(tight)).toBe(56);
    expect(below(tight)).toBeCloseTo(78.4);
    expect(apart(roomy)).toBe(200);
    expect(below(roomy)).toBeCloseTo(198);
  });

  it.each([
    ["the guide's first diagram", flowOf(GUIDE)],
    ["an arrow that skips the shape in line with it", flowOf(SKIPPED)],
    ["a long word on a left-to-right arrow", flowOf(ACROSS)],
    ["a relation with words at its ends", classesOf(OWNED)],
  ])("draws %s with no arrow behind a shape and no word over another", (_, scene) => {
    expect(crowdingIn(scene)).toEqual({ crossed: [], covering: [] });
  });

  it("lays a window's wires once per change, and again when a shape moves", () => {
    const edges: Edge[] = [{ id: "e", source: "a", target: "b", data: { label: "sigue" } }];
    const still = new Map([
      ["a", node("a", 0, 0)],
      ["b", node("b", 0, 200)],
    ]);
    const moved = new Map([
      ["a", node("a", 0, 0)],
      ["b", node("b", 300, 200)],
    ]);

    const first = routesIn(edges, shapesSaid(still), still);
    const again = routesIn(edges, shapesSaid(still), still);
    const after = routesIn(edges, shapesSaid(moved), moved);

    expect(again).toBe(first);
    expect(after).not.toBe(first);
    expect(after.get("e")?.to).toEqual({ x: 300, y: 220 });
  });
});
