import type { Network } from "../../brainsty";
import type { Drawing } from "./drawing";
import { PITCH_AT_MOST, type Turn } from "./globe";
import { isGroup, laidEdges } from "./grouping";
import { bigOnes, degreesOf, rgba } from "./looks";
import { fitted, LAID } from "./overlays";
import { ease, type Flash, setOut, stepped, type Traveller, type World } from "./travellers";

const STEPS: readonly [below: number, travellers: number][] = [
  [60, 2],
  [100, 4],
  [150, 6],
  [250, 8],
];

export const travellersFor = (nodes: number): number =>
  STEPS.find(([below]) => nodes < below)?.[1] ?? 10;
export const HOP_SECONDS = 5;
const TURN_SECONDS = 60;
const FLASH_SECONDS = 0.6;
const LONGEST_STEP = 0.1;
const ZOOM_REST_MS = 800;
const SHAPE_REACH_PX = 32;
const MEASURED_EVERY = 30;

export interface Living {
  ready: () => void;
  travel: (on: boolean) => void;
  spin: (on: boolean) => void;
  measured: () => { ms: number; fps: number };
  turnBy: (yaw: number, pitch: number) => void;
  hold: (held: boolean) => void;
  sift: (hidden: readonly boolean[]) => void;
  stop: () => void;
}

export interface Clock {
  now: () => number;
  frame: (step: (at: number) => void) => number;
  cancel: (frame: number) => void;
}

const CLOCK: Clock = {
  now: () => performance.now(),
  frame: (step) => requestAnimationFrame(step),
  cancel: (frame) => cancelAnimationFrame(frame),
};

function neighboursOf(network: Network): number[][] {
  const near = network.nodes.map((): number[] => []);
  for (const [from, to] of laidEdges(network)) {
    near[from].push(to);
    near[to].push(from);
  }
  return near;
}

