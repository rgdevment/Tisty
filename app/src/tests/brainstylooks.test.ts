import { describe, expect, it } from "vitest";
import type { Neuron } from "../brainsty";
import {
  HUE_KEYS,
  lineLook,
  nodeLook,
  type Palette,
  paletteFrom,
  rgba,
  shapeOf,
  standingOf,
} from "../ui/brainsty/looks";

const palette: Palette = paletteFrom((name) =>
  name === "--tisty-ink"
    ? "#1d1d1f"
    : name === "--tisty-soft"
      ? "#57575c"
      : name === "--tisty-accent"
        ? "#0060e3"
        : name === "--tisty-hue-purple"
          ? "rgb(122 68 184)"
          : name === "--tisty-hue-gray"
            ? "#6a6a70"
            : "#1f6fb2",
);

const one = (kind: Neuron["kind"], state?: string, group?: string): Neuron => ({
  id: "x",
  kind,
  title: "x",
  state,
  group,
});

describe("how Brainsty draws each node and line", () => {
  it("reads every hue of the palette from Tisty's colours", () => {
    expect(Object.keys(palette.hues)).toEqual([...HUE_KEYS, "coral"]);
    expect(palette.tag).toBe("rgb(122 68 184)");
    expect(palette.file).toBe("#6a6a70");
  });

  it("turns hex and both rgb spellings into rgba with the alpha asked", () => {
    expect(rgba("#0060e3", 0.5)).toBe("rgba(0, 96, 227, 0.5)");
    expect(rgba(" rgb(0 0 0 / 0.16) ", 0.5)).toBe("rgba(0, 0, 0, 0.08)");
    expect(rgba("rgba(10, 20, 30, 50%)", 1)).toBe("rgba(10, 20, 30, 0.5)");
    expect(rgba("rgb(10, 20, 30)", 0.25)).toBe("rgba(10, 20, 30, 0.25)");
    expect(rgba("teal", 0.5)).toBe("teal");
  });

  it("draws documents and pages as squares and the rest as circles", () => {
    expect(shapeOf("doc")).toBe("square");
    expect(shapeOf("page")).toBe("square");
    expect(shapeOf("task")).toBe("circle");
    expect(shapeOf("tag")).toBe("circle");
    expect(shapeOf("file")).toBe("circle");
  });

  it("lights what is under way, dims what is done and fades what is put away", () => {
    expect(standingOf(one("task", "open"))).toBe("live");
    expect(standingOf(one("task", "said_done"))).toBe("live");
    expect(standingOf(one("tag"))).toBe("live");
    expect(standingOf(one("task", "done"))).toBe("done");
    expect(standingOf(one("task", "dropped"))).toBe("archived");
    expect(standingOf(one("doc", "archived"))).toBe("archived");

    const live = nodeLook(one("task", "open"), undefined, palette);
    const done = nodeLook(one("task", "done"), undefined, palette);
    const away = nodeLook(one("task", "dropped"), undefined, palette);
    expect(live.color).toBe("rgba(0, 96, 227, 1)");
    expect(done.color).toBe("rgba(0, 96, 227, 0.45)");
    expect(away.size).toBeLessThan(done.size);
    expect(done.size).toBeLessThan(live.size);
  });

  it("paints a node in its folder or list colour, or in its kind's own", () => {
    expect(nodeLook(one("doc", "open", "f1"), "blue", palette).color).toBe("rgba(31, 111, 178, 1)");
    expect(nodeLook(one("doc", "open"), undefined, palette).color).toBe("rgba(87, 87, 92, 1)");
    expect(nodeLook(one("tag"), undefined, palette).color).toBe("rgba(122, 68, 184, 1)");
    expect(nodeLook(one("file"), undefined, palette).color).toBe("rgba(106, 106, 112, 1)");
    expect(nodeLook(one("page", "open"), undefined, palette).size).toBeLessThan(
      nodeLook(one("doc", "open"), undefined, palette).size,
    );
  });

  it("gives each kind of line its own look, and only a mention a gap", () => {
    const cite = lineLook("cite", palette);
    const mention = lineLook("mention", palette);
    const structure = lineLook("structure", palette);
    const tag = lineLook("tag", palette);
    const uses = lineLook("uses", palette);
    expect(mention.gap).toBeGreaterThan(0);
    expect([cite, structure, tag, uses].every((one) => one.gap === 0)).toBe(true);
    expect(cite.size).toBeGreaterThan(structure.size);
    expect(structure.size).toBeGreaterThan(tag.size);
    expect(tag.color).toBe("rgba(122, 68, 184, 0.22)");
    expect(uses.color).toBe("rgba(106, 106, 112, 0.35)");
    expect(
      new Set([cite, mention, structure, tag, uses].map((one) => JSON.stringify(one))).size,
    ).toBe(5);
  });
});
