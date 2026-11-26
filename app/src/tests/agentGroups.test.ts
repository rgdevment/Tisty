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

const row = (file: string, more: Partial<AgentRow> = {}): [string, AgentRow] => [
  file,
  {
    file,
    everyone: true,
    kind: "rule",
    level: "claude",
    name: `rules/${file}.md`,
    label: file,
    skill: null,
    ...more,
  },
];

const FOLDERS = [
  folder("agents", "AGENTS", null),
  folder("global", "Global", "agents"),
  folder("claude", "Claude", "agents"),
  folder("codex", "Codex", "agents"),
  folder("codex-sub", "rules", "codex"),
  folder("laptop", "Laptop", "agents", { computer: "Laptop", mine: true, room: false }),
  folder("laptop-claude", "Claude", "laptop", { computer: "Laptop", mine: true, room: false }),
  folder("notes", "Notes", null),
];

const groups = (docs: Filed[], rows: [string, AgentRow][]) =>
  agentGroups(docs, FOLDERS, new Map(rows), "agents");

const files = (list: readonly Filed[]) => list.map((one) => one.file);

describe("the groups Global draws for what is for all computers", () => {
  it("groups rules by agent in the order Agents lists them", () => {
    const made = groups(
      [doc("g", "global"), doc("c", "global"), doc("x", "global"), doc("d", "global")],
      [
        row("g", { level: "gemini" }),
        row("c", { level: "claude" }),
        row("x", { level: "codex" }),
        row("d", { level: "dsh" }),
      ],
    );
    expect(made.rules.map((one) => one.level)).toEqual(["claude", "codex", "gemini", "dsh"]);
    expect(made.rules.map((one) => one.named)).toEqual([
      "Claude",
      "Codex",
      "Gemini",
      "DeepSeek Harness",
    ]);
    expect(files(made.rules[0].docs)).toEqual(["c"]);
  });

  it("puts a rule of an unknown agent after the known ones", () => {
    const made = groups(
      [doc("z", "global"), doc("c", "global")],
      [row("z", { level: "zed" }), row("c", { level: "cursor" })],
    );
    expect(made.rules.map((one) => one.level)).toEqual(["cursor", "zed"]);
    expect(made.rules[1].named).toBe("zed");
  });

  it("puts a rule of Claude in Rules whether it sits in Global or in the Claude folder", () => {
    const made = groups(
      [doc("a", "global"), doc("b", "claude"), doc("c", "agents")],
      [row("a"), row("b"), row("c")],
    );
    expect(made.rules).toHaveLength(1);
    expect(made.rules[0].level).toBe("claude");
    expect(files(made.rules[0].docs)).toEqual(["a", "b", "c"]);
  });

  it("leaves out a rule inside a computer's own folder", () => {
    const made = groups(
      [doc("mine", "laptop-claude"), doc("also", "laptop")],
      [row("mine", { everyone: false }), row("also", { everyone: false })],
    );
    expect(made.rules).toEqual([]);
    expect(made.left).toEqual([]);
  });

  it("leaves out a document inside a computer's folder even if its row says everyone", () => {
    const made = groups([doc("odd", "laptop-claude")], [row("odd")]);
    expect(made.rules).toEqual([]);
    expect(made.left).toEqual([]);
  });

  it("leaves out documents outside the AGENTS folder", () => {
    const made = groups([doc("x", "notes"), doc("y", null)], [row("x"), row("y")]);
    expect(made.rules).toEqual([]);
    expect(made.left).toEqual([]);
  });

  it("leaves out pages of a document", () => {
    const made = groups([doc("p", "global", { pageOf: "a" })], [row("p")]);
    expect(made.rules).toEqual([]);
    expect(made.left).toEqual([]);
  });

  it("draws a skill of several files as one skill and one of a single file as itself", () => {
    const made = groups(
      [doc("s1", "global"), doc("s2", "global"), doc("one", "global")],
      [
        row("s1", { kind: "skill", name: "pr/SKILL.md", label: "pr", skill: "pr" }),
        row("s2", {
          kind: "skill",
          name: "pr/references/body.md",
          label: "body",
          skill: "pr",
        }),
        row("one", { kind: "skill", name: "solo/SKILL.md", label: "solo", skill: "solo" }),
      ],
    );
    expect(made.rules).toEqual([]);
    expect(made.skills).toHaveLength(2);
    const [several, single] = made.skills;
    expect("skill" in several && files(several.skill.docs)).toEqual(["s1", "s2"]);
    expect("skill" in several && several.skill.label).toBe("pr");
    expect("doc" in single && single.doc.file).toBe("one");
  });

  it("leaves a memory, a global-level rule and a plain document where they are kept", () => {
    const made = groups(
      [doc("m", "global"), doc("g", "global"), doc("plain", "global"), doc("nocard", "claude")],
      [
        row("m", { kind: "memory", name: "MEMORY.md" }),
        row("g", { level: "global" }),
        row("plain", { everyone: false }),
      ],
    );
    expect(made.rules).toEqual([]);
    expect(made.skills).toEqual([]);
    expect(files(made.left)).toEqual(["m", "g", "plain", "nocard"]);
  });

  it("keeps a rule that is not for everyone out of the groups", () => {
    const made = groups([doc("a", "global")], [row("a", { everyone: false })]);
    expect(made.rules).toEqual([]);
    expect(files(made.left)).toEqual(["a"]);
  });
});

