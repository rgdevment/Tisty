import { describe, expect, it } from "vitest";
import type { Group, Network } from "../brainsty";
import { groupsKey } from "../ui/brainsty/grouping";
import { EVERYTHING, hiddenBy } from "../ui/brainsty/sifting";

const network: Network = {
  nodes: [
    { id: "a.md", kind: "doc", title: "A", group: "f1" },
    { id: "CLAUDE.md", kind: "doc", title: "CLAUDE", group: "a1" },
    { id: "01A", kind: "task", title: "bread", group: "l1" },
    { id: "#home", kind: "tag", title: "home" },
  ],
  edges: [[0, 3, 3]],
};

describe("the folders and lists the network is painted in", () => {
  it("changes its key only with what the network draws of them", () => {
    const folder: Group = { id: "f1", name: "Proyectos", color: "teal" };
    const key = groupsKey([folder]);

    expect(groupsKey([{ ...folder, order: 3 } as Group])).toBe(key);
    expect(groupsKey([{ ...folder, color: "amber" }])).not.toBe(key);
    expect(groupsKey([{ ...folder, agents: true }])).not.toBe(key);
  });

  it("leaves out what sits in the folders asked, and nothing else", () => {
    const today = new Date("2026-11-19");

    expect(hiddenBy(network, EVERYTHING, today, new Set(["a1"]))).toEqual([
      false,
      true,
      false,
      false,
    ]);
    expect(hiddenBy(network, EVERYTHING, today)).toEqual([false, false, false, false]);
  });
});
