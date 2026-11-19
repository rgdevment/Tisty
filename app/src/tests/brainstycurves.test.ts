import { afterEach, describe, expect, it, vi } from "vitest";
import type { Network } from "../brainsty";
import { bendOf, curvesOn, paintCurves } from "../ui/brainsty/curves";
import type { Drawing } from "../ui/brainsty/drawing";
import { paletteFrom } from "../ui/brainsty/looks";

interface Stroke {
  colour: string;
  path: number[][];
}

const brushed = () => {
  const strokes: Stroke[] = [];
  let path: number[][] = [];
  let cleared = 0;
  const brush = {
    lineWidth: 0,
    lineCap: "",
    strokeStyle: "",
    clearRect: () => {
      cleared++;
      strokes.length = 0;
    },
    beginPath: () => {
      path = [];
    },
    moveTo: (x: number, y: number) => path.push([x, y]),
    quadraticCurveTo: (cx: number, cy: number, x: number, y: number) => path.push([cx, cy, x, y]),
    stroke: () => strokes.push({ colour: brush.strokeStyle, path }),
  };
  const canvas = document.createElement("canvas");
  canvas.width = 200;
  canvas.height = 100;
  vi.spyOn(canvas, "getContext").mockReturnValue(brush as unknown as CanvasRenderingContext2D);
  return { canvas, brush, strokes, cleared: () => cleared };
};

const palette = paletteFrom((name) => (name === "--tisty-hue-amber" ? "#e0a030" : "#888888"));

const network: Network = {
  nodes: [
    { id: "a", kind: "task", title: "a" },
    { id: "b", kind: "task", title: "b" },
    { id: "c", kind: "doc", title: "c" },
  ],
  edges: [[0, 2, 0]],
  alike: [
    [0, 1, 300, "budget"],
    [1, 2, 900, "family"],
  ],
};

const drawingOf = (zoom: number) => {
  const frames: (() => void)[] = [];
  const drawing = {
    zoom: () => zoom,
    toGraph: (point: [number, number]) => point,
    nodeOnScreen: (node: number) =>
      [
        [10, 10],
        [50, 10],
        [90, 40],
      ][node] as [number, number],
    onFrame: (listener: () => void) => {
      frames.push(listener);
      return () => frames.splice(frames.indexOf(listener), 1);
    },
  } as unknown as Drawing;
  return { drawing, frames };
};

afterEach(() => {
  vi.restoreAllMocks();
  document.body.innerHTML = "";
});

describe("the amber curves", () => {
  it("bend to one side of the straight line between the two ends", () => {
    expect(bendOf([0, 0], [10, 0])).toEqual([5, 2]);
    expect(bendOf([0, 0], [0, 10])).toEqual([-2, 5]);
  });

  it("paint each guess as a curve, scaled to the canvas, skipping what is off screen", () => {
    const { canvas, strokes, brush } = brushed();
    const ends: Record<number, [number, number]> = { 0: [10, 10], 1: [50, 10], 2: [500, 500] };

    paintCurves(
      canvas,
      [
        { curve: { from: 0, to: 1, strength: 1 }, fade: 1 },
        { curve: { from: 1, to: 2, strength: 0 }, fade: 1 },
        { curve: { from: 2, to: 2, strength: 0 }, fade: 1 },
        { curve: { from: 0, to: 9, strength: 0 }, fade: 1 },
      ],
      (node) => ends[node],
      "#e0a030",
      2,
    );

    expect(brush.lineWidth).toBe(3);
    expect(strokes).toHaveLength(2);
    expect(strokes[0].path).toEqual([
      [20, 20],
      [60, 36, 100, 20],
    ]);
    expect(strokes[0].colour).toBe("rgba(224, 160, 48, 0.95)");
    expect(strokes[1].colour).toBe("rgba(224, 160, 48, 0.3)");
  });

  it("draw nothing without a canvas to paint on", () => {
    const canvas = document.createElement("canvas");
    vi.spyOn(canvas, "getContext").mockReturnValue(null);

    expect(() =>
      paintCurves(
        canvas,
        [{ curve: { from: 0, to: 1, strength: 1 }, fade: 1 }],
        () => [0, 0],
        "#e0a030",
        1,
      ),
    ).not.toThrow();
  });

  it("lie beneath the network, follow each frame and answer the filter, a pick and the sift", () => {
    const container = document.createElement("div");
    Object.defineProperty(container, "clientWidth", { value: 200 });
    Object.defineProperty(container, "clientHeight", { value: 100 });
    const sigmaCanvas = document.createElement("canvas");
    container.append(sigmaCanvas);
    document.body.append(container);
    const { canvas, strokes } = brushed();
    vi.spyOn(document, "createElement").mockReturnValueOnce(canvas);
    const { drawing, frames } = drawingOf(1);

    const curves = curvesOn(container, drawing, network, palette);

    expect(container.firstElementChild).toBe(canvas);
    expect(canvas.style.pointerEvents).toBe("none");
    expect(frames).toHaveLength(1);
    frames[0]();
    expect(strokes).toHaveLength(0);
    curves.reshape({ x: Float32Array.of(10, 50), y: Float32Array.of(10, 10) });
    frames[0]();
    expect(strokes).toHaveLength(0);
    curves.reshape({ x: Float32Array.of(10, 50, 900), y: Float32Array.of(10, 10, 40) });
    frames[0]();
    expect(strokes).toHaveLength(1);
    curves.reshape({ x: Float32Array.of(10, 50, 90), y: Float32Array.of(10, 10, 40) });
    frames[0]();
    expect(strokes).toHaveLength(2);
    curves.pick(0);
    expect(strokes).toHaveLength(1);
    curves.pick(null);
    curves.sift([false, false, true]);
    expect(strokes).toHaveLength(1);
    curves.show(false);
    expect(strokes).toHaveLength(0);
    curves.show(true);
    curves.retheme(paletteFrom(() => "#102030"));
    expect(strokes[0].colour).toContain("16, 32, 48");

    curves.stop();
    expect(frames).toHaveLength(0);
    expect(container.contains(canvas)).toBe(false);
  });

  it("cost nothing on a network with no guess", () => {
    const container = document.createElement("div");
    const { canvas, cleared } = brushed();
    vi.spyOn(document, "createElement").mockReturnValueOnce(canvas);
    const { drawing, frames } = drawingOf(1);

    const curves = curvesOn(container, drawing, { ...network, alike: undefined }, palette);
    curves.pick(0);
    curves.show(false);

    expect(frames).toHaveLength(0);
    expect(cleared()).toBe(0);
    curves.stop();
    expect(container.childElementCount).toBe(0);
  });
});
