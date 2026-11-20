import { GLOBE } from "./globeShader";

export interface Turn {
  yaw: number;
  pitch: number;
}

export interface Turned {
  x: number;
  y: number;
  near: number;
}

export const FRONT: Turn = { yaw: 0, pitch: 0 };

export const PITCH_AT_MOST = 0.7;

export function hashOf(index: number): number {
  let h = index >>> 0;
  h ^= h >>> 16;
  h = Math.imul(h, 0x7feb352d) >>> 0;
  h ^= h >>> 15;
  h = Math.imul(h, 0x846ca68b) >>> 0;
  h ^= h >>> 16;
  return ((h & 0xffff) / 65535) * 2 - 1;
}

const swellOf = (x: number, y: number): number =>
  0.55 * Math.sin(9 * x + 2 * Math.sin(5 * y)) + 0.45 * Math.cos(8 * y - 3 * x + 1.7);

export function depthOf(x: number, y: number, index: number, radius: number): number {
  const dx = x - 0.5;
  const dy = y - 0.5;
  const room = Math.sqrt(Math.max(radius * radius - dx * dx - dy * dy, 0));
  return room * Math.min(1, Math.max(-1, 0.7 * swellOf(x, y) + 0.45 * hashOf(index)));
}

export type Trig = readonly [cy: number, sy: number, cp: number, sp: number];

export const trigOf = (turn: Turn): Trig => [
  Math.cos(turn.yaw),
  Math.sin(turn.yaw),
  Math.cos(turn.pitch),
  Math.sin(turn.pitch),
];

// The same sums as the shader, so what is drawn over the network sits on the nodes the GPU turned.
export function rotated(x: number, y: number, z: number, radius: number, trig: Trig): Turned {
  const [cy, sy, cp, sp] = trig;
  const dx = x - 0.5;
  const dy = y - 0.5;
  const x1 = dx * cy + z * sy;
  const z1 = -dx * sy + z * cy;
  const y2 = dy * cp - z1 * sp;
  const z2 = dy * sp + z1 * cp;
  const depth = radius > 0 ? z2 / radius : 0;
  const grow = 1 + GLOBE.perspective * depth;
  return { x: 0.5 + x1 * grow, y: 0.5 + y2 * grow, near: 0.5 + 0.5 * depth };
}

export const radiusOf = (points: Iterable<{ x: number; y: number }>): number => {
  let radius = 0;
  for (const { x, y } of points) radius = Math.max(radius, Math.hypot(x - 0.5, y - 0.5));
  return radius;
};

export interface Sewn {
  set: (turn: Turn | null, radius: number) => void;
  sewn: () => boolean;
}

// Every sigma program reads its nodes through the patched function, so each one hears the turn when it is used.
export function sewnInto(gl: WebGL2RenderingContext | null): Sewn {
  let turn: Turn | null = null;
  let radius = 0;
  let version = 0;
  let found = false;
  if (!gl) return { set: () => undefined, sewn: () => false };
  const use = gl.useProgram.bind(gl);
  const known = new WeakMap<
    WebGLProgram,
    { turn: WebGLUniformLocation | null; shape: WebGLUniformLocation | null; at: number }
  >();
  gl.useProgram = (program: WebGLProgram | null) => {
    use(program);
    if (!program) return;
    let slot = known.get(program);
    if (!slot) {
      slot = {
        turn: gl.getUniformLocation(program, "u_globeTurn"),
        shape: gl.getUniformLocation(program, "u_globeShape"),
        at: -1,
      };
      known.set(program, slot);
    }
    if (!slot.shape || slot.at === version) return;
    found = true;
    slot.at = version;
    gl.uniform4f(slot.shape, turn ? 1 : 0, radius, 0, 0);
    if (turn && slot.turn) gl.uniform4f(slot.turn, ...trigOf(turn));
  };
  return {
    set: (next, size) => {
      turn = next;
      radius = size;
      version++;
    },
    sewn: () => found,
  };
}

export interface Spinnable {
  getNodeDisplayData: (key: string) => { x: number; y: number } | undefined;
  getNodeDataTexture: () => { getIndex: (key: string) => number | undefined };
  scheduleRender: () => void;
}

export interface Spinning {
  turn: (next: Turn | null) => void;
  turned: () => boolean;
  framed: (key: string) => { x: number; y: number } | undefined;
  nearness: (key: string) => number;
  moved: () => void;
}

interface Laid {
  slot: Map<string, number>;
  x: Float32Array;
  y: Float32Array;
  z: Float32Array;
}

// sigma copies a node's whole look each time its place is asked, so each place is read once and kept.
function laidFrom(sigma: Spinnable, keys: readonly string[]): { laid: Laid; radius: number } {
  const shown = keys.flatMap((key) => {
    const at = sigma.getNodeDisplayData(key);
    return at ? [{ key, x: at.x, y: at.y }] : [];
  });
  const radius = radiusOf(shown);
  const texture = sigma.getNodeDataTexture();
  const laid: Laid = {
    slot: new Map(shown.map((one, n) => [one.key, n])),
    x: Float32Array.from(shown, (one) => one.x),
    y: Float32Array.from(shown, (one) => one.y),
    z: Float32Array.from(shown, (one) =>
      depthOf(one.x, one.y, texture.getIndex(one.key) ?? 0, radius),
    ),
  };
  return { laid, radius };
}

export function spinning(sigma: Spinnable, keys: () => readonly string[]): Spinning {
  const gl = (sigma as unknown as { webGLContext?: WebGL2RenderingContext | null }).webGLContext;
  const sewn = sewnInto(gl ?? null);
  let turn: Turn | null = null;
  let trig = trigOf(FRONT);
  let radius = 0;
  let laid: Laid | null = null;
  const turned = () => turn !== null && laid !== null && sewn.sewn();
  return {
    turn: (next) => {
      turn = next;
      if (next) {
        if (!laid) {
          const fresh = laidFrom(sigma, keys());
          laid = fresh.laid.slot.size > 0 ? fresh.laid : null;
          radius = fresh.radius;
        }
        trig = trigOf(next);
      }
      sewn.set(next, radius);
      sigma.scheduleRender();
    },
    turned,
    framed: (key) => {
      const slot = turned() ? laid?.slot.get(key) : undefined;
      if (!laid || slot === undefined) return sigma.getNodeDisplayData(key);
      const spun = rotated(laid.x[slot], laid.y[slot], laid.z[slot], radius, trig);
      return { x: spun.x, y: spun.y };
    },
    nearness: (key) => {
      const slot = turned() ? laid?.slot.get(key) : undefined;
      if (!laid || slot === undefined) return 1;
      return rotated(laid.x[slot], laid.y[slot], laid.z[slot], radius, trig).near;
    },
    moved: () => {
      laid = null;
    },
  };
}
