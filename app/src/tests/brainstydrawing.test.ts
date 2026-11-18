import type Graph from "graphology";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Network } from "../brainsty";
import { drawNetwork } from "../ui/brainsty/drawing";
import { type Palette, paletteFrom } from "../ui/brainsty/looks";

type Styled = (
  attributes: Record<string, unknown>,
  state: Record<string, unknown>,
  shared: { zoom: number },
) => unknown;

const drawn = vi.hoisted(() => ({
  graph: null as unknown,
  options: null as unknown as {
    customGraphState: { zoom: number };
    primitives: { nodes: { shapes: { name: string }[] }; edges: { layers: { name: string }[] } };
    styles: { nodes: Record<string, unknown>[]; edges: Record<string, unknown>[] };
    settings: Record<string, unknown>;
  },
  camera: {
    ratio: 1,
    heard: [] as (() => void)[],
    set: [] as unknown[],
    setState(state: unknown) {
      this.set.push(state);
    },
    on(_: string, listener: () => void) {
      this.heard.push(listener);
    },
    off(_: string, listener: () => void) {
      this.heard = this.heard.filter((one) => one !== listener);
    },
  },
  shared: [] as unknown[],
  frames: [] as (() => void)[],
  refreshed: 0,
  killed: 0,
  shown: true,
}));

vi.mock("sigma", () => ({
  DEFAULT_STYLES: { nodes: { base: true }, edges: { base: true } },
  default: class {
    constructor(graph: unknown, _container: HTMLElement, options: typeof drawn.options) {
      drawn.graph = graph;
      drawn.options = options;
    }
    getNodeDisplayData() {
      return drawn.shown ? { x: 0.25, y: 0.75 } : undefined;
    }
    getCamera() {
      return drawn.camera;
    }
    setGraphState(state: unknown) {
      drawn.shared.push(state);
    }
    graphToViewport({ x, y }: { x: number; y: number }) {
      return { x: x * 10, y: y * 10 };
    }
    on(_: string, listener: () => void) {
      drawn.frames.push(listener);
    }
    off(_: string, listener: () => void) {
      drawn.frames = drawn.frames.filter((one) => one !== listener);
    }
    refresh() {
      drawn.refreshed++;
    }
    kill() {
      drawn.killed++;
    }
  },
}));

vi.mock("sigma/rendering", () => ({
  sdfCircle: () => ({ name: "circle" }),
  sdfSquare: () => ({ name: "square" }),
  layerFill: () => ({ name: "fill" }),
  layerPlain: (options: unknown) => ({ name: "plain", options }),
  layerDashed: (options: unknown) => ({ name: "dashed", options }),
  pathLine: () => ({ name: "line" }),
}));

const light: Palette = paletteFrom((name) => (name === "--tisty-ink" ? "#000000" : "#102030"));
const dark: Palette = paletteFrom((name) => (name === "--tisty-ink" ? "#ffffff" : "#405060"));

const network: Network = {
  nodes: [
    { id: "notes.md", kind: "doc", title: "Notes", group: "f1", state: "open" },
    { id: "01A", kind: "task", title: "Bread", state: "done" },
    { id: "#home", kind: "tag", title: "home" },
  ],
  edges: [
    [0, 1, 0],
    [0, 1, 1],
    [1, 2, 3],
    [0, 1, 9],
  ],
};

const spots = { x: Float32Array.of(1, 2, 3), y: Float32Array.of(3, 4, 5) };

const nodeStyle = (name: string) => drawn.options.styles.nodes[1][name] as Styled;
const edgeStyle = (name: string) => drawn.options.styles.edges[1][name] as Styled;

beforeEach(() => {
  drawn.refreshed = 0;
  drawn.killed = 0;
  drawn.shown = true;
  drawn.shared = [];
  drawn.frames = [];
  drawn.camera.ratio = 1;
  drawn.camera.heard = [];
  drawn.camera.set = [];
});

