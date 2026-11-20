import Graph from "graphology";
import forceAtlas2 from "graphology-layout-forceatlas2";
import { type Neuron, TIES, type Tie } from "../../brainsty";

export interface Spots {
  x: Float32Array;
  y: Float32Array;
}

export interface Asked extends Spots {
  edges: Uint32Array;
  weights?: Float32Array;
}

// What a person or an agent wrote pulls hardest; a tag barely, and guesses stay out of the layout.
export const PULL: Readonly<Record<Tie, number>> = {
  cite: 3,
  mention: 3,
  structure: 3,
  uses: 1.5,
  tag: 0.3,
};

export function weightsOf(edges: readonly [number, number, number][]): Float32Array {
  return Float32Array.from(edges, ([, , code]) => PULL[TIES[code] ?? "tag"]);
}

export interface Step extends Spots {
  done: boolean;
  iterations: number;
  ms: number;
}

const SPREAD = 10;
const BATCH = 10;
const MOST = 300;
const BUDGET_MS = 4000;
const CALM = 0.002;
// Each step costs the page time in proportion to its nodes, so a big network shows fewer of them.
const STEP_MS_PER_NODE = 1 / 30;

const hashed = (text: string): number => {
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.codePointAt(i) ?? 0;
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0) / 4294967296;
};

// Each folder or list starts in its own place around a ring, so the forces pull apart zones that already lean apart.
export function seeded(nodes: readonly Neuron[]): Spots {
  const sizes = new Map<string, number>();
  for (const one of nodes) sizes.set(one.group ?? "", (sizes.get(one.group ?? "") ?? 0) + 1);
  const groups = [...sizes.keys()].filter(Boolean).sort((one, other) => one.localeCompare(other));
  const turn = new Map(groups.map((group, n) => [group, (n / groups.length) * 2 * Math.PI]));
  const ring = Math.sqrt(nodes.length) * SPREAD;
  const x = new Float32Array(nodes.length);
  const y = new Float32Array(nodes.length);
  nodes.forEach((one, n) => {
    const group = one.group ?? "";
    const angle = turn.get(group);
    const reach = Math.sqrt(sizes.get(group) ?? 1) * SPREAD * 0.5 * hashed(one.id);
    const spin = 2 * Math.PI * hashed(`${one.id}~`);
    x[n] = (angle === undefined ? 0 : Math.cos(angle) * ring) + Math.cos(spin) * reach;
    y[n] = (angle === undefined ? 0 : Math.sin(angle) * ring) + Math.sin(spin) * reach;
  });
  return { x, y };
}

export function edgesOf(edges: readonly [number, number, number][]): Uint32Array {
  const flat = new Uint32Array(edges.length * 2);
  edges.forEach(([from, to], n) => {
    flat[n * 2] = from;
    flat[n * 2 + 1] = to;
  });
  return flat;
}

export function graphOf({ x, y, edges, weights }: Asked): Graph {
  const graph = new Graph({ type: "undirected" });
  for (let n = 0; n < x.length; n++) graph.addNode(String(n), { x: x[n], y: y[n] });
  for (let n = 0; n < edges.length; n += 2) {
    const from = String(edges[n]);
    const to = String(edges[n + 1]);
    if (from === to || !graph.hasNode(from) || !graph.hasNode(to)) continue;
    const weight = weights?.[n / 2] ?? 1;
    graph.updateEdge(from, to, (at) => ({ weight: Math.max(Number(at.weight ?? 0), weight) }));
  }
  return graph;
}

export function spotsOf(graph: Graph): Spots {
  const x = new Float32Array(graph.order);
  const y = new Float32Array(graph.order);
  graph.forEachNode((key, at) => {
    x[Number(key)] = at.x;
    y[Number(key)] = at.y;
  });
  return { x, y };
}

const widthOf = ({ x, y }: Spots): number => {
  let [left, right, top, bottom] = [Infinity, -Infinity, Infinity, -Infinity];
  for (let n = 0; n < x.length; n++) {
    left = Math.min(left, x[n]);
    right = Math.max(right, x[n]);
    top = Math.min(top, y[n]);
    bottom = Math.max(bottom, y[n]);
  }
  return Math.max(right - left, bottom - top, 1);
};

// How far the nodes moved in one batch, measured against the width of the whole network.
export function movedBy(before: Spots, after: Spots): number {
  if (after.x.length === 0) return 0;
  let sum = 0;
  for (let n = 0; n < after.x.length; n++)
    sum += Math.hypot(after.x[n] - before.x[n], after.y[n] - before.y[n]);
  return sum / after.x.length / widthOf(after);
}

// Gephi's coarser Barnes–Hut makes a big network's iterations about three times cheaper, and a lighter brake lets each one go further.
export function forcesFor(order: number) {
  const inferred = forceAtlas2.inferSettings(order);
  const big = inferred.barnesHutOptimize ? { barnesHutTheta: 1.2, slowDown: 3 } : {};
  return { ...inferred, gravity: 1, scalingRatio: 10, ...big };
}

export function settle(
  asked: Asked,
  tell: (step: Step) => void,
  now: () => number = () => performance.now(),
) {
  const started = now();
  const graph = graphOf(asked);
  const settings = forcesFor(graph.order);
  let spots = spotsOf(graph);
  let iterations = 0;
  let told = started;
  while (iterations < MOST && graph.size > 0 && now() - started < BUDGET_MS) {
    forceAtlas2.assign(graph, { iterations: BATCH, settings });
    iterations += BATCH;
    const next = spotsOf(graph);
    const moved = movedBy(spots, next);
    spots = next;
    if (moved < CALM) break;
    const at = now();
    if (at - told < graph.order * STEP_MS_PER_NODE) continue;
    told = at;
    tell({ x: next.x.slice(), y: next.y.slice(), done: false, iterations, ms: at - started });
  }
  tell({ ...spots, done: true, iterations, ms: now() - started });
}
