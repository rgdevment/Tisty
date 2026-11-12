import { describe, expect, it } from "vitest";
import { readFlow } from "../src/flow";
import { overlaps, routeOf, routesOf } from "../src/flowroutes";
import type { Rect } from "../src/graphlayout";
import { layered } from "../src/graphlayout";
import { pillSize } from "../src/picture";
import { flowScene } from "../src/scenes";

const GUIDE = `flowchart TD
  A[Aparece algo] --> B{¿Lo vas a hacer,<br/>o a volver a leer?}
  B -->|Hacerlo| T[Una tarea]
  B -->|Volver a leerlo| D[Un documento]
  B -->|Todavía no lo sé| I[Una idea en la<br/>Lluvia de ideas]
  I -->|Convertir en tarea| T
  I -->|Anexar a un documento| D
  T --> P[Su página: notas, pasos,<br/>archivos, grabaciones]
  D --> G[Léelo, corrígelo y amplíalo<br/>durante años]`;

const rect = (x: number, y: number, wide = 100, tall = 40): Rect => ({ x, y, wide, tall });

const runsOf = (way: readonly { x: number; y: number }[]) =>
  way.slice(1).map((one, at) => ({
    x: Math.min(way[at].x, one.x),
    y: Math.min(way[at].y, one.y),
    wide: Math.abs(one.x - way[at].x),
    tall: Math.abs(one.y - way[at].y),
  }));

const guideDrawn = () => {
  const flow = readFlow(GUIDE);
  if (!flow) throw new Error("the guide's diagram does not read");
  const scene = flowScene(flow);
  const shapes = [...scene.laid.at.values()];
  const joints = scene.joints.map((one) => ({
    from: scene.laid.at.get(one.from) as Rect,
    to: scene.laid.at.get(one.to) as Rect,
    label: one.label,
  }));
  return { shapes, joints, routes: routesOf(joints, shapes) };
};

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

  it("sets shapes in a row far enough apart for the words on the arrows that reach them", () => {
    const sizes = new Map(["a", "b", "c"].map((id) => [id, { wide: 100, tall: 40 }]));
    const edges = [
      ["a", "b"],
      ["a", "c"],
    ] as const;

    const tight = layered(["a", "b", "c"], edges, sizes, "TB");
    const roomy = layered(
      ["a", "b", "c"],
      edges,
      sizes,
      "TB",
      new Map([
        ["b", 200],
        ["c", 200],
      ]),
    );
    const gap = (laid: typeof tight) =>
      (laid.at.get("c")?.x ?? 0) - ((laid.at.get("b")?.x ?? 0) + 100);

    expect(gap(tight)).toBe(56);
    expect(gap(roomy)).toBe(200);
  });

  it("draws the guide's first diagram with no arrow behind a shape and no word over another", () => {
    const { shapes, joints, routes } = guideDrawn();

    routes.forEach((route, at) => {
      const others = shapes.filter((one) => one !== joints[at].from && one !== joints[at].to);
      for (const run of runsOf(route.way))
        expect(
          others.some((one) => overlaps(run, one)),
          joints[at].label,
        ).toBe(false);
    });
    const words = routes.flatMap((route, at) => {
      const said = joints[at].label;
      if (!said) return [];
      const { wide, tall } = pillSize(said);
      return [
        { said, box: { x: route.labelX - wide / 2, y: route.labelY - tall / 2, wide, tall } },
      ];
    });
    for (const one of words) {
      expect(
        shapes.some((shape) => overlaps(one.box, shape)),
        one.said,
      ).toBe(false);
      for (const other of words)
        if (other !== one)
          expect(overlaps(one.box, other.box), `${one.said} / ${other.said}`).toBe(false);
    }
  });
});
