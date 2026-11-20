import { afterEach, describe, expect, it, vi } from "vitest";
import type { Network } from "../brainsty";
import type { Drawing } from "../ui/brainsty/drawing";
import {
  depthOf,
  FRONT,
  hashOf,
  PITCH_AT_MOST,
  radiusOf,
  rotated,
  sewnInto,
  spinning,
  trigOf,
} from "../ui/brainsty/globe";
import { type Clock, HOP_SECONDS, livingOn, travellersFor } from "../ui/brainsty/living";
import { type Flash, setOut, stepped, type World } from "../ui/brainsty/travellers";
import { turnedByHand } from "../ui/brainsty/turning";

const program = (named: boolean) => ({ named });

const glOf = () => {
  const set: unknown[][] = [];
  const gl = {
    used: 0,
    useProgram(_: unknown) {
      gl.used++;
    },
    getUniformLocation: (one: { named: boolean }, name: string) => (one.named ? name : null),
    uniform4f: (...args: unknown[]) => set.push(args),
  };
  return { gl: gl as unknown as WebGL2RenderingContext, set };
};

describe("the globe's turn, worked out the way the shader does", () => {
  it("hashes each node to the same depth every time, within the sphere", () => {
    expect(hashOf(7)).toBe(hashOf(7));
    expect(hashOf(7)).not.toBe(hashOf(8));
    for (let n = 0; n < 200; n++) expect(Math.abs(hashOf(n))).toBeLessThanOrEqual(1);
    expect(Math.abs(depthOf(0.6, 0.5, 3, 0.4))).toBeLessThanOrEqual(Math.sqrt(0.16 - 0.01));
    expect(depthOf(0.9, 0.9, 3, 0.1)).toBe(0);
    expect(
      radiusOf([
        { x: 0.5, y: 0.5 },
        { x: 0.8, y: 0.9 },
      ]),
    ).toBeCloseTo(0.5);
  });

  it("brings a node at the front forward and mirrors it half a turn later", () => {
    const front = rotated(0.7, 0.5, 0, 0.4, trigOf(FRONT));
    const half = rotated(0.7, 0.5, 0, 0.4, trigOf({ yaw: Math.PI, pitch: 0 }));

    expect(front).toEqual({ x: 0.7, y: 0.5, near: 0.5 });
    expect(half.x).toBeCloseTo(0.3);
    expect(rotated(0.5, 0.5, 0.4, 0.4, trigOf(FRONT)).near).toBe(1);
  });
});

describe("the turn sewn into sigma's programs", () => {
  it("tells each program the turn once per change, and turns off cleanly", () => {
    const { gl, set } = glOf();
    const sewn = sewnInto(gl);
    const one = program(true);

    gl.useProgram(one as never);
    gl.useProgram(one as never);
    sewn.set({ yaw: 0, pitch: 0 }, 0.4);
    gl.useProgram(one as never);
    sewn.set(null, 0.4);
    gl.useProgram(one as never);

    expect(sewn.sewn()).toBe(true);
    expect(set).toEqual([
      ["u_globeShape", 0, 0, 0, 0],
      ["u_globeShape", 1, 0.4, 0, 0],
      ["u_globeTurn", 1, 0, 1, 0],
      ["u_globeShape", 0, 0.4, 0, 0],
    ]);
  });

  it("knows when sigma's shaders came without the turn, or there is no WebGL", () => {
    const { gl } = glOf();
    const sewn = sewnInto(gl);
    gl.useProgram(program(false) as never);
    gl.useProgram(null as never);

    expect(sewn.sewn()).toBe(false);
    expect(sewnInto(null).sewn()).toBe(false);
  });

  it("places what is drawn over a node on the turned node, and flat until sigma uses the turn", () => {
    const { gl } = glOf();
    const rendered = vi.fn();
    const places: Record<string, { x: number; y: number }> = {
      "0": { x: 0.7, y: 0.5 },
      "1": { x: 0.3, y: 0.5 },
    };
    const sigma = {
      webGLContext: gl,
      getNodeDisplayData: (key: string) => places[key],
      getNodeDataTexture: () => ({ getIndex: (key: string) => Number(key) }),
      scheduleRender: rendered,
    };
    const spin = spinning(sigma, () => ["0", "1"]);

    spin.turn({ yaw: Math.PI, pitch: 0 });
    expect(spin.turned()).toBe(false);
    expect(spin.framed("0")).toEqual(places["0"]);
    gl.useProgram(program(true) as never);

    expect(spin.turned()).toBe(true);
    expect(spin.framed("0")?.x).toBeLessThan(0.5);
    expect(spin.framed("9")).toBeUndefined();
    spin.moved();
    spin.turn(null);
    expect(spin.turned()).toBe(false);
    expect(rendered).toHaveBeenCalledTimes(2);
  });
});