export function livingOn(
  container: HTMLElement,
  drawing: Drawing,
  network: Network,
  colourOf: (node: number) => string,
  { clock = CLOCK }: { clock?: Clock } = {},
): Living {
  const canvas = container.ownerDocument.createElement("canvas");
  Object.assign(canvas.style, LAID);
  container.append(canvas);
  const near = neighboursOf(network);
  const big = bigOnes(degreesOf(network.nodes.length, laidEdges(network)));
  let hidden: readonly boolean[] = [];
  const onScreen = (node: number): boolean => {
    const at = drawing.nodeOnScreen(node);
    return (
      at !== undefined &&
      at[0] >= 0 &&
      at[1] >= 0 &&
      at[0] <= container.clientWidth &&
      at[1] <= container.clientHeight
    );
  };
  const walkable = (node: number) => !hidden[node] && near[node].length > 0;
  const world: World = {
    neighbours: (node) => near[node].filter((one) => !hidden[one]),
    kindOf: (node) => network.nodes[node].kind,
    big: (node) => big[node],
    flashy: (node) => network.nodes[node].kind === "tag" || big[node],
    seen: (node) => !hidden[node] && onScreen(node),
    start: () => {
      const hubs = network.nodes.flatMap((_, node) =>
        walkable(node) && big[node] && onScreen(node) ? [node] : [],
      );
      const any =
        hubs.length > 0 ? hubs : network.nodes.flatMap((_, n) => (walkable(n) ? [n] : []));
      return any.length > 0 ? any[Math.floor(Math.random() * any.length)] : 0;
    },
    rand: Math.random,
  };
  let travellers: Traveller[] = [];
  let flashes: Flash[] = [];
  let turn: Turn = { yaw: 0, pitch: 0 };
  let laid = false;
  let frame = 0;
  let last = 0;
  let spent = 0;
  let frames = 0;
  let ticked = 0;
  let rendering = 0;
  let held = false;
  let pointer: [number, number] | null = null;
  let zoomedAt = Number.NEGATIVE_INFINITY;
  let travelling = true;
  let spinning = true;
  let counted = 0;
  let countedFrom = 0;
  let fps = 0;
  const many = travellersFor(network.nodes.filter((one) => !isGroup(one.kind)).length);

  const painted = () => {
    const brush = canvas.getContext("2d");
    if (!brush) return;
    fitted(canvas, container);
    brush.clearRect(0, 0, canvas.width, canvas.height);
    const ratio = canvas.width / Math.max(container.clientWidth, 1);
    const spot = (node: number) => {
      const at = hidden[node] ? undefined : drawing.nodeOnScreen(node);
      return at && [at[0] * ratio, at[1] * ratio];
    };
    for (const one of travellers) {
      const from = spot(one.from);
      const to = spot(one.to);
      if (!from || !to) continue;
      const t = ease(one.t);
      const x = from[0] + (to[0] - from[0]) * t;
      const y = from[1] + (to[1] - from[1]) * t;
      const colour = colourOf(one.to);
      brush.lineWidth = 1.5 * ratio;
      brush.strokeStyle = rgba(colour, 0.35);
      brush.beginPath();
      one.trail.forEach((node, n) => {
        const at = spot(node);
        if (!at) return;
        if (n === 0) brush.moveTo(at[0], at[1]);
        else brush.lineTo(at[0], at[1]);
      });
      brush.lineTo(x, y);
      brush.stroke();
      brush.fillStyle = rgba(colour, 0.35);
      brush.beginPath();
      brush.arc(x, y, 6 * ratio, 0, 2 * Math.PI);
      brush.fill();
      brush.fillStyle = "rgba(255, 255, 255, 0.95)";
      brush.beginPath();
      brush.arc(x, y, 2.5 * ratio, 0, 2 * Math.PI);
      brush.fill();
    }
    for (const flash of flashes) {
      const at = spot(flash.node);
      if (!at) continue;
      const grown = flash.age / FLASH_SECONDS;
      brush.lineWidth = 2 * ratio;
      brush.strokeStyle = rgba(colourOf(flash.node), 1 - grown);
      brush.beginPath();
      brush.arc(at[0], at[1], (6 + 18 * grown) * ratio, 0, 2 * Math.PI);
      brush.stroke();
    }
  };
  const noted = (ms: number) => {
    counted++;
    const now = clock.now();
    if (now - countedFrom >= 1000) {
      fps = (counted * 1000) / (now - countedFrom);
      counted = 0;
      countedFrom = now;
    }
    spent = spent * 0.9 + ms * 0.1;
    if (frames++ % MEASURED_EVERY === 0) container.dataset.livingMs = spent.toFixed(2);
  };
  const measured = () => {
    painted();
    if (rendering === 0) return;
    noted(ticked + clock.now() - rendering);
    rendering = 0;
  };
  const unhook = drawing.onFrame(measured);
  const unheard = drawing.beforeFrame(() => {
    if (frame !== 0) rendering = clock.now();
  });
  const moved = (event: PointerEvent) => {
    const box = container.getBoundingClientRect();
    pointer = [event.clientX - box.left, event.clientY - box.top];
  };
  const leave = () => {
    pointer = null;
  };
  // The globe rests only under the pointer on the network itself, so the empty corners of the panel let it turn.
  const onShape = (): boolean => {
    if (!pointer) return false;
    const [px, py] = pointer;
    for (let node = 0; node < network.nodes.length; node++) {
      if (hidden[node]) continue;
      const at = drawing.nodeOnScreen(node);
      if (at && Math.abs(at[0] - px) < SHAPE_REACH_PX && Math.abs(at[1] - py) < SHAPE_REACH_PX)
        return true;
    }
    return false;
  };
  const zoom = () => {
    zoomedAt = clock.now();
  };
  container.addEventListener("pointermove", moved);
  container.addEventListener("pointerleave", leave);
  container.addEventListener("wheel", zoom, { passive: true });
  const still = () => held || clock.now() - zoomedAt < ZOOM_REST_MS || onShape();

  const tick = (at: number) => {
    const started = clock.now();
    const seconds = Math.min(LONGEST_STEP, Math.max(0, (at - last) / 1000));
    last = at;
    if (spinning && !still())
      turn = { ...turn, yaw: turn.yaw + (seconds * 2 * Math.PI) / TURN_SECONDS };
    const fresh: Flash[] = [];
    if (travelling && travellers.length === 0)
      travellers = Array.from({ length: many }, () => setOut(world.start()));
    if (travelling)
      travellers = travellers.map((one) => stepped(world, one, seconds, HOP_SECONDS, fresh));
    flashes = [
      ...flashes
        .map((one) => ({ ...one, age: one.age + seconds }))
        .filter((one) => one.age < FLASH_SECONDS),
      ...fresh,
    ];
    if (spinning) {
      drawing.turn(turn);
      ticked = clock.now() - started;
    } else {
      painted();
      noted(clock.now() - started);
    }
    frame = clock.frame(tick);
  };
  const running = () =>
    laid && (travelling || spinning) && container.ownerDocument.visibilityState !== "hidden";
  const run = () => {
    if (frame !== 0 || !running()) return;
    last = clock.now();
    countedFrom = last;
    counted = 0;
    frame = clock.frame(tick);
  };
  const halt = () => {
    clock.cancel(frame);
    frame = 0;
  };
  const seen = () => (running() ? run() : halt());
  container.ownerDocument.addEventListener("visibilitychange", seen);

  return {
    ready: () => {
      laid = true;
      seen();
    },
    travel: (on) => {
      travelling = on;
      if (!on) {
        travellers = [];
        flashes = [];
        painted();
      }
      seen();
    },
    spin: (on) => {
      spinning = on;
      if (!on) {
        turn = { yaw: 0, pitch: 0 };
        drawing.turn(null);
      }
      seen();
    },
    measured: () => ({ ms: frame === 0 ? 0 : spent, fps: frame === 0 ? 0 : fps }),
    turnBy: (yaw, pitch) => {
      if (!laid || !spinning) return;
      turn = {
        yaw: turn.yaw + yaw,
        pitch: Math.max(-PITCH_AT_MOST, Math.min(PITCH_AT_MOST, turn.pitch + pitch)),
      };
      drawing.turn(turn);
    },
    hold: (on) => {
      held = on;
    },
    sift: (next) => {
      hidden = next;
      travellers = travellers.map((one) => (hidden[one.to] ? setOut(world.start()) : one));
    },
    stop: () => {
      halt();
      container.ownerDocument.removeEventListener("visibilitychange", seen);
      container.removeEventListener("pointermove", moved);
      container.removeEventListener("pointerleave", leave);
      container.removeEventListener("wheel", zoom);
      unheard();
      unhook();
      canvas.remove();
    },
  };
}
