import { PALETTES, Refused, sayIn, scaleFor, svgFor } from "tisty-diagrams/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { plainly } from "../refusal";
import { copiedImage, exportChoices, exported, fileName } from "../ui/drawing/exporting";

const kept = vi.hoisted(() => ({
  path: null as string | null,
  asked: [] as { defaultPath?: string }[],
  files: [] as { at: string; bytes: number[] }[],
  images: [] as unknown[],
  drawn: null as string | null,
  broken: false,
  sources: [] as string[],
  canvases: [] as [number, number, string | undefined][],
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: (asked: { defaultPath?: string }) => {
    kept.asked.push(asked);
    return Promise.resolve(kept.path);
  },
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeImage: (image: unknown) => {
    kept.images.push(image);
    return Promise.resolve();
  },
}));

vi.mock("../core", () => ({
  keepFile: (at: string, bytes: number[]) => {
    kept.files.push({ at, bytes });
    return Promise.resolve();
  },
}));

vi.mock("tisty-reader/page", async (real) => ({
  ...(await real<typeof import("tisty-reader/page")>()),
  drawing: () => Promise.resolve(kept.drawn),
}));

// The page cannot load pictures or paint in a test; these stand in for the browser's own.
class Picture {
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  set src(said: string) {
    kept.sources.push(said);
    queueMicrotask(() => (kept.broken ? this.onerror?.() : this.onload?.()));
  }
}

vi.stubGlobal("Image", Picture);
HTMLCanvasElement.prototype.getContext = (() => ({
  scale: () => {},
  drawImage: () => {},
})) as unknown as HTMLCanvasElement["getContext"];
HTMLCanvasElement.prototype.toBlob = function (done: BlobCallback, type?: string) {
  kept.canvases.push([this.width, this.height, type]);
  done({ arrayBuffer: () => Promise.resolve(new Uint8Array([1, 2, 3]).buffer) } as Blob);
};

const FLOW = "flowchart TD\n  A[Inicio] --> B[Fin]\n";
const draw = () => Promise.resolve(kept.drawn);
const svgIn = (source: string) => decodeURIComponent(source.replace(/^data:[^,]*,/, ""));

beforeEach(() => {
  kept.path = "C:/salida/pagos";
  kept.asked = [];
  kept.files = [];
  kept.images = [];
  kept.drawn = null;
  kept.broken = false;
  kept.sources = [];
  kept.canvases = [];
});

// One per line: the document's title → which diagram it is, counting from 0 → the name offered.
const NAMED = `
Pagos | 0 | Pagos
Pagos | 2 | Pagos-3
Plan: Q1/Q2 <borrador>? | 0 | Plan Q1Q2 borrador
  |  0 | ${t("untitledDoc")}
`;

describe("the name a diagram is saved under", () => {
  it("is the document's title, without what a disk refuses, numbered from the second diagram", () => {
    for (const case_ of NAMED.split("\n").filter((line) => line.includes("|"))) {
      const [title, nth, name] = case_.split("|").map((one) => one.trim());
      expect(fileName(title, Number(nth)), case_).toBe(name);
    }
  });
});