const chain: Network = {
  nodes: [
    { id: "a", kind: "doc", title: "A" },
    { id: "b", kind: "doc", title: "B" },
    { id: "#t", kind: "tag", title: "t" },
    { id: "c", kind: "task", title: "C" },
    { id: "lonely", kind: "doc", title: "Lonely" },
  ],
  edges: [
    [0, 1, 0],
    [1, 2, 3],
    [2, 3, 3],
  ],
};

const near: Record<number, number[]> = { 0: [1], 1: [0, 2], 2: [1, 3], 3: [2], 4: [] };

const worldOf = (seen = true, start = 0): World => ({
  neighbours: (node) => near[node],
  kindOf: (node) => chain.nodes[node].kind,
  big: () => false,
  flashy: (node) => chain.nodes[node].kind === "tag",
  seen: () => seen,
  start: () => start,
  rand: () => 0.5,
});

const walked = (world: World, from: number, steps: number, flashes: Flash[] = []) => {
  let one = setOut(from);
  const path = [from];
  for (let n = 0; n < steps; n++) {
    const next = stepped(world, one, 1, 1, flashes);
    if (next.to !== one.to) path.push(next.to);
    one = next;
  }
  return { one, path };
};

describe("a traveller", () => {
  it("walks real lines one hop at a time, flashing on a tag and lingering there", () => {
    const flashes: Flash[] = [];
    const { path } = walked(worldOf(), 0, 8, flashes);

    expect(path.slice(0, 4)).toEqual([0, 1, 2, 3]);
    expect(flashes.map((one) => one.node)).toEqual([2]);
  });

  it("goes back from a dead end, and sets out again from a node with no line", () => {
    expect(walked(worldOf(), 3, 4).path.slice(0, 2)).toEqual([3, 2]);
    expect(walked(worldOf(true, 1), 4, 1).one.to).toBe(1);
  });

  it("sets out again after a few hops out of sight", () => {
    const { one } = walked(worldOf(false, 3), 0, 60);
    expect(one.trail.length).toBeLessThanOrEqual(6);
    expect(one.away).toBeLessThan(12);
  });
});

const fakeClock = () => {
  const frames: ((at: number) => void)[] = [];
  let at = 0;
  const clock: Clock = {
    now: () => at,
    frame: (step) => {
      frames.push(step);
      return frames.length;
    },
    cancel: vi.fn(),
  };
  const tick = (ms: number) => {
    at += ms;
    frames.shift()?.(at);
  };
  return { clock, frames, tick };
};

const drawingOf = () => {
  const turns: { yaw: number; pitch: number }[] = [];
  const camera = { zoom: 0 };
  const frames: (() => void)[] = [];
  const before: (() => void)[] = [];
  const drawing = {
    nodeOnScreen: (node: number) => [10 + node * 30, 20] as [number, number],
    zoom: () => camera.zoom,
    onFrame: (listener: () => void) => {
      frames.push(listener);
      return () => frames.splice(frames.indexOf(listener), 1);
    },
    beforeFrame: (listener: () => void) => {
      before.push(listener);
      return () => before.splice(before.indexOf(listener), 1);
    },
    turn: (turn: { yaw: number; pitch: number }) => turns.push(turn),
  } as unknown as Drawing;
  const render = () => {
    for (const one of before) one();
    for (const one of frames) one();
  };
  return { drawing, turns, frames, render, camera };
};

const boxed = () => {
  const box = document.createElement("div");
  Object.defineProperty(box, "clientWidth", { value: 400 });
  Object.defineProperty(box, "clientHeight", { value: 300 });
  document.body.append(box);
  return box;
};

