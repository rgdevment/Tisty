import { describe, expect, it } from "vitest";
import { layered } from "../src/graphlayout";

const lines = (table: string) => table.split("\n").filter((line) => line.trim());

// One per line: the arrows of a chart → the layer of each node, in the order they were written.
const LAYERS = `
A>B B>C → A0 B1 C2
A>B A>C B>C → A0 B1 C2
A>B B>C C>A → A0 B1 C2
A>A A>B → A0 B1
A>C B>C C>D → A0 B0 C1 D2
`;

const ranksOf = (case_: string) => {
  const [arrows, expected] = case_.split(" → ");
  const edges = arrows.split(" ").map((one) => one.split(">") as [string, string]);
  const ids = expected.split(" ").map((one) => one[0]);
  const laid = layered(ids, edges, new Map(), "TB");
  const rows = [...new Set(ids.map((id) => laid.at.get(id)?.y))].sort(
    (a, b) => (a ?? 0) - (b ?? 0),
  );
  return {
    ids,
    expected,
    got: ids.map((id) => `${id}${rows.indexOf(laid.at.get(id)?.y)}`).join(" "),
  };
};

describe("laying out a flowchart in layers", () => {
  it("puts each node one layer past the longest way to it, cutting loops where they close", () => {
    for (const case_ of lines(LAYERS)) {
      const { expected, got } = ranksOf(case_);
      expect(got, case_).toBe(expected);
    }
  });

  it("lays out a large, densely joined chart at once", () => {
    const ids = Array.from({ length: 400 }, (_, at) => `n${at}`);
    const edges = ids.flatMap((id, at) =>
      [1, 2, 3].flatMap((step) => (ids[at + step] ? [[id, ids[at + step]] as const] : [])),
    );
    const started = performance.now();
    const laid = layered(ids, edges, new Map(), "LR");

    expect(laid.at.size).toBe(400);
    expect((laid.at.get("n399")?.x ?? 0) > (laid.at.get("n398")?.x ?? 0)).toBe(true);
    expect(performance.now() - started).toBeLessThan(1000);
  });
});