describe("saving a diagram as a file", () => {
  it("writes Tisty's own drawing as SVG, light and without a ground", async () => {
    expect(await exported(FLOW, "svg", "Pagos", "Inter")).toBe(true);

    const written = new TextDecoder().decode(new Uint8Array(kept.files[0].bytes));
    expect(kept.asked[0].defaultPath).toBe("Pagos.svg");
    expect(kept.files[0].at).toBe("C:/salida/pagos");
    expect(written).toMatch(/^<svg xmlns="http:\/\/www.w3.org\/2000\/svg"/);
    expect(written).toContain("Inicio");
    expect(written).toContain(PALETTES.light.accent);
    expect(written).not.toContain('width="100%"');
  });

  it("paints a PNG or a JPG at twice the size, on the ground of the theme chosen", async () => {
    const drawn = await svgFor(FLOW, true, true, "Inter", draw);

    await exported(FLOW, "pngDark", "Pagos", "Inter");
    await exported(FLOW, "jpgLight", "Pagos", "Inter");

    const twice = [Math.ceil(drawn.size.wide * 2), Math.ceil(drawn.size.tall * 2)];
    expect(kept.canvases).toEqual([
      [...twice, "image/png"],
      [...twice, "image/jpeg"],
    ]);
    expect(kept.asked.map((one) => one.defaultPath)).toEqual(["Pagos.png", "Pagos.jpg"]);
    expect(kept.files.map((one) => one.bytes)).toEqual([
      [1, 2, 3],
      [1, 2, 3],
    ]);
    expect(svgIn(kept.sources[0])).toContain(`fill="${PALETTES.dark.sheet}"`);
    expect(svgIn(kept.sources[1])).toContain(`fill="${PALETTES.light.sheet}"`);
  });

  it("writes nothing when the person closes the dialog", async () => {
    kept.path = null;

    expect(await exported(FLOW, "pngClear", "Pagos", "Inter")).toBe(false);
    expect(kept.files).toEqual([]);
  });

  it("takes Mermaid's drawing for a kind the window leaves to it, at its own size", async () => {
    kept.drawn = '<svg width="100%" viewBox="0 0 200 100"><text>hola</text></svg>';

    const drawn = await svgFor("sequenceDiagram\n  A->>B: hola\n", false, false, "Inter", draw);

    expect(drawn.size).toEqual({ wide: 200, tall: 100 });
    expect(drawn.svg).toBe(
      `<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 100">${kept.drawn}</svg>`,
    );
  });

  it("says what failed rather than writing an empty file: Mermaid's drawing, or the painting", async () => {
    await expect(exported("sequenceDiagram\n  roto", "svg", "x", "Inter")).rejects.toMatchObject({
      code: "diagramUnreadable",
    });
    kept.broken = true;
    await expect(exported(FLOW, "pngLight", "x", "Inter")).rejects.toMatchObject({
      code: "diagramUnpainted",
    });
    expect(kept.files).toEqual([]);
  });
});

// One per line: a drawing's width and height → how many times its size it is painted at.
const SCALED = `
800 600 → 2
9000 300 → 1.82
3000 3000 → 1.33
`;

describe("painting a large drawing", () => {
  it("paints at twice its size unless the browser would refuse a canvas that big", () => {
    for (const case_ of SCALED.split("\n").filter((line) => line.trim())) {
      const [size, scale] = case_.split(" → ");
      const [wide, tall] = size.split(" ").map(Number);
      expect(scaleFor({ wide, tall }), case_).toBeCloseTo(Number(scale), 2);
    }
  });
});

describe("copying a diagram as an image", () => {
  it("puts a PNG on the clipboard, on the ground of the theme in use", async () => {
    await copiedImage(FLOW, true, "Inter");

    expect(kept.images).toEqual([new Uint8Array([1, 2, 3])]);
    expect(svgIn(kept.sources[0])).toContain(`fill="${PALETTES.dark.sheet}"`);
  });

  it("offers every file and then the copy, and says how each went", async () => {
    const said = vi.fn();
    const failed = vi.fn();
    const choices = exportChoices({
      source: FLOW,
      name: () => Promise.resolve("Pagos"),
      dark: false,
      said,
      failed,
    });

    expect(choices.map((one) => one.key)).toEqual([
      "pngClear",
      "pngLight",
      "pngDark",
      "jpgLight",
      "jpgDark",
      "svg",
      "copy",
    ]);
    choices.find((one) => one.key === "svg")?.onPick?.();
    choices.find((one) => one.key === "copy")?.onPick?.();
    await vi.waitFor(() => expect(said).toHaveBeenCalledTimes(2));
    expect(said.mock.calls).toEqual([[t("diagramExported")], [t("diagramCopiedImage")]]);

    kept.broken = true;
    choices.find((one) => one.key === "copy")?.onPick?.();
    await vi.waitFor(() => expect(failed).toHaveBeenCalled());
  });
});

describe("a failure the studio names", () => {
  it("is said in the studio's own words, in the person's language", () => {
    expect(plainly(new Refused("diagramUnpainted"))).toBe(sayIn("en")("diagramUnpainted"));
  });
});
