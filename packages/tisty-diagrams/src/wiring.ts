import type { Edge, InternalNode } from "@xyflow/react";
import { type Route, routesOf } from "./flowroutes";
import type { Rect } from "./graphlayout";

type Worded = {
  readonly label?: string;
  readonly startLabel?: string;
  readonly endLabel?: string;
};

export const rectOf = (node: InternalNode): Rect => ({
  x: node.internals.positionAbsolute.x,
  y: node.internals.positionAbsolute.y,
  wide: node.measured.width ?? node.width ?? 0,
  tall: node.measured.height ?? node.height ?? 0,
});

const shapesIn = (lookup: ReadonlyMap<string, InternalNode>) =>
  [...lookup.values()].filter((one) => one.type !== "lane");

export const shapesSaid = (lookup: ReadonlyMap<string, InternalNode>) =>
  shapesIn(lookup)
    .map((one) => {
      const rect = rectOf(one);
      return `${one.id} ${rect.x} ${rect.y} ${rect.wide} ${rect.tall}`;
    })
    .join(";");

const saidIn = new WeakMap<object, string>();

export const shapesSaidOnce = (state: {
  readonly nodeLookup: ReadonlyMap<string, InternalNode>;
}) => {
  const known = saidIn.get(state);
  if (known !== undefined) return known;
  const said = shapesSaid(state.nodeLookup);
  saidIn.set(state, said);
  return said;
};

const laid = new WeakMap<
  readonly Edge[],
  { readonly said: string; readonly routes: ReadonlyMap<string, Route> }
>();

// Laid once per change and shared by every wire, so dragging one shape does not reroute the drawing once per wire.
export const routesIn = (
  edges: readonly Edge[],
  said: string,
  lookup: ReadonlyMap<string, InternalNode>,
): ReadonlyMap<string, Route> => {
  const known = laid.get(edges);
  if (known?.said === said) return known.routes;
  const joints = edges.flatMap((one) => {
    const from = lookup.get(one.source);
    const to = lookup.get(one.target);
    const words = (one.data ?? {}) as Worded;
    return from && to
      ? [
          {
            id: one.id,
            from: rectOf(from),
            to: rectOf(to),
            label: words.label,
            startLabel: words.startLabel,
            endLabel: words.endLabel,
          },
        ]
      : [];
  });
  const routes = routesOf(joints, shapesIn(lookup).map(rectOf));
  const made = new Map(joints.map((one, at) => [one.id, routes[at]]));
  laid.set(edges, { said, routes: made });
  return made;
};
