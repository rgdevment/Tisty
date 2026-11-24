import { describe, expect, it } from "vitest";
import { type AgentDoc, carded, fencedFile, touched } from "../agentDocs";

const doc = (states: AgentDoc["places"][number]["said"][]): AgentDoc => ({
  kind: "rule",
  level: "global",
  named: null,
  name: "AGENTS.md",
  label: "AGENTS.md",
  code: false,
  folderOff: false,
  rivals: [],
  here: "applied",
  print: "p1",
  proposal: null,
  places: states.map((said, n) => ({
    assistant: `a${n}`,
    named: `A${n}`,
    at: `/home/a${n}/AGENTS.md`,
    said,
    shared: 1,
    heldBy: [],
    changed: null,
    bytes: null,
    kept: [],
  })),
  own: { about: null, fields: [], raw: null },
});

describe("agent documents", () => {
  it("are told by their card alone", () => {
    expect(carded("---\nagent: claude/rule/CLAUDE.md\n---\n# Rules\n")).toBe(true);
    expect(carded("---\ntitle: Rules\n---\nagent: not a card\n")).toBe(false);
    expect(carded("# agent: no front matter\n")).toBe(false);
    expect(carded("---\nagent: [claude/skill/pdf/SKILL.md, crlf, bom]\n---\n")).toBe(true);
    const marked = "agent: [claude/rule/CLAUDE.md, crlf, on-dev_ab12cd34]";
    expect(carded(`---\n${marked}\n---\n`)).toBe(true);
    for (const odd of [
      "[claude/rule/CLAUDE.md, on-]",
      "[claude/rule/CLAUDE.md, on-dev_a, on-dev_b]",
      "global/note/x.md",
      "Global/rule/x.md",
      "global/rule/",
      "[global/rule/x.md, odd]",
      "claude/rule/notes#1.md",
      "claude/rule/../CLAUDE.md",
      "claude/rule/a//b.md",
    ])
      expect(carded(`---\nagent: ${odd}\n---\n`)).toBe(false);
  });

  it("show a file in a fence longer than any it holds, named by its kind", () => {
    expect(fencedFile("echo hi", "/x/run.sh")).toBe("```sh\necho hi\n```\n");
    expect(fencedFile("a\n````\nb\n", "/x/SKILL.md")).toBe("`````markdown\na\n````\nb\n`````\n");
    expect(fencedFile("", "/x/noext")).toBe("```text\n```\n");
  });

  it("tell a file changed outside or on both sides from one that waits on nothing", () => {
    const places = doc([
      { state: "synced" },
      { state: "outside", mine: true, gone: false },
      { state: "both" },
      { state: "off" },
    ]).places;
    expect(places.filter(touched).map((one) => one.assistant)).toEqual(["a1", "a2"]);
  });
});
