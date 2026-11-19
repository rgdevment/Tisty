import { describe, expect, it } from "vitest";
import type { Network } from "../brainsty";
import {
  alikeOf,
  alphaOf,
  type Curve,
  curvesOf,
  curvesShown,
  wordsSaid,
} from "../ui/brainsty/guessing";

const network: Network = {
  nodes: [
    { id: "a", kind: "task", title: "a" },
    { id: "b", kind: "task", title: "b" },
    { id: "c", kind: "doc", title: "c" },
    { id: "d", kind: "doc", title: "d" },
  ],
  edges: [],
  alike: [
    [0, 1, 200, 0, "budget family"],
    [2, 0, 900, 1, ""],
    [3, 0, 600, 0, "budget"],
    [2, 3, 800, 1, ""],
  ],
};

const showing = { on: true, picked: null, zoom: 1, hidden: [] };

describe("Tisty's guesses in the network", () => {
  it("weighs each guess within its own method, so neither method always looks faint", () => {
    const curves = curvesOf(network.alike ?? []);

    expect(curves.map((one) => one.strength)).toEqual([0, 1, 1, 0]);
    expect(curvesOf([[0, 1, 500, 0, "one"]])[0].strength).toBe(1);
    expect(curvesOf([])).toEqual([]);
  });

  it("grows more opaque the more alike the two are, never vanishing nor solid", () => {
    expect(alphaOf(0)).toBeCloseTo(0.3);
    expect(alphaOf(1)).toBeCloseTo(0.95);
    expect(alphaOf(0.5)).toBeGreaterThan(alphaOf(0.2));
    expect(alphaOf(3)).toBeCloseTo(0.95);
    expect(alphaOf(-1)).toBeCloseTo(0.3);
  });

  it("shows near up, fading in, and none from afar", () => {
    const curves = curvesOf(network.alike ?? []);

    expect(curvesShown(curves, { ...showing, zoom: 0.2 })).toEqual([]);
    expect(curvesShown(curves, { ...showing, zoom: 1 }).map((one) => one.fade)).toEqual([
      1, 1, 1, 1,
    ]);
    const fading = curvesShown(curves, { ...showing, zoom: 0.65 });
    expect(fading).toHaveLength(4);
    expect(fading[0].fade).toBeGreaterThan(0);
    expect(fading[0].fade).toBeLessThan(1);
  });

  it("shows only the picked node's guesses, from any distance", () => {
    const curves = curvesOf(network.alike ?? []);

    const shown = curvesShown(curves, { ...showing, picked: 3, zoom: 0 });

    expect(shown.map(({ curve }) => [curve.from, curve.to])).toEqual([
      [3, 0],
      [2, 3],
    ]);
    expect(shown.every((one) => one.fade === 1)).toBe(true);
  });

  it("shows none with the filter off, nor one with an end out of sight", () => {
    const curves: Curve[] = curvesOf(network.alike ?? []);

    expect(curvesShown(curves, { ...showing, on: false })).toEqual([]);
    const shown = curvesShown(curves, { ...showing, hidden: [false, false, true, false] });
    expect(shown.map(({ curve }) => [curve.from, curve.to])).toEqual([
      [0, 1],
      [3, 0],
    ]);
  });

  it("lists a node's guesses from the most alike, with the words they share", () => {
    expect(alikeOf(network, 0, [])).toEqual([
      { node: 2, words: [] },
      { node: 3, words: ["budget"] },
      { node: 1, words: ["budget", "family"] },
    ]);
    expect(alikeOf(network, 0, [false, false, true])).toEqual([
      { node: 3, words: ["budget"] },
      { node: 1, words: ["budget", "family"] },
    ]);
    expect(alikeOf({ ...network, alike: undefined }, 0, [])).toEqual([]);
  });

  it("says the shared words as one phrase, each word marked", () => {
    expect(wordsSaid(["presupuesto"], "y")).toEqual([{ text: "presupuesto", word: true }]);
    expect(wordsSaid(["presupuesto", "familiar"], "y")).toEqual([
      { text: "presupuesto", word: true },
      { text: " y ", word: false },
      { text: "familiar", word: true },
    ]);
    expect(
      wordsSaid(["a", "b", "c"], "and")
        .map((one) => one.text)
        .join(""),
    ).toBe("a, b and c");
    expect(wordsSaid([], "y")).toEqual([]);
  });
});