describe("the living network", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    document.body.replaceChildren();
  });

  it("waits for the layout and the person, then turns and walks one frame at a time", () => {
    const { clock, frames, tick } = fakeClock();
    const { drawing, turns, render } = drawingOf();
    const box = boxed();
    const living = livingOn(box, drawing, chain, () => "#2266ff", { clock });

    expect(frames).toHaveLength(0);
    living.ready();
    expect(frames).toHaveLength(1);
    tick(50);
    render();
    tick(50);

    expect(turns).toHaveLength(2);
    expect(turns[1].yaw).toBeGreaterThan(turns[0].yaw);
    expect(box.dataset.livingMs).toMatch(/^\d+\.\d\d$/);
    expect(HOP_SECONDS).toBe(5);
  });

  it("holds the turn while a node is picked, the pointer is on the network or the view is close, and walks on", () => {
    const { clock, tick } = fakeClock();
    const { drawing, turns, camera } = drawingOf();
    const box = boxed();
    const living = livingOn(box, drawing, chain, () => "#2266ff", { clock });
    living.ready();
    const yaws = () => turns.map((one) => one.yaw);

    living.hold(true);
    tick(50);
    living.hold(false);
    box.dispatchEvent(new PointerEvent("pointermove", { clientX: 42, clientY: 30 }));
    tick(50);
    box.dispatchEvent(new PointerEvent("pointerleave"));
    camera.zoom = 0.3;
    tick(50);
    tick(5000);
    expect(yaws()).toEqual([0, 0, 0, 0]);
    camera.zoom = 0;
    tick(50);
    expect(yaws()[4]).toBeGreaterThan(0);
    box.dispatchEvent(new PointerEvent("pointermove", { clientX: 380, clientY: 280 }));
    tick(50);
    expect(yaws()[5]).toBeGreaterThan(yaws()[4]);
  });

  it("sends out more travellers as the network grows, two to ten", () => {
    expect([1, 39, 59, 60, 99, 100, 149, 150, 249, 250, 9000].map(travellersFor)).toEqual([
      2, 2, 2, 4, 4, 6, 6, 8, 8, 10, 10,
    ]);
  });

  it("lays the globe flat without turning, walks on alone, and stops with neither, saying what a frame costs", () => {
    const { clock, frames, tick } = fakeClock();
    const { drawing, render, turns } = drawingOf();
    const living = livingOn(boxed(), drawing, chain, () => "#2266ff", { clock });
    living.ready();
    tick(16);
    render();
    expect(living.measured().ms).toBeGreaterThanOrEqual(0);

    living.spin(false);
    expect(turns[turns.length - 1]).toBeNull();
    living.turnBy(1, 0);
    expect(turns[turns.length - 1]).toBeNull();
    const flat = turns.length;
    tick(16);
    expect(turns).toHaveLength(flat);
    expect(frames).toHaveLength(1);
    expect(living.measured().fps).toBeGreaterThanOrEqual(0);

    living.travel(false);
    frames.length = 0;
    tick(16);
    expect(frames).toHaveLength(0);
    expect(living.measured()).toEqual({ ms: 0, fps: 0 });
    living.travel(true);
    expect(frames).toHaveLength(1);
  });

  it("stops asking for frames while the window is hidden, and comes back when seen again", () => {
    const { clock, frames, tick } = fakeClock();
    const { drawing } = drawingOf();
    const living = livingOn(boxed(), drawing, chain, () => "#2266ff", { clock });
    living.ready();
    expect(frames).toHaveLength(1);
    const visibility = vi.spyOn(document, "visibilityState", "get").mockReturnValue("hidden");
    document.dispatchEvent(new Event("visibilitychange"));
    frames.length = 0;
    tick(16);
    expect(frames).toHaveLength(0);
    visibility.mockReturnValue("visible");
    document.dispatchEvent(new Event("visibilitychange"));
    expect(frames).toHaveLength(1);
  });

  it("turns by hand within its tilt, paints over its frames, and leaves nothing once stopped", () => {
    const { clock, tick } = fakeClock();
    const { drawing, turns, frames } = drawingOf();
    const box = boxed();
    const brush = {
      clearRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      arc: vi.fn(),
      fill: vi.fn(),
    };
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(
      brush as unknown as CanvasRenderingContext2D,
    );
    const living = livingOn(box, drawing, chain, () => "#2266ff", { clock });

    living.turnBy(1, 0);
    expect(turns).toHaveLength(0);
    living.ready();
    living.turnBy(0.5, 5);
    expect(turns[0]).toEqual({ yaw: 0.5, pitch: PITCH_AT_MOST });
    tick(16);
    living.sift([true, false, false, false, false]);
    for (const one of frames) one();
    expect(brush.arc).toHaveBeenCalled();

    living.stop();
    expect(box.querySelector("canvas")).toBeNull();
    expect(frames).toHaveLength(0);
  });
});

describe("turning by hand", () => {
  it("takes the right button or Shift for itself and leaves a plain drag to the view", () => {
    const box = boxed();
    const inside = document.createElement("canvas");
    box.append(inside);
    const view = vi.fn();
    inside.addEventListener("pointerdown", view);
    const turnBy = vi.fn();
    const stop = turnedByHand(box, turnBy);

    inside.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true, button: 0 }));
    expect(view).toHaveBeenCalledTimes(1);
    inside.dispatchEvent(
      new PointerEvent("pointerdown", { bubbles: true, button: 2, clientX: 10, clientY: 10 }),
    );
    document.dispatchEvent(new PointerEvent("pointermove", { clientX: 20, clientY: 5 }));
    document.dispatchEvent(new PointerEvent("pointerup"));
    document.dispatchEvent(new PointerEvent("pointermove", { clientX: 90, clientY: 90 }));

    expect(view).toHaveBeenCalledTimes(1);
    expect(turnBy).toHaveBeenCalledTimes(1);
    expect(turnBy.mock.calls[0][0]).toBeCloseTo(0.08);
    expect(turnBy.mock.calls[0][1]).toBeCloseTo(-0.04);
    const menu = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
    inside.dispatchEvent(menu);
    expect(menu.defaultPrevented).toBe(true);
    stop();
  });
});
