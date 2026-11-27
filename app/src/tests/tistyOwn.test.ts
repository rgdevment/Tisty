import { describe, expect, it } from "vitest";
import type { Wired } from "../core";
import { adds, ownOf, shown, tistyHookLines } from "../tistyOwn";
import { skillsFor, wiredOne } from "./knownFor";

const at = "/home/someone/hooks.json";
const hooksOn = { on: true, at };
const hooksOff = { on: false, at };
const door = { on: true, called: "espino 3" };
const landing = [
  { assistant: "claude", at: "/h/.claude/skills/tisty/SKILL.md", standing: "write" },
];

const agent = (id: string, hooks: Wired["hooks"], more: Partial<Wired> = {}) =>
  wiredOne(id, { wired: true, hooks, ...more });

const hooksFor = (on: boolean, more = {}) => ({ ...(on ? hooksOn : hooksOff), ...more });

const skillsOn = (more = {}) => skillsFor({ on: true, ...more });
const skillsOff = (more = {}) => skillsFor({ on: false, landing, ...more });

describe("tistyHookLines", () => {
  it("gives one line per connected agent with hooks, on when switched on or plugged", () => {
    expect(
      tistyHookLines([
        agent("claude", hooksFor(true)),
        agent("codex", hooksFor(false)),
        agent("gemini", { on: false, plugged: true, at }),
        agent("cursor", null),
        wiredOne("copilot", { wired: false, hooks: hooksOn }),
      ]),
    ).toEqual([
      { id: "claude", name: "claude", on: true },
      { id: "codex", name: "codex", on: false },
      { id: "gemini", name: "gemini", on: true },
    ]);
  });

  it("gives none when the agents are not read yet", () => {
    expect(tistyHookLines(null)).toEqual([]);
    expect(tistyHookLines([])).toEqual([]);
  });
});

describe("ownOf: the dot", () => {
  it("is off when skills are off and no hook is on", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(false))],
      skills: skillsOff(),
    });
    expect(own.dot).toBe("off");
    expect(own.skills).toEqual({ on: false, adds: true });
    expect(own.hooks?.lines).toEqual([{ id: "claude-code", name: "claude-code", on: false }]);
  });

  it("is partly with skills on and hooks off", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(false))],
      skills: skillsOn(),
    });
    expect(own.dot).toBe("partly");
    expect(own.skills).toEqual({ on: true, adds: false });
  });

  it("is partly with skills off and one hook on", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(true))],
      skills: skillsOff(),
    });
    expect(own.dot).toBe("partly");
  });

  it("is partly when the hooks are only in some agents", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(true)), agent("codex", hooksFor(false))],
      skills: skillsOn(),
    });
    expect(own.dot).toBe("partly");
    expect(own.hooks?.adds.map((one) => one.id)).toEqual(["codex"]);
  });

  it("is on with skills on and every hook on", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(true)), agent("codex", hooksFor(true))],
      skills: skillsOn(),
    });
    expect(own.dot).toBe("on");
    expect(adds(own)).toBe(false);
  });

  it("counts a plugged hook as on", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", { on: false, plugged: true, at })],
      skills: skillsOn(),
    });
    expect(own.dot).toBe("on");
  });

  it("counts skills off as on when every skilled connected agent has them plugged", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(true)), agent("codex", hooksFor(true))],
      skills: skillsOff({ plugged: ["claude", "codex"] }),
    });
    expect(own.skills).toEqual({ on: true, adds: false });
    expect(own.dot).toBe("on");
  });

  it("does not count them as on while one skilled agent lacks them", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(true)), agent("codex", hooksFor(true))],
      skills: skillsOff({ plugged: ["claude"] }),
    });
    expect(own.skills).toEqual({ on: false, adds: true });
    expect(own.dot).toBe("partly");
  });

  it("is on with no hooks to show when the skills alone are on", () => {
    const own = ownOf({ agent: door, agents: [], skills: skillsOn() });
    expect(own.hooks).toBeNull();
    expect(own.dot).toBe("on");
    expect(shown(own)).toBe(true);
  });
});

describe("ownOf: the door muted the hooks", () => {
  const muted = { ...door, hooks: false };

  it("never calls the hooks on, and adds none", () => {
    const own = ownOf({
      agent: muted,
      agents: [agent("claude-code", hooksFor(true)), agent("codex", hooksFor(false))],
      skills: skillsOn(),
    });
    expect(own.hooks?.muted).toBe(true);
    expect(own.hooks?.adds).toEqual([]);
    expect(own.dot).toBe("partly");
    expect(adds(own)).toBe(false);
  });

  it("is off with skills off", () => {
    const own = ownOf({
      agent: muted,
      agents: [agent("claude-code", hooksFor(true))],
      skills: skillsOff({ landing: [] }),
    });
    expect(own.dot).toBe("off");
  });

  it("still adds the skills", () => {
    const own = ownOf({
      agent: muted,
      agents: [agent("claude-code", hooksFor(false))],
      skills: skillsOff(),
    });
    expect(adds(own)).toBe(true);
    expect(own.hooks?.adds).toEqual([]);
  });
});

