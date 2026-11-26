import { describe, expect, it } from "vitest";
import type { AgentRow, FileSeen } from "../agentsSync";
import { skillDirs, skillOf } from "../skillFolders";
import { offeredOf } from "../ui/AgentsFirst";
import { nested } from "../ui/agentRows";

const seen = (name: string): FileSeen => ({
  at: `/h/.agents/skills/${name}`,
  name,
  kind: "skill",
  skip: null,
  held: false,
});

const row = (file: string, name: string, more: Partial<AgentRow> = {}): AgentRow => ({
  file,
  everyone: false,
  kind: "skill",
  level: "codex",
  name,
  label: name.split("/").pop() ?? name,
  skill: name.split("/")[0],
  ...more,
});

describe("the folder a skill lives in", () => {
  it("is the folder of its SKILL.md, however deep its category puts it", () => {
    const dirs = skillDirs(["trabajo/pr/SKILL.md", "pr/skill.md", "pr/examples/mini/SKILL.md"]);
    expect(dirs).toEqual(["pr", "trabajo/pr", "pr/examples/mini"]);
    expect(skillOf("trabajo/pr/scripts/run.sh", dirs)).toBe("trabajo/pr");
    expect(skillOf("pr/examples/mini/SKILL.md", dirs)).toBe("pr");
    expect(skillOf("suelto/notes.md", dirs)).toBe("suelto");
  });

  it("makes one row in the first-time list for each skill in a category", () => {
    const rows = offeredOf([
      seen("trabajo/pr/SKILL.md"),
      seen("trabajo/pr/plantilla.md"),
      seen("trabajo/review/SKILL.md"),
    ]);
    expect(rows.map((one) => [one.name, one.files.length])).toEqual([
      ["trabajo/pr", 2],
      ["trabajo/review", 1],
    ]);
  });

  it("draws each skill of a category apart in the sidebar, named by its own folder", () => {
    const rows = new Map([
      ["a", row("a", "trabajo/pr/SKILL.md", { label: "trabajo" })],
      ["b", row("b", "trabajo/pr/plantilla.md")],
      ["c", row("c", "trabajo/review/SKILL.md", { label: "Revisar PRs" })],
      ["d", row("d", "pr/SKILL.md", { level: "claude" })],
      ["e", row("e", "CLAUDE.md", { kind: "rule", skill: null, level: "claude" })],
    ]);
    const placed = nested(rows);
    expect(placed.get("a")).toMatchObject({ skill: "trabajo/pr", label: "pr" });
    expect(placed.get("b")).toMatchObject({ skill: "trabajo/pr" });
    expect(placed.get("c")).toMatchObject({ skill: "trabajo/review", label: "Revisar PRs" });
    expect(placed.get("d")).toBe(rows.get("d"));
    expect(placed.get("e")).toBe(rows.get("e"));
  });
});
