import type { NeuronKind } from "../../brainsty";

const MEMORY = 30;
const TRAIL_NODES = 6;
const REST = 0.6;
const FLASH_REST = 1.1;
const AWAY_HOPS = 12;
const OFFSCREEN = 0.15;

export interface World {
  neighbours: (node: number) => readonly number[];
  pull: (from: number, to: number) => number;
  kindOf: (node: number) => NeuronKind;
  big: (node: number) => boolean;
  flashy: (node: number) => boolean;
  seen: (node: number) => boolean;
  start: () => number;
  rand: () => number;
}

export interface Traveller {
  from: number;
  to: number;
  t: number;
  rest: number;
  recent: number[];
  trail: number[];
  away: number;
}

export interface Flash {
  node: number;
  age: number;
}

export const ease = (t: number): number => 0.5 - 0.5 * Math.cos(Math.PI * t);

export const setOut = (at: number): Traveller => ({
  from: at,
  to: at,
  t: 1,
  rest: REST,
  recent: [at],
  trail: [at],
  away: 0,
});

const WEIGHT: Partial<Record<NeuronKind, number>> = { doc: 4, page: 4, task: 1.2, tag: 0.7 };

function weightOf(world: World, one: Traveller, node: number): number {
  let weight = (WEIGHT[world.kindOf(node)] ?? 0.6) * world.pull(one.to, node);
  if (world.big(node)) weight *= 0.6;
  if (one.recent.includes(node)) weight *= 0.03;
  if (!world.seen(node)) weight *= OFFSCREEN;
  return weight;
}

// The next node along a real line, back one hop at a dead end, or none for a node with no line at all.
function nextOf(world: World, one: Traveller): number | null {
  const all = world.neighbours(one.to);
  const options = all.filter((node) => node !== one.from && node !== one.to);
  if (options.length === 0) return all.length > 0 ? one.from : null;
  const weights = options.map((node) => weightOf(world, one, node));
  let at = world.rand() * weights.reduce((sum, weight) => sum + weight, 0);
  for (let n = 0; n < options.length; n++) {
    at -= weights[n];
    if (at <= 0) return options[n];
  }
  return options[options.length - 1];
}

function arrived(world: World, one: Traveller, flashes: Flash[]): Traveller {
  const at = one.to;
  const recent = [...one.recent, at].slice(-MEMORY);
  const trail = [...one.trail, at].slice(-TRAIL_NODES);
  const away = world.seen(at) ? 0 : one.away + 1;
  if (away >= AWAY_HOPS) return setOut(world.start());
  const flashy = world.flashy(at);
  if (flashy) flashes.push({ node: at, age: 0 });
  return { ...one, recent, trail, away, rest: flashy ? FLASH_REST : REST };
}

export function stepped(
  world: World,
  one: Traveller,
  seconds: number,
  hop: number,
  flashes: Flash[],
): Traveller {
  if (one.t < 1) {
    const t = Math.min(1, one.t + seconds / hop);
    return t === 1 ? arrived(world, { ...one, t }, flashes) : { ...one, t };
  }
  const rest = one.rest - seconds;
  if (rest > 0) return { ...one, rest };
  const next = nextOf(world, one);
  if (next === null) return setOut(world.start());
  return { ...one, from: one.to, to: next, t: 0, rest: 0 };
}