describe("ownOf: what may be added", () => {
  it("leaves out hooks that are plugin-unknown, apart or unread", () => {
    const own = ownOf({
      agent: door,
      agents: [
        agent("claude-code", hooksFor(false, { pluginUnknown: true })),
        agent("codex", hooksFor(false, { apart: true })),
        agent("cursor", hooksFor(false, { unread: true })),
        agent("gemini-cli", hooksFor(false)),
      ],
      skills: skillsOn(),
    });
    expect(own.hooks?.adds.map((one) => one.id)).toEqual(["gemini-cli"]);
    expect(own.hooks?.lines).toHaveLength(4);
  });

  it("leaves out hooks with stray records, and adds those with an empty stray list", () => {
    const own = ownOf({
      agent: door,
      agents: [
        agent("claude-code", hooksFor(false, { stray: [{ said: "/old/tisty", gone: true }] })),
        agent("codex", hooksFor(false, { stray: [] })),
      ],
      skills: skillsOn(),
    });
    expect(own.hooks?.adds.map((one) => one.id)).toEqual(["codex"]);
  });

  it("adds no hooks while the door is closed, without calling them muted", () => {
    const own = ownOf({
      agent: { on: false },
      agents: [agent("claude-code", hooksFor(false))],
      skills: skillsOn(),
    });
    expect(own.hooks?.adds).toEqual([]);
    expect(own.hooks?.muted).toBe(false);
    expect(adds(own)).toBe(false);
    expect(own.dot).toBe("partly");
  });

  it("still adds the skills while the door is closed", () => {
    const own = ownOf({
      agent: { on: false },
      agents: [agent("claude-code", hooksFor(false))],
      skills: skillsOff(),
    });
    expect(own.hooks?.adds).toEqual([]);
    expect(adds(own)).toBe(true);
  });

  it("adds nothing when every part is on, or the only missing part cannot be added", () => {
    const blocked = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(false, { pluginUnknown: true }))],
      skills: skillsOn(),
    });
    expect(adds(blocked)).toBe(false);
    expect(blocked.dot).toBe("partly");
  });

  it("adds the skills only when they are off and have somewhere to land", () => {
    expect(adds(ownOf({ agent: door, agents: [], skills: skillsOff() }))).toBe(true);
    expect(
      ownOf({ agent: door, agents: [], skills: skillsOff({ landing: [] }) }).skills,
    ).toBeNull();
  });
});

describe("ownOf: which agents count", () => {
  it("ignores agents in a place, astray or not connected", () => {
    const own = ownOf({
      agent: door,
      agents: [
        agent("claude-code", hooksFor(true)),
        agent("codex", hooksFor(false), { place: "Ubuntu" }),
        agent("cursor", hooksFor(false), { astray: true }),
        wiredOne("gemini-cli", { wired: false, hooks: hooksOff }),
      ],
      skills: skillsOn(),
    });
    expect(own.hooks?.lines.map((one) => one.id)).toEqual(["claude-code"]);
    expect(own.dot).toBe("on");
  });

  it("ignores a place's agent when counting plugged skills", () => {
    const own = ownOf({
      agent: door,
      agents: [
        agent("claude-code", hooksFor(true)),
        agent("codex", hooksFor(true), { place: "Ubuntu" }),
      ],
      skills: skillsOff({ plugged: ["claude"] }),
    });
    expect(own.skills?.on).toBe(true);
  });

  it("has no hooks part when no connected agent has hooks", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", null)],
      skills: skillsOn(),
    });
    expect(own.hooks).toBeNull();
  });
});

describe("ownOf: nothing to show", () => {
  it("has no skills part for a build with no skills", () => {
    const own = ownOf({
      agent: door,
      agents: [agent("claude-code", hooksFor(true))],
      skills: skillsOn({ skills: [] }),
    });
    expect(own.skills).toBeNull();
    expect(own.hooks).not.toBeNull();
  });

  it("is shown for neither part when skills are unread and there are no hooks", () => {
    for (const skills of [null, undefined]) {
      const own = ownOf({ agent: door, agents: null, skills });
      expect(own).toMatchObject({ skills: null, hooks: null });
      expect(shown(own)).toBe(false);
      expect(adds(own)).toBe(false);
    }
  });

  it("reads a missing door as not muted", () => {
    const own = ownOf({
      agent: null,
      agents: [agent("claude-code", hooksFor(false))],
      skills: skillsOn(),
    });
    expect(own.hooks?.muted).toBe(false);
    expect(own.hooks?.adds).toHaveLength(1);
  });
});
