import { afterEach, describe, expect, it, vi } from "vitest";
import type { Network } from "../brainsty";
import type { Drawing } from "../ui/brainsty/drawing";
import { glyphOf, iconSource, iconsOn } from "../ui/brainsty/icons";

const network: Network = {
  nodes: [
    { id: "a.md", kind: "doc", title: "A" },
    { id: "01A", kind: "task", title: "bread", state: "open" },
    { id: "01B", kind: "task", title: "milk", state: "done" },
    { id: "rina.jpg", kind: "file", title: "rina" },
    { id: "list:l1", kind: "list", title: "Casa", group: "l1" },
  ],
  edges: [],
};

describe("the icon of each node up close", () => {
  it("says what the node is, with a done task ticked", () => {
    expect(network.nodes.map(glyphOf)).toEqual(["page", "checkbox", "check-mark", "clip", "list"]);
  });

  it("is drawn from Tisty's own glyphs in the colour asked", () => {
    const source = decodeURIComponent(iconSource("folder", "#ffffff") ?? "");
    expect(source).toContain('stroke="#ffffff"');
    expect(source).toContain("<path");
    expect(iconSource("no-such-glyph", "#ffffff")).toBeNull();
  });
});

const tint = { of: (_: number): string => "#2266ff" };

describe("the icon layer", () => {
  const painted: { image: string; x: number; y: number; alpha: number }[] = [];
  const brush = {
    globalAlpha: 1,
    clearRect: vi.fn(),
    drawImage(image: HTMLImageElement, x: number, y: number) {
      painted.push({ image: image.src, x, y, alpha: brush.globalAlpha });
    },
  };

  afterEach(() => {
    tint.of = () => "#2266ff";
    vi.restoreAllMocks();
    painted.length = 0;
  });

  const drawn = (zoom: number) => {
    const frames: (() => void)[] = [];
    const drawing = {
      zoom: () => zoom,
      nodeOnScreen: (node: number) => [10 + node * 20, 30] as [number, number],
      onFrame: (listener: () => void) => {
        frames.push(listener);
        return () => frames.splice(frames.indexOf(listener), 1);
      },
    } as unknown as Drawing;
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(
      brush as unknown as CanvasRenderingContext2D,
    );
    vi.spyOn(HTMLImageElement.prototype, "complete", "get").mockReturnValue(true);
    const box = document.createElement("div");
    Object.defineProperty(box, "clientWidth", { value: 400 });
    Object.defineProperty(box, "clientHeight", { value: 300 });
    const icons = iconsOn(
      box,
      drawing,
      network,
      (node) => tint.of(node),
      (id) => id === "rina.jpg",
    );
    return {
      box,
      icons,
      frame: () => {
        for (const one of frames) one();
      },
    };
  };

  it("paints nothing from afar", () => {
    const { frame } = drawn(0.3);
    frame();
    expect(painted).toEqual([]);
  });

  it("paints every shown node up close, dims what is not lit, skips the hidden and the pictured", () => {
    const { icons, box } = drawn(1);
    icons.sift([false, true, false, false, false]);
    painted.length = 0;
    icons.light(new Set([0]));

    expect(painted.map((one) => decodeURIComponent(one.image).includes("<svg"))).toEqual([
      true,
      true,
      true,
    ]);
    expect(painted.map((one) => one.alpha)).toEqual([1, 0.15, 0.15]);
    expect(box.querySelector("canvas")).not.toBeNull();
    icons.stop();
    expect(box.querySelector("canvas")).toBeNull();
  });

  it("draws each node as its icon in the node's own colour, and again in a new theme's", () => {
    tint.of = (node) => (node === 0 ? "#11aa99" : "#2266ff");
    const { icons } = drawn(1);
    icons.light(null);
    const colours = painted.map(
      (one) => /stroke="([^"]+)"/.exec(decodeURIComponent(one.image))?.[1],
    );
    expect(colours[0]).toBe("#11aa99");
    expect(colours[1]).toBe("#2266ff");

    tint.of = () => "#101010";
    painted.length = 0;
    icons.retheme();
    expect(painted.every((one) => decodeURIComponent(one.image).includes("#101010"))).toBe(true);
  });
});
