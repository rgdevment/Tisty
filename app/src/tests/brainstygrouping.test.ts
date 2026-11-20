import { describe, expect, it } from "vitest";
import type { Group, Network } from "../brainsty";
import { amidMembers, laidEdges, withGroupsAfter } from "../ui/brainsty/grouping";
import { EVERYTHING, hiddenBy } from "../ui/brainsty/sifting";

const network: Network = {
  nodes: [
    { id: "a.md", kind: "doc", title: "A", group: "f1" },
    { id: "b.md", kind: "doc", title: "B", group: "f1" },
    { id: "01A", kind: "task", title: "bread", group: "l1" },
    { id: "#home", kind: "tag", title: "home" },
  ],
  edges: [[0, 3, 3]],
  near: 2,
};

const groups: Group[] = [
  { id: "f1", name: "Proyectos", color: "teal", kind: "folder" },
  { id: "l1", name: "Casa", kind: "list" },
  { id: "l2", name: "Vacía", kind: "list" },
  { id: "x", name: "Sin clase" },
];

describe("folders and lists as nodes of the view", () => {
  const grown = withGroupsAfter(network, groups);

  it("adds one node for each folder or list that holds something, after the ones Tisty sent", () => {
    expect(grown.nodes.slice(0, 4)).toEqual(network.nodes);
    expect(grown.nodes.slice(4)).toEqual([
      { id: "folder:f1", kind: "folder", title: "Proyectos", group: "f1" },
      { id: "list:l1", kind: "list", title: "Casa", group: "l1" },
    ]);
    expect(grown.near).toBe(2);
  });

  it("ties each member to its folder or list by a line of belonging, the view's own", () => {
    expect(grown.edges).toEqual([
      [0, 3, 3],
      [0, 4, 5],
      [1, 4, 5],
      [2, 5, 5],
    ]);
  });

  it("leaves a network with no folder or list as it was", () => {
    expect(withGroupsAfter(network, [])).toBe(network);
  });

  it("keeps folders and lists out of the layout and puts each amid its members", () => {
    expect(laidEdges(grown)).toEqual([[0, 3, 3]]);
    const spots = {
      x: Float32Array.from([0, 4, 10, 7, 99, 99]),
      y: Float32Array.from([2, 6, 1, 7, 99, 99]),
    };

    const placed = amidMembers(spots, grown);

    expect([...placed.x]).toEqual([0, 4, 10, 7, 2, 10]);
    expect([...placed.y]).toEqual([2, 6, 1, 7, 4, 1]);
    expect(spots.x[4]).toBe(99);
  });

  it("hides a folder or list once the filters hide every one of its members", () => {
    const today = new Date("2026-11-19");
    const tasks = { ...EVERYTHING, kinds: new Set(["task", "list", "folder"] as const) };

    expect(hiddenBy(grown, tasks, today)).toEqual([true, true, false, true, true, false]);
    expect(hiddenBy(grown, EVERYTHING, today)).toEqual([false, false, false, false, false, false]);
  });
});
