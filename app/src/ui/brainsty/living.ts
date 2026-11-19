import type { Network } from "../../brainsty";
import type { Drawing } from "./drawing";
import { PITCH_AT_MOST, type Turn } from "./globe";
import { laidEdges } from "./grouping";
import { bigOnes, degreesOf, rgba } from "./looks";
import { fitted, LAID } from "./overlays";
import { ease, type Flash, setOut, stepped, type Traveller, type World } from "./travellers";

export const TRAVELLERS = 12;
export const HOP_SECONDS = 4;
const TURN_SECONDS = 120;
const FLASH_SECONDS = 0.6;
const LONGEST_STEP = 0.1;

export interface Living {
  ready: () => void;
  play: () => void;
  pause: () => void;
  turnBy: (yaw: number, pitch: number) => void;
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
  clock: Clock = CLOCK,
): Living {
  const canvas = container.ownerDocument.createElement("canvas");
  Object.assign(canvas.style, LAID);
  container.append(canvas);
  const near = neighboursOf(network);
  const big = bigOnes(degreesOf(network.nodes.length, network.edges));
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
  let playing = false;
  let frame = 0;
  let last = 0;
  let spent = 0;

  const painted = () => {
    const brush = canvas.getContext("2d");
    if (!brush) return;
    fitted(canvas, container);
    brush.clearRect(0, 0, canvas.width, canvas.height);
    const ratio = canvas.width / Math.max(container.clientWidth, 1);
    const spot = (node: number) => {
      const at = drawing.nodeOnScreen(node);
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
  const unhook = drawing.onFrame(painted);

  const tick = (at: number) => {
    const started = clock.now();
    const seconds = Math.min(LONGEST_STEP, Math.max(0, (at - last) / 1000));
    last = at;
    turn = { ...turn, yaw: turn.yaw + (seconds * 2 * Math.PI) / TURN_SECONDS };
    const fresh: Flash[] = [];
    travellers = travellers.map((one) => stepped(world, one, seconds, HOP_SECONDS, fresh));
    flashes = [
      ...flashes
        .map((one) => ({ ...one, age: one.age + seconds }))
        .filter((one) => one.age < FLASH_SECONDS),
      ...fresh,
    ];
    drawing.turn(turn);
    spent = spent * 0.9 + (clock.now() - started) * 0.1;
    container.dataset.livingMs = spent.toFixed(2);
    frame = clock.frame(tick);
  };
  const running = () => laid && playing && container.ownerDocument.visibilityState !== "hidden";
  const run = () => {
    if (frame !== 0 || !running()) return;
    if (travellers.length === 0)
      travellers = Array.from({ length: TRAVELLERS }, () => setOut(world.start()));
    last = clock.now();
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
    play: () => {
      playing = true;
      seen();
    },
    pause: () => {
      playing = false;
      halt();
    },
    turnBy: (yaw, pitch) => {
      if (!laid) return;
      turn = {
        yaw: turn.yaw + yaw,
        pitch: Math.max(-PITCH_AT_MOST, Math.min(PITCH_AT_MOST, turn.pitch + pitch)),
      };
      drawing.turn(turn);
    },
    sift: (next) => {
      hidden = next;
      travellers = travellers.map((one) => (hidden[one.to] ? setOut(world.start()) : one));
    },
    stop: () => {
      halt();
      container.ownerDocument.removeEventListener("visibilitychange", seen);
      unhook();
      canvas.remove();
    },
  };
}
