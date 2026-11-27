import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentGroups, RuleTarget } from "../agentGroups";
import type { Filed, Wired } from "../core";
import { adopt } from "../locales";
import AgentGroupRows, { groupsDrawn } from "../ui/AgentGroupRows";

const ipc = vi.hoisted(() => ({
  skills: (): Promise<unknown> =>
    Promise.resolve({
      version: "1.50.0",
      on: true,
      leftover: [],
      skills: [],
      landing: [],
      plugged: [],
    }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => (cmd === "agents_tisty_skills" ? ipc.skills() : Promise.resolve(null)),
}));

const agent = (id: string, name: string, more: Partial<Wired> = {}): Wired => ({
  id,
  name,
  at: `/home/${id}`,
  wired: true,
  astray: false,
  maybe: false,
  byHand: false,
  manual: { file: `${id}.json`, snippet: "{}" },
  hooks: { on: true, at: `/home/${id}/hooks.json` },
  ...more,
});

const doc = (file: string): Filed => ({
  id: `id-${file}`,
  file,
  title: file,
  folder: "global",
  archived: false,
  away: false,
  wrote: "2026-11-19T10:00:00Z",
});

const target = (level: string, named: string): RuleTarget => ({ level, named });

const NONE: AgentGroups = {
  rules: [],
  ruleTargets: [],
  skills: [],
  hooks: [],
  left: [],
  emptied: new Set(),
};

const CLAUDE = target("claude", "Claude");
const CODEX = target("codex", "Codex");

const drawn = (
  groups: Partial<AgentGroups>,
  more: Partial<Parameters<typeof AgentGroupRows>[0]> = {},
) =>
  render(
    <ul>
      <AgentGroupRows
        groups={{ ...NONE, ...groups }}
        depth={0}
        step={12}
        paper={(one) => <li key={one.id}>{one.title}</li>}
        skill={() => null}
        rowOf={(key) => ({ "data-row": key }) as never}
        tistySkills={false}
        agents={[]}
        {...more}
      />
    </ul>,
  );

const follows = (a: Node, b: Node) =>
  Boolean(a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING);

beforeEach(() => {
  adopt("en");
  ipc.skills = () =>
    Promise.resolve({
      version: "1.50.0",
      on: true,
      leftover: [],
      skills: [],
      landing: [],
      plugged: [],
    });
});

describe("the plus of Rules", () => {
  it("goes straight to the rule when one agent can take it", () => {
    const onNewRule = vi.fn();
    drawn({ ruleTargets: [CODEX] }, { onNewRule });
    const plus = screen.getByRole("button", { name: "New rule for Codex, for all your computers" });
    expect(plus.getAttribute("data-row")).toBe("rules:new");
    fireEvent.click(plus);
    expect(onNewRule).toHaveBeenCalledExactlyOnceWith("codex");
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("opens a menu of the agents by name when several can, and picking one makes its rule", async () => {
    const onNewRule = vi.fn();
    drawn({ ruleTargets: [CLAUDE, CODEX] }, { onNewRule });
    const plus = screen.getByRole("button", { name: "New rule, for all your computers" });
    expect(screen.getAllByRole("button", { name: /New rule/ })).toHaveLength(1);
    fireEvent.click(plus);
    expect(onNewRule).not.toHaveBeenCalled();
    const menu = await screen.findByRole("menu", { name: "New rule, for all your computers" });
    const names = screen.getAllByRole("menuitem").map((one) => one.textContent);
    expect(names).toEqual(["Claude", "Codex"]);
    expect(menu).toBeTruthy();
    fireEvent.click(screen.getByRole("menuitem", { name: "Codex" }));
    expect(onNewRule).toHaveBeenCalledExactlyOnceWith("codex");
  });

  it("speaks Spanish", () => {
    adopt("es");
    drawn({ ruleTargets: [CLAUDE, CODEX] }, { onNewRule: vi.fn() });
    expect(screen.getByText("Reglas")).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Nueva regla, para todos tus equipos" }),
    ).toBeTruthy();
  });

  it("draws neither heading nor plus when no agent can take a rule and none holds one", () => {
    drawn({}, { onNewRule: vi.fn() });
    expect(screen.queryByText("Rules")).toBeNull();
    expect(screen.queryAllByRole("button")).toHaveLength(0);
  });

  it("draws the heading without a plus when there is no way to make a rule", () => {
    drawn({
      ruleTargets: [CLAUDE],
      rules: [{ level: "claude", named: "Claude", docs: [doc("a")] }],
    });
    expect(screen.getByText("Rules")).toBeTruthy();
    expect(screen.queryAllByRole("button")).toHaveLength(0);
  });

  it("draws no Rules heading for agents that could take a rule but hold none", () => {
    drawn({ ruleTargets: [CLAUDE, CODEX] });
    expect(screen.queryByText("Rules")).toBeNull();
  });
});

describe("the rules under the plus", () => {
  it("draws a subheading and its rules only for the agents that hold some", () => {
    drawn(
      {
        ruleTargets: [CLAUDE, CODEX],
        rules: [{ level: "codex", named: "Codex", docs: [doc("horario"), doc("estilo")] }],
      },
      { onNewRule: vi.fn() },
    );
    const sub = screen.getByText("Codex", { selector: "li" });
    expect(sub.closest("[data-row]")).toBeNull();
    expect(screen.queryByText("Claude", { selector: "li" })).toBeNull();
    expect(follows(screen.getByText("Rules"), sub)).toBe(true);
    expect(follows(sub, screen.getByText("horario"))).toBe(true);
    expect(follows(screen.getByText("horario"), screen.getByText("estilo"))).toBe(true);
  });

  it("has no per-agent plus", () => {
    drawn(
      {
        ruleTargets: [CLAUDE, CODEX],
        rules: [
          { level: "claude", named: "Claude", docs: [doc("a")] },
          { level: "codex", named: "Codex", docs: [doc("b")] },
        ],
      },
      { onNewRule: vi.fn() },
    );
    const keys = screen.getAllByRole("button").map((one) => one.getAttribute("data-row"));
    expect(keys).toEqual(["rules:new"]);
  });
});

describe("Skills, Hooks and From Tisty", () => {
  const lone = { agents: [agent("claude", "Claude")], onHooks: vi.fn() };

  it("draws the Skills heading only when the person has skills", () => {
    const first = drawn({}, { tistySkills: true });
    expect(screen.queryByText("Skills")).toBeNull();
    first.unmount();
    drawn({ skills: [{ doc: doc("solo") }] });
    expect(screen.getByText("Skills")).toBeTruthy();
    expect(screen.getByText("solo")).toBeTruthy();
  });

  it("puts From Tisty last, after Hooks, with Tisty's skills and then Tisty's hooks", async () => {
    drawn({ hooks: [doc("guard")] }, { tistySkills: true, onNewHook: vi.fn(), ...lone });
    const hooks = screen.getByText("Hooks");
    const own = screen.getByText("guard");
    const from = screen.getByText("From Tisty");
    const skills = await screen.findByText("Tisty's skills");
    const tisty = screen.getByText("Tisty's hooks");
    expect(follows(hooks, own)).toBe(true);
    expect(follows(own, from)).toBe(true);
    expect(follows(from, skills)).toBe(true);
    expect(follows(skills, tisty)).toBe(true);
    expect(skills.closest("button")?.getAttribute("data-row")).toBe("tisty-skills");
    expect(tisty.closest("button")?.getAttribute("data-row")).toBe("tisty-hooks");
  });

  it("keeps Tisty's hooks out of the Hooks heading, which has its own plus", () => {
    const onNewHook = vi.fn();
    drawn({}, { onNewHook, ...lone });
    const plus = screen.getByRole("button", { name: "New hook" });
    expect(plus.getAttribute("data-row")).toBe("hooks:new");
    fireEvent.click(plus);
    expect(onNewHook).toHaveBeenCalledOnce();
    expect(follows(screen.getByText("Hooks"), screen.getByText("From Tisty"))).toBe(true);
  });

  it("draws From Tisty for Tisty's skills alone, or for Tisty's hooks alone", async () => {
    const skills = drawn({}, { tistySkills: true });
    await screen.findByText("Tisty's skills");
    expect(screen.getByText("From Tisty")).toBeTruthy();
    expect(screen.queryByText("Tisty's hooks")).toBeNull();
    skills.unmount();
    drawn({}, lone);
    expect(screen.getByText("From Tisty")).toBeTruthy();
    expect(screen.getByText("Tisty's hooks")).toBeTruthy();
    expect(screen.queryByText("Tisty's skills")).toBeNull();
  });

  it("holds the From Tisty heading back until the skills row has an answer", async () => {
    let answer: (now: unknown) => void = () => undefined;
    ipc.skills = () => new Promise((done) => (answer = done));
    drawn({}, { tistySkills: true });
    expect(screen.queryByText("From Tisty")).toBeNull();
    expect(screen.queryByText("Tisty's skills")).toBeNull();
    answer({
      version: "1.50.0",
      on: true,
      leftover: [],
      skills: [],
      landing: [],
      plugged: [],
    });
    const row = await screen.findByText("Tisty's skills");
    expect(follows(screen.getByText("From Tisty"), row)).toBe(true);
  });

  it("draws no From Tisty when the skills cannot be read", async () => {
    ipc.skills = () => Promise.reject(new Error("unreadable"));
    drawn({}, { tistySkills: true });
    await new Promise((done) => setTimeout(done, 20));
    expect(screen.queryByText("From Tisty")).toBeNull();
    expect(screen.queryByText("Tisty's skills")).toBeNull();
  });

  it("draws no From Tisty when Tisty has neither skills nor hooks to show", () => {
    drawn(
      { hooks: [doc("guard")] },
      { agents: [agent("claude", "Claude", { hooks: null })], onHooks: vi.fn() },
    );
    expect(screen.queryByText("From Tisty")).toBeNull();
  });

  it("draws no Tisty hooks row without a way to open them", () => {
    drawn({}, { agents: [agent("claude", "Claude")] });
    expect(screen.queryByText("From Tisty")).toBeNull();
    expect(screen.queryByText("Tisty's hooks")).toBeNull();
  });

  it("says From Tisty in Spanish", () => {
    adopt("es");
    drawn({}, lone);
    expect(screen.getByText("De Tisty")).toBeTruthy();
    expect(screen.getByText("Hooks de Tisty")).toBeTruthy();
  });
});

describe("groupsDrawn", () => {
  const agents = [agent("claude", "Claude")];

  it("is false for nothing, true for each thing Global can show", () => {
    expect(groupsDrawn(NONE, false, [], false)).toBe(false);
    expect(groupsDrawn({ ...NONE, ruleTargets: [CLAUDE] }, false, [], false)).toBe(true);
    expect(
      groupsDrawn(
        { ...NONE, rules: [{ level: "claude", named: "Claude", docs: [] }] },
        false,
        [],
        false,
      ),
    ).toBe(true);
    expect(groupsDrawn({ ...NONE, skills: [{ doc: doc("s") }] }, false, [], false)).toBe(true);
    expect(groupsDrawn({ ...NONE, hooks: [doc("h")] }, false, [], false)).toBe(true);
    expect(groupsDrawn(NONE, true, [], false)).toBe(true);
    expect(groupsDrawn(NONE, false, agents, true)).toBe(true);
    expect(groupsDrawn(NONE, false, agents, false)).toBe(false);
  });
});
