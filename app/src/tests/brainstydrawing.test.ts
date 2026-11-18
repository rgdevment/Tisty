import type Graph from "graphology";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Network } from "../brainsty";
import { drawNetwork } from "../ui/brainsty/drawing";
import { type Palette, paletteFrom } from "../ui/brainsty/looks";

const drawn = vi.hoisted(() => ({
  graph: null as unknown,
  options: null as unknown as {
    primitives: { nodes: { shapes: { name: string }[] }; edges: { layers: { name: string }[] } };
    styles: { nodes: unknown[] };
    settings: Record<string, unknown>;
  },
  camera: { setState: (_: unknown) => {} },
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
  ],
  edges: [
    [0, 1, 0],
    [0, 1, 1],
    [0, 1, 9],
  ],
};

const spots = { x: Float32Array.of(1, 2), y: Float32Array.of(3, 4) };

beforeEach(() => {
  drawn.refreshed = 0;
  drawn.killed = 0;
  drawn.shown = true;
});

describe("the drawing of the network", () => {
  it("draws each node with its shape and colour, and each known line with its own layer", () => {
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
    expect(graph.size).toBe(2);
    const [cite, mention] = graph.edges().map((one) => graph.getEdgeAttributes(one));
    expect(cite).toMatchObject({ tie: "cite", gap: 0 });
    expect(cite.solid).toBe(cite.color);
    expect(mention).toMatchObject({ tie: "mention", solid: "rgba(0, 0, 0, 0)" });
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
  });

  it("moves the nodes, follows the theme and lets go of WebGL when killed", () => {
    const drawing = drawNetwork(document.body, network, spots, () => undefined, light, "Inter");
    const graph = drawn.graph as Graph;
    const labelColor = (drawn.options.styles.nodes[1] as { labelColor: () => string }).labelColor;

    drawing.place({ x: Float32Array.of(5, 6), y: Float32Array.of(7, 8) });
    drawing.place({ x: new Float32Array(), y: new Float32Array() });
    expect(graph.getNodeAttributes("1")).toMatchObject({ x: 6, y: 8 });
    expect(labelColor()).toBe("#000000");

    const before = graph.getNodeAttribute("0", "color");
    drawing.retheme(dark);
    expect(graph.getNodeAttribute("0", "color")).not.toBe(before);
    expect(labelColor()).toBe("#ffffff");
    expect(drawn.refreshed).toBe(1);

    drawing.kill();
    expect(drawn.killed).toBe(1);
  });

  it("centres the camera on a node it can see, and leaves it be otherwise", () => {
    const setState = vi.fn();
    drawn.camera = { setState };
    const drawing = drawNetwork(document.body, network, spots, () => undefined, light, "Inter");

    drawing.centre(1);
    drawn.shown = false;
    drawing.centre(5);

    expect(setState).toHaveBeenCalledTimes(1);
    expect(setState).toHaveBeenCalledWith(expect.objectContaining({ x: 0.25, y: 0.75 }));
  });
});