describe("the drawing of the network", () => {
  it("draws each node with its shape, colour and weight, and each known line with its own layer", () => {
    drawNetwork(
      document.body,
      network,
      spots,
      (group) => (group ? "blue" : undefined),
      light,
      "Inter",
    );
    const graph = drawn.graph as Graph;

    expect(graph.getNodeAttributes("0")).toMatchObject({
      x: 1,
      y: 3,
      label: "Notes",
      shape: "square",
    });
    expect(graph.getNodeAttributes("1")).toMatchObject({ shape: "circle" });
    expect(graph.getNodeAttribute("2", "label")).toBe("#home");
    expect(graph.size).toBe(3);
    const [cite, mention] = graph.edges().map((one) => graph.getEdgeAttributes(one));
    expect(cite).toMatchObject({ tie: "cite", gap: 0 });
    expect(mention.tie).toBe("mention");
    expect(mention.gap).toBeGreaterThan(0);
    expect(drawn.options.primitives.nodes.shapes.map((one) => one.name)).toEqual([
      "circle",
      "square",
    ]);
    expect(drawn.options.primitives.edges.layers.map((one) => one.name)).toEqual([
      "plain",
      "dashed",
    ]);
    expect(drawn.options.settings.allowInvalidContainer).toBe(true);
    expect(drawn.options.customGraphState).toEqual({ zoom: 0 });
  });

  it("hides every title from afar, shows the big ones at middle distance and the rest up close", () => {
    drawNetwork(document.body, network, spots, () => undefined, light, "Inter");
    const shown = nodeStyle("labelVisibility");
    const colour = nodeStyle("labelColor");
    const big = { big: true };
    const small = { big: false };

    expect(shown(big, {}, { zoom: 0 })).toBe("hidden");
    expect(shown(big, { isHovered: true }, { zoom: 0 })).toBe("visible");
    expect(shown(big, {}, { zoom: 0.4 })).toBe("visible");
    expect(shown(small, {}, { zoom: 0.4 })).toBe("hidden");
    expect(shown(small, {}, { zoom: 1 })).toBe("auto");
    expect(colour(small, {}, { zoom: 1 })).toBe("rgba(0, 0, 0, 1)");
    expect(colour(big, {}, { zoom: 0.25 })).toBe("rgba(0, 0, 0, 0.5)");
  });

  it("draws lines plain and faint from afar and in their own kind up close", () => {
    drawNetwork(document.body, network, spots, () => undefined, light, "Inter");
    const graph = drawn.graph as Graph;
    const mention = graph.getEdgeAttributes(graph.edges()[1]);
    const colour = edgeStyle("color");
    const gap = edgeStyle("gap");
    const solid = edgeStyle("solid");
    const size = edgeStyle("size");

    expect(gap(mention, {}, { zoom: 0 })).toBe(0);
    expect(solid(mention, {}, { zoom: 0 })).toBe(colour(mention, {}, { zoom: 0 }));
    expect(gap(mention, {}, { zoom: 1 })).toBe(mention.gap);
    expect(solid(mention, {}, { zoom: 1 })).toBe("rgba(0, 0, 0, 0)");
    expect(colour(mention, {}, { zoom: 0 })).toBe("rgba(16, 32, 48, 0.175)");
    expect(colour(mention, {}, { zoom: 1 })).toBe("rgba(16, 32, 48, 0.5)");
    expect(size(mention, {}, { zoom: 0 })).toBeLessThan(size(mention, {}, { zoom: 1 }) as number);
  });

  it("tells sigma the zoom only when it crosses a step", () => {
    drawNetwork(document.body, network, spots, () => undefined, light, "Inter");
    const zoomed = drawn.camera.heard[0];

    drawn.camera.ratio = 0.999;
    zoomed();
    drawn.camera.ratio = 0.3;
    zoomed();

    expect(drawn.shared).toHaveLength(2);
    expect((drawn.shared[1] as { zoom: number }).zoom).toBeCloseTo(0.475);
  });

  it("moves the nodes, follows the theme, maps places to the screen and lets go when killed", () => {
    const drawing = drawNetwork(document.body, network, spots, () => undefined, light, "Inter");
    const graph = drawn.graph as Graph;
    const colour = nodeStyle("labelColor");

    drawing.place({ x: Float32Array.of(5, 6, 7), y: Float32Array.of(7, 8, 9) });
    drawing.place({ x: new Float32Array(), y: new Float32Array() });
    expect(graph.getNodeAttributes("1")).toMatchObject({ x: 6, y: 8 });
    expect(drawing.nodeOnScreen(1)).toEqual([60, 80]);
    expect(drawing.nodeOnScreen(9)).toBeUndefined();
    expect(drawing.toScreen([1, 2])).toEqual([10, 20]);
    expect(drawing.zoom()).toBe(0);

    const before = graph.getNodeAttribute("0", "color");
    drawing.retheme(dark);
    expect(graph.getNodeAttribute("0", "color")).not.toBe(before);
    expect(colour({ big: false }, {}, { zoom: 1 })).toBe("rgba(255, 255, 255, 1)");
    expect(drawn.refreshed).toBe(1);

    const frame = vi.fn();
    const unhook = drawing.onFrame(frame);
    for (const one of drawn.frames) one();
    unhook();
    expect(frame).toHaveBeenCalledTimes(1);
    expect(drawn.frames).toHaveLength(0);

    drawing.kill();
    expect(drawn.killed).toBe(1);
    expect(drawn.camera.heard).toHaveLength(0);
  });

  it("centres the camera on a node it can see, and leaves it be otherwise", () => {
    const drawing = drawNetwork(document.body, network, spots, () => undefined, light, "Inter");

    drawing.centre(1);
    drawn.shown = false;
    drawing.centre(5);

    expect(drawn.camera.set).toEqual([expect.objectContaining({ x: 0.25, y: 0.75 })]);
  });
});
