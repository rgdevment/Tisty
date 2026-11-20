import { describe, expect, it, vi } from "vitest";
import type { Network, Neuron } from "../brainsty";
import type { Drawing } from "../ui/brainsty/drawing";
import { bigOnes, degreesOf, paletteFrom, weightOf } from "../ui/brainsty/looks";
import { overlaysOn } from "../ui/brainsty/overlays";
import { brainShaped, outline } from "../ui/brainsty/shape";
import { thumbsIn } from "../ui/brainsty/thumbs";
import { hullOf, paintZones, zonesOf } from "../ui/brainsty/zones";
import {
  allTitles,
  bigTitles,
  levelOf,
  lineDetail,
  quantized,
  thumbsShown,
  zonesShown,
  zoomOf,
} from "../ui/brainsty/zoom";

const neuron = (id: string, group?: string, kind: Neuron["kind"] = "task"): Neuron => ({
  id,
  kind,
  title: id,
  group,
});

const flush = () => new Promise((done) => setTimeout(done, 0));

describe("how far the camera is", () => {
  it("reads the camera's ratio as a closeness from 0, everything in view, to 1, every title", () => {
    expect(zoomOf(1)).toBe(0);
    expect(zoomOf(2)).toBe(0);
    expect(zoomOf(0)).toBe(0);
    expect(zoomOf(0.08)).toBeCloseTo(1);
    expect(zoomOf(0.01)).toBe(1);
    expect(quantized(0.4766)).toBeCloseTo(0.5);
    expect(quantized(1.4)).toBe(1);
  });

  it("names three distances and lets each detail come in smoothly at its own", () => {
    expect([0.1, 0.4, 0.8].map(levelOf)).toEqual(["far", "mid", "near"]);
    expect(bigTitles(0.1)).toBe(0);
    expect(bigTitles(0.25)).toBeCloseTo(0.5);
    expect(bigTitles(0.4)).toBe(1);
    expect(allTitles(0.4)).toBe(0);
    expect(allTitles(0.9)).toBe(1);
    expect(lineDetail(0.1)).toBe(0);
    expect(lineDetail(0.9)).toBe(1);
    expect(zonesShown(0.05)).toBe(0);
    expect(zonesShown(0.45)).toBe(1);
    expect(zonesShown(0.95)).toBe(0);
    expect(thumbsShown(0.5)).toBe(false);
    expect(thumbsShown(0.8)).toBe(true);
  });
});

describe("the brain's silhouette", () => {
  it("is wider than it is tall and nicked under the front", () => {
    expect(outline(0)).toBeGreaterThan(outline(Math.PI / 2));
    expect(outline(Math.PI + 0.62)).toBeLessThan(outline(Math.PI + 0.3));
  });

  it("keeps each node's direction and distance order while filling the outline", () => {
    const spots = {
      x: Float32Array.of(0, 10, -10, 0, 0, 5),
      y: Float32Array.of(0, 0, 0, 10, -10, 5),
    };

    const shaped = brainShaped(spots);

    expect(shaped.x).toHaveLength(6);
    expect(Math.sign(shaped.x[1] - shaped.x[0])).toBe(1);
    expect(Math.sign(shaped.x[2] - shaped.x[0])).toBe(-1);
    expect(Math.sign(shaped.y[3] - shaped.y[0])).toBe(1);
    const near = Math.hypot(shaped.x[5] - shaped.x[0], shaped.y[5] - shaped.y[0]);
    expect(near).toBeLessThan(Math.hypot(shaped.x[1] - shaped.x[0], shaped.y[1] - shaped.y[0]));
    expect(brainShaped({ x: Float32Array.of(1), y: Float32Array.of(2) }).x[0]).toBe(1);
  });
});

