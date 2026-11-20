import { describe, expect, it, vi } from "vitest";
import type { Neuron } from "../brainsty";
import { layOut } from "../ui/brainsty/layingOut";
import {
  edgesOf,
  forcesFor,
  graphOf,
  movedBy,
  type Step,
  seeded,
  settle,
  spotsOf,
  weightsOf,
} from "../ui/brainsty/settling";

const neuron = (id: string, group?: string): Neuron => ({ id, kind: "task", title: id, group });

describe("where the network starts and how it settles", () => {
  it("starts each node in the same place every time, and each zone apart from the others", () => {
    const nodes = [neuron("a", "home"), neuron("b", "home"), neuron("c", "work"), neuron("#t")];

    const first = seeded(nodes);
    const again = seeded(nodes);

    expect([...first.x]).toEqual([...again.x]);
    expect([...first.y]).toEqual([...again.y]);
    const home = Math.hypot(first.x[0] - first.x[1], first.y[0] - first.y[1]);
    const apart = Math.hypot(first.x[0] - first.x[2], first.y[0] - first.y[2]);
    expect(home).toBeLessThan(apart);
    expect(Math.hypot(first.x[3], first.y[3])).toBeLessThan(apart);
  });

  it("keeps the library's forces for a small network and trades Barnes–Hut detail for iterations in a big one", () => {
    expect(forcesFor(1500)).toMatchObject({
      barnesHutOptimize: false,
      slowDown: 1 + Math.log(1500),
    });
    expect(forcesFor(1500)).not.toHaveProperty("barnesHutTheta");
    expect(forcesFor(12000)).toMatchObject({
      barnesHutOptimize: true,
      barnesHutTheta: 1.2,
      slowDown: 3,
      gravity: 1,
      scalingRatio: 10,
    });
  });

  it("flattens the lines into pairs and builds a graph that skips loops and repeats", () => {
    const edges = edgesOf([
      [0, 1, 0],
      [1, 0, 1],
      [2, 2, 2],
      [0, 7, 0],
    ]);
    expect([...edges]).toEqual([0, 1, 1, 0, 2, 2, 0, 7]);

    const graph = graphOf({ x: Float32Array.of(0, 1, 2), y: Float32Array.of(0, 1, 2), edges });

    expect(graph.order).toBe(3);
    expect(graph.size).toBe(1);
    expect([...spotsOf(graph).x]).toEqual([0, 1, 2]);
  });

  it("pulls hardest along what was written, barely along a tag, and keeps the strongest of two lines", () => {
    const ties: [number, number, number][] = [
      [0, 1, 3],
      [1, 0, 0],
      [1, 2, 4],
      [0, 2, 3],
    ];
    const weights = weightsOf(ties);
    expect([...weights].map((one) => Math.round(one * 10) / 10)).toEqual([0.3, 3, 1.5, 0.3]);

    const graph = graphOf({
      x: Float32Array.of(0, 1, 2),
      y: Float32Array.of(0, 1, 2),
      edges: edgesOf(ties),
      weights,
    });

    expect(graph.size).toBe(3);
    expect(graph.getEdgeAttribute(graph.edge("0", "1"), "weight")).toBe(3);
    expect(graph.getEdgeAttribute(graph.edge("1", "2"), "weight")).toBe(1.5);
    expect(Math.round(graph.getEdgeAttribute(graph.edge("0", "2"), "weight") * 10) / 10).toBe(0.3);
  });

  it("measures a batch's movement against the network's width", () => {
    const before = { x: Float32Array.of(0, 10), y: Float32Array.of(0, 0) };
    const after = { x: Float32Array.of(1, 11), y: Float32Array.of(0, 0) };

    expect(movedBy(before, after)).toBeCloseTo(0.1);
    expect(movedBy(before, { x: new Float32Array(), y: new Float32Array() })).toBe(0);
  });

  it("tells each batch while it settles and ends with the time it took", () => {
    const nodes = Array.from({ length: 30 }, (_, n) => neuron(`n${n}`, `g${n % 3}`));
    const lines = nodes.slice(1).map((_, n): [number, number, number] => [n, n + 1, 0]);
    const steps: Step[] = [];
    let clock = 0;

    settle(
      { ...seeded(nodes), edges: edgesOf(lines) },
      (step) => steps.push(step),
      () => clock++,
    );

    const last = steps[steps.length - 1];
    expect(last.done).toBe(true);
    expect(steps.slice(0, -1).every((one) => !one.done)).toBe(true);
    expect(last.iterations).toBeGreaterThan(0);
    expect(last.ms).toBeGreaterThan(0);
    expect(last.x).toHaveLength(30);
  });

  it("spaces the steps it tells by the size of the network, and still ends with the last", () => {
    const nodes = Array.from({ length: 60 }, (_, n) => neuron(`n${n}`, `g${n % 3}`));
    const lines = nodes.slice(1).map((_, n): [number, number, number] => [n, n + 1, 0]);
    const steps: Step[] = [];
    let clock = 0;

    settle(
      { ...seeded(nodes), edges: edgesOf(lines) },
      (step) => steps.push(step),
      () => {
        clock += 0.5;
        return clock;
      },
    );

    const told = steps.filter((one) => !one.done).map((one) => one.iterations);
    expect(told.length).toBeGreaterThan(1);
    expect(told.every((one) => one % 20 === 0)).toBe(true);
    expect(steps[steps.length - 1].done).toBe(true);
  });

  it("stops once its time is spent", () => {
    const nodes = Array.from({ length: 30 }, (_, n) => neuron(`n${n}`));
    const lines = nodes.slice(1).map((_, n): [number, number, number] => [0, n + 1, 0]);
    const steps: Step[] = [];
    let clock = 0;

    settle(
      { ...seeded(nodes), edges: edgesOf(lines) },
      (step) => steps.push(step),
      () => {
        clock += 3000;
        return clock;
      },
    );

    expect(steps.map((one) => one.iterations)).toEqual([10, 10]);
  });

  it("leaves a network without lines where it started", () => {
    const steps: Step[] = [];
    const spots = seeded([neuron("a"), neuron("b")]);

    settle({ ...spots, edges: new Uint32Array() }, (step) => steps.push(step));

    expect(steps).toHaveLength(1);
    expect(steps[0]).toMatchObject({ done: true, iterations: 0 });
  });

  it("hands the work to a worker, passes its steps on and lets it go when done or stopped", () => {
    const worker = {
      onmessage: null as ((heard: MessageEvent<Step>) => void) | null,
      postMessage: vi.fn(),
      terminate: vi.fn(),
    };
    const heard: Step[] = [];
    const asked = { ...seeded([neuron("a")]), edges: new Uint32Array() };

    const stop = layOut(asked, (step) => heard.push(step), worker as unknown as Worker);
    const step = (done: boolean): Step => ({ x: asked.x, y: asked.y, done, iterations: 1, ms: 1 });
    worker.onmessage?.({ data: step(false) } as MessageEvent<Step>);
    expect(worker.terminate).not.toHaveBeenCalled();
    worker.onmessage?.({ data: step(true) } as MessageEvent<Step>);
    stop();

    expect(worker.postMessage).toHaveBeenCalledWith(asked, [
      asked.x.buffer,
      asked.y.buffer,
      asked.edges.buffer,
    ]);
    expect(heard.map((one) => one.done)).toEqual([false, true]);
    expect(worker.terminate).toHaveBeenCalledTimes(2);
  });

  it("ends the layout where it stands when the worker fails", () => {
    const worker = {
      onmessage: null,
      onerror: null as null | (() => void),
      postMessage: vi.fn(),
      terminate: vi.fn(),
    };
    const heard: Step[] = [];

    layOut(
      { ...seeded([neuron("a")]), edges: new Uint32Array() },
      (step) => heard.push(step),
      worker as unknown as Worker,
    );
    worker.onerror?.();

    expect(worker.terminate).toHaveBeenCalledTimes(1);
    expect(heard).toEqual([expect.objectContaining({ done: true, iterations: 0 })]);
    expect(heard[0].x).toHaveLength(0);
  });
});