describe("the agent folders Global no longer draws", () => {
  it("empties a root Codex folder whose only document went to Rules", () => {
    const made = groups([doc("x", "codex")], [row("x", { level: "codex" })]);
    expect([...made.emptied]).toEqual(["codex"]);
  });

  it("empties it when the documents are in a folder below it", () => {
    const made = groups([doc("x", "codex-sub")], [row("x", { level: "codex" })]);
    expect([...made.emptied]).toEqual(["codex"]);
  });

  it("keeps a root Codex folder that still holds a document left behind", () => {
    const made = groups(
      [doc("x", "codex"), doc("m", "codex-sub")],
      [row("x", { level: "codex" }), row("m", { kind: "memory", name: "MEMORY.md" })],
    );
    expect(made.emptied.has("codex")).toBe(false);
  });

  it("never empties Global, a computer's folder or a folder that holds nothing", () => {
    const made = groups(
      [doc("a", "global"), doc("b", "laptop-claude")],
      [row("a"), row("b", { everyone: false })],
    );
    expect(made.emptied.size).toBe(0);
  });

  it("does not empty a Claude folder that holds nothing for all", () => {
    expect(groups([], []).emptied.size).toBe(0);
  });
});

describe("the empty rules entries for this computer's agents", () => {
  const room = (id: string, name: string, mine = true, parent = "laptop") =>
    folder(id, name, parent, { computer: "Laptop", mine, room: true });
  const withRooms = (rooms: Folded[], docs: Filed[] = [], rows: [string, AgentRow][] = []) =>
    agentGroups(docs, [...FOLDERS, ...rooms], new Map(rows), "agents");

  it("adds an empty entry for each agent this computer has a folder for", () => {
    const made = withRooms([room("l-codex", "Codex"), room("l-gemini", "Gemini")]);
    expect(made.rules.map((one) => [one.level, one.named, one.docs])).toEqual([
      ["codex", "Codex", []],
      ["gemini", "Gemini", []],
    ]);
  });

  it("leaves out Cursor, unknown names, another computer's rooms and folders with no room", () => {
    const made = withRooms([
      room("l-cursor", "Cursor"),
      room("l-zed", "Zed"),
      room("o-codex", "Codex", false, "other"),
      folder("l-bare", "Gemini", "laptop", { computer: "Laptop", mine: true, room: false }),
    ]);
    expect(made.rules).toEqual([]);
  });

  it("adds none when rules for that agent already exist, and keeps the order", () => {
    const made = withRooms(
      [room("l-claude", "Claude"), room("l-codex", "Codex")],
      [doc("a", "global")],
      [row("a")],
    );
    expect(made.rules.map((one) => one.level)).toEqual(["claude", "codex"]);
    expect(files(made.rules[0].docs)).toEqual(["a"]);
    expect(made.rules[1].docs).toEqual([]);
  });
});