describe("weights and zones", () => {
  it("weighs a node by its lines and marks the most tied as big", () => {
    const degrees = degreesOf(5, [
      [0, 1, 0],
      [0, 2, 0],
      [0, 3, 0],
      [1, 9, 0],
    ]);
    expect(degrees).toEqual([3, 2, 1, 1, 0]);
    expect(bigOnes(degrees)).toEqual([true, false, false, false, false]);
    expect(bigOnes([1, 1, 1])).toEqual([false, false, false]);
    expect(weightOf(0)).toBe(1);
    expect(weightOf(1000)).toBe(3);
  });

  it("wraps points in their convex hull", () => {
    const hull = hullOf([
      [0, 0],
      [2, 0],
      [2, 2],
      [0, 2],
      [1, 1],
    ]);
    expect(hull).toHaveLength(4);
    expect(hull).not.toContainEqual([1, 1]);
    expect(hullOf([[0, 0]])).toEqual([[0, 0]]);
  });

  it("makes a zone of each folder or list with enough members, in its colour or a steady one", () => {
    const nodes = [
      ...Array.from({ length: 6 }, (_, n) => neuron(`a${n}`, "home")),
      ...Array.from({ length: 6 }, (_, n) => neuron(`b${n}`, "work")),
      neuron("c", "few"),
      neuron("#t", undefined, "tag"),
    ];
    const x = Float32Array.from(nodes.map((_, n) => n % 3));
    const y = Float32Array.from(nodes.map((_, n) => Math.floor(n / 3)));

    const zones = zonesOf(nodes, { x, y }, (group) => (group === "home" ? "teal" : undefined));

    expect(zones.map((one) => one.group)).toEqual(["home", "work"]);
    expect(zones[0].hue).toBe("teal");
    expect(zonesOf(nodes, { x, y }, () => undefined)[1].hue).toBe(zones[1].hue);
  });

  it("paints the zones only while they are wanted", () => {
    const calls: string[] = [];
    const brush = {
      clearRect: () => calls.push("clear"),
      beginPath: () => calls.push("path"),
      moveTo: () => {},
      lineTo: () => {},
      closePath: () => {},
      fill: () => calls.push("fill"),
      stroke: () => {},
    };
    const canvas = {
      width: 10,
      height: 10,
      clientWidth: 10,
      getContext: () => brush,
    } as unknown as HTMLCanvasElement;
    const zone = {
      group: "home",
      hue: "teal",
      hull: [
        [0, 0],
        [1, 0],
        [1, 1],
      ] as [number, number][],
    };

    paintZones(canvas, [zone], (point) => point, { teal: "#0f7a68" }, 0);
    paintZones(canvas, [zone, { ...zone, hull: [] }], (point) => point, { teal: "#0f7a68" }, 1);
    paintZones({ getContext: () => null } as unknown as HTMLCanvasElement, [zone], (p) => p, {}, 1);

    expect(calls).toEqual(["clear", "clear", "path", "fill"]);
  });
});

describe("thumbnails and overlays", () => {
  it("shows the pictures in view, nearest the middle first, and takes them away", async () => {
    const layer = document.createElement("div");
    Object.defineProperties(layer, {
      clientWidth: { value: 100 },
      clientHeight: { value: 100 },
    });
    const urlOf = vi.fn((id: string) =>
      id === "bad" ? Promise.reject(new Error("gone")) : Promise.resolve(`asset://${id}`),
    );
    const thumbs = thumbsIn(
      layer,
      [
        { node: 0, id: "a.png" },
        { node: 1, id: "b.png" },
        { node: 2, id: "bad" },
      ],
      urlOf,
    );
    const places: Record<number, [number, number] | undefined> = {
      0: [50, 50],
      1: [500, 50],
      2: [40, 40],
    };

    thumbs.place((node) => places[node], true);
    await flush();

    const shown = [...layer.querySelectorAll("img")].filter((img) => !img.hidden);
    expect(shown.map((img) => img.getAttribute("src"))).toEqual(["asset://a.png"]);
    thumbs.place((node) => places[node], false);
    expect([...layer.querySelectorAll("img")].every((img) => img.hidden)).toBe(true);
    thumbs.clear();
    expect(layer.querySelectorAll("img")).toHaveLength(0);
  });

  it("lays its canvas under the drawing, repaints on each frame and leaves nothing behind", () => {
    const container = document.createElement("figure");
    container.append(document.createElement("canvas"));
    let frame: () => void = () => {};
    const unhook = vi.fn();
    const drawing = {
      zoom: () => 0.4,
      toScreen: (point: [number, number]) => point,
      nodeOnScreen: () => undefined,
      turned: () => false,
      onFrame: (listener: () => void) => {
        frame = listener;
        return unhook;
      },
    } as unknown as Drawing;
    const network: Network = {
      nodes: [neuron("pics/a.png", undefined, "file"), neuron("notes.md", "home", "doc")],
      edges: [],
    };
    const pictures = {
      shows: (id: string) => id.endsWith(".png"),
      urlOf: vi.fn(() => Promise.resolve("")),
    };
    const palette = paletteFrom(() => "#102030");

    const overlays = overlaysOn(container, drawing, network, () => undefined, palette, pictures);
    overlays.reshape({ x: Float32Array.of(0, 1), y: Float32Array.of(0, 1) });
    overlays.reshape({ x: new Float32Array(), y: new Float32Array() });
    overlays.retheme(paletteFrom(() => "#405060"));
    frame();

    expect(container.firstElementChild?.tagName).toBe("CANVAS");
    expect(container.querySelectorAll("canvas")).toHaveLength(2);
    overlays.stop();
    expect(unhook).toHaveBeenCalled();
    expect(container.querySelectorAll("canvas")).toHaveLength(1);
  });
});
