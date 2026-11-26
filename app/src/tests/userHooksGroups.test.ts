import { describe, expect, it } from "vitest";
import { agentGroups } from "../agentGroups";
import type { AgentRow } from "../agentsSync";
import type { Filed, Folded } from "../core";

const folder = (
  id: string,
  name: string,
  parent: string | null,
  agents?: Folded["agents"],
): Folded => ({ id, name, parent, icon: null, holds: 0, archived: false, away: false, agents });

const doc = (file: string, folderId: string | null, more: Partial<Filed> = {}): Filed => ({
  id: `id-${file}`,
  file,
  title: file,
  folder: folderId,
  archived: false,
  away: false,
  wrote: "2026-11-19T10:00:00Z",
  ...more,
});

const hook = (file: string, more: Partial<AgentRow> = {}): [string, AgentRow] => [
  file,
  {
    file,
    everyone: true,
    kind: "hook",
    level: "global",
    name: file,
    label: file,
    skill: null,
    ...more,
  },
];

const rule = (file: string): [string, AgentRow] => [
  file,
  {
    file,
    everyone: true,
    kind: "rule",
    level: "claude",
    name: "CLAUDE.md",
    label: file,
    skill: null,
  },
];

const FOLDERS = [
  folder("agents", "AGENTS", null),
  folder("global", "Global", "agents"),
  folder("claude", "Claude", "agents"),
  folder("laptop", "Laptop", "agents", { computer: "Laptop", mine: true, room: false }),
  folder("laptop-claude", "Claude", "laptop", { computer: "Laptop", mine: true, room: false }),
  folder("desk", "Desk", "agents", { computer: "Desk", mine: false, room: false }),
  folder("notes", "Notes", null),
];

const groups = (docs: Filed[], rows: [string, AgentRow][]) =>
  agentGroups(docs, FOLDERS, new Map(rows), "agents");

const files = (list: readonly Filed[]) => list.map((one) => one.file);

describe("the hooks Global draws from the person's own hook documents", () => {
  it("puts a hook for all computers and this computer's own in the Hooks group, never in left", () => {
    const made = groups(
      [doc("all", "global"), doc("mine", "laptop"), doc("deeper", "laptop-claude")],
      [hook("all"), hook("mine"), hook("deeper")],
    );
    expect(files(made.hooks)).toEqual(["all", "mine", "deeper"]);
    expect(made.left).toEqual([]);
    expect(made.rules.every((one) => one.docs.length === 0)).toBe(true);
  });

  it("leaves another computer's hooks out of the group and out of left", () => {
    const made = groups([doc("theirs", "desk")], [hook("theirs")]);
    expect(made.hooks).toEqual([]);
    expect(made.left).toEqual([]);
  });

  it("skips hooks that are pages, away, outside AGENTS or in no folder", () => {
    const made = groups(
      [
        doc("page", "global", { pageOf: "id-all" }),
        doc("away", "global", { away: true }),
        doc("outside", "notes"),
        doc("loose", null),
      ],
      [hook("page"), hook("away"), hook("outside"), hook("loose")],
    );
    expect(made.hooks).toEqual([]);
  });

  it("keeps rules and plain documents where they were beside the hooks", () => {
    const made = groups(
      [doc("r", "claude"), doc("h", "global"), doc("plain", "global")],
      [rule("r"), hook("h")],
    );
    expect(files(made.hooks)).toEqual(["h"]);
    expect(files(made.rules.find((one) => one.level === "claude")?.docs ?? [])).toEqual(["r"]);
    expect(files(made.left)).toEqual(["plain"]);
  });

  it("does not take a document without a row for a hook", () => {
    const made = groups([doc("unknown", "global")], []);
    expect(made.hooks).toEqual([]);
    expect(files(made.left)).toEqual(["unknown"]);
  });
});
