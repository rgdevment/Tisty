import { act, render, renderHook, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  amiss,
  extrasIds,
  extrasOf,
  hushAmiss,
  hushedHere,
  lost,
  type Offering,
  unheeded,
} from "../agentsAmiss";
import { mendOne } from "../agentsMending";
import { offered, offeredHere, SKILLS_DECLINED } from "../agentsOffered";
import { AGENTS_CHANGED } from "../agentsSync";
import type { Wired } from "../core";
import { showAll } from "../hiding";
import { type KnownAgents, useKnownAgents } from "../knownAgents";
import { adopt, t } from "../locales";
import AgentsAmiss, { useUnheeded } from "../ui/AgentsAmiss";
import AgentsMending from "../ui/AgentsMending";
import { TistySkillsCard } from "../ui/TistySkills";
import { useExtras } from "../ui/useExtras";
import { IDLE } from "../ui/working";
import { knownFor, lostOne, skillsFor, wiredOne } from "./knownFor";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd, args ?? {});
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);
const ids = (cmd: string) => sent(cmd).map((one) => one.args.id);

const hooks = { on: true, at: "/home/someone/hooks.json" };
const lostHooks = { ...hooks, astray: true, gone: true, points: "/old/tisty" };
const noHooks = { ...hooks, on: false };
const open = { on: true, called: "espino 3" };

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
  ipc.answer = () => Promise.resolve(null);
  offeredHere.put([]);
  hushedHere.put({ said: [] });
  showAll();
});

describe("what is amiss with an agent", () => {
  it("is a connection or hooks that run another Tisty, and lost once that one is gone", () => {
    expect(amiss(wiredOne("a", { wired: true, hooks }))).toBe(false);
    expect(amiss(lostOne("a")) && lost(lostOne("a"))).toBe(true);
    expect(lost(lostOne("a", { gone: false }))).toBe(false);
    const byHooks = wiredOne("a", { wired: true, hooks: lostHooks });
    expect(amiss(byHooks) && lost(byHooks)).toBe(true);
    expect(amiss(wiredOne("a", { hooks: { ...lostHooks, on: false } }))).toBe(false);
  });

  it("is never a bare name, found on the PATH of whoever runs it, nor an agent that may be gone", () => {
    expect(amiss(wiredOne("a", { astray: true, points: "tisty" }))).toBe(false);
    expect(amiss(lostOne("a", { maybe: true }))).toBe(false);
  });

  it("stays quiet once put off, until something else breaks", () => {
    const all = [lostOne("a"), lostOne("b", { gone: false })];
    expect(unheeded(all, [])).toEqual([all[0]]);
    hushAmiss(all);
    expect(unheeded(all, hushedHere.current().said)).toEqual([]);
    const moved = [lostOne("a", { points: "/older/tisty" })];
    expect(unheeded(moved, hushedHere.current().said)).toEqual(moved);
    hushAmiss(moved);
    expect(unheeded([...all, ...moved], hushedHere.current().said)).toEqual([]);
    expect(unheeded(null, [])).toEqual([]);
  });
});

describe("what may be added", () => {
  const offering = (over: Partial<Offering> = {}): Offering => ({
    agent: open,
    agents: [wiredOne("claude-code", { wired: true, hooks: noHooks })],
    skills: skillsFor(),
    declined: [],
    skillsHidden: false,
    ...over,
  });
  const named = (over?: Partial<Offering>) => {
    const extras = extrasOf(offering(over));
    return extras && [extras.hooks.map((one) => one.id), extras.skills.map((one) => one.id)];
  };

  it("is offered only to a connected agent, with the door open", () => {
    expect(named()).toEqual([["claude-code"], ["claude-code"]]);
    expect(named({ agent: { on: false } })).toBeNull();
    expect(named({ agent: null })).toBeNull();
    expect(named({ agents: null })).toBeNull();
    expect(named({ agents: [wiredOne("claude-code", { hooks: noHooks })] })).toBeNull();
    expect(named({ agents: [lostOne("claude-code", { hooks: noHooks })] })).toBeNull();
  });

  it("waits until what is broken is fixed", () => {
    const able = wiredOne("claude-code", { wired: true, hooks: noHooks });
    expect(named({ agents: [able, lostOne("codex")] })).toBeNull();
    const landing = [{ assistant: "claude", at: "/s", standing: "write" }];
    expect(named({ skills: skillsFor({ on: true, landing }) })).toBeNull();
  });

  it("leaves out what is there already, what the plugin brings and what was turned down", () => {
    const agents = [wiredOne("claude-code", { wired: true, hooks: { ...noHooks, plugged: true } })];
    expect(named({ agents, skills: skillsFor({ plugged: ["claude"] }) })).toBeNull();
    expect(named({ skills: skillsFor({ on: true }) })).toEqual([["claude-code"], []]);
    expect(named({ skills: null })).toEqual([["claude-code"], []]);
    expect(named({ skills: skillsFor({ skills: [] }) })).toEqual([["claude-code"], []]);
    expect(named({ skillsHidden: true })).toEqual([["claude-code"], []]);
    expect(named({ declined: [SKILLS_DECLINED] })).toEqual([["claude-code"], []]);
    expect(named({ agent: { on: true, hooks: false } })).toEqual([[], ["claude-code"]]);
    expect(named({ declined: ["claude-code"] })).toBeNull();
  });

  it("is turned down for the agents it named, and for the skills as a whole", () => {
    const all = extrasOf(offering());
    expect(all && extrasIds(all)).toEqual(["claude-code", SKILLS_DECLINED]);
    const onlyHooks = extrasOf(offering({ skills: null }));
    expect(onlyHooks && extrasIds(onlyHooks)).toEqual(["claude-code"]);
  });

  it("remembers the skills as turned down once they are turned off by hand", async () => {
    ipc.answer = () => Promise.resolve(skillsFor());
    render(
      <TistySkillsCard skills={skillsFor({ on: true })} onTaken={() => {}} onClose={() => {}} />,
    );
    await userEvent.click(screen.getByRole("button", { name: t("tistySkillsRemove") }));
    expect(offered()).toEqual([SKILLS_DECLINED]);
  });
});

describe("fixing one agent", () => {
  it("takes away its own hooks when the plugin brings them too, so they never run twice", async () => {
    ipc.answer = (cmd) =>
      Promise.resolve([
        wiredOne("claude-code", { wired: true, hooks: { ...hooks, on: cmd !== "unhook" } }),
      ]);
    const { said } = await mendOne(
      lostOne("claude-code", { hooks: { ...lostHooks, plugged: true } }),
    );

    expect(ids("wire")).toEqual(["claude-code"]);
    expect(ids("unhook")).toEqual(["claude-code"]);
    expect(said).toEqual({
      id: "claude-code",
      name: "claude-code",
      parts: ["connection", "hooks"],
    });
  });

  it("never connects an agent that has only its hooks left", async () => {
    ipc.answer = () => Promise.resolve([wiredOne("cursor", { hooks })]);
    await mendOne(wiredOne("cursor", { hooks: lostHooks }));

    expect(sent("wire")).toHaveLength(0);
    expect(ids("hook")).toEqual(["cursor"]);
  });

  it("says so when it still points at another Tisty afterwards", async () => {
    ipc.answer = () => Promise.resolve([lostOne("cursor")]);
    expect((await mendOne(lostOne("cursor"))).said.why).toBe("it still points at another Tisty");
  });
});

const quietly = <T,>(
  _card: string,
  work: Promise<T> | (() => Promise<T>),
  then: (answer: T) => void,
) => {
  void (typeof work === "function" ? work() : work).then(then);
};

function Page({ known }: Readonly<{ known: KnownAgents }>) {
  return <AgentsMending known={known} busy={IDLE} quietly={quietly as never} />;
}

describe("the agents' page", () => {
  it("is quiet with nothing to fix or add", () => {
    render(
      <Page
        known={knownFor({
          agent: open,
          agents: [wiredOne("codex", { wired: true })],
          skills: null,
        })}
      />,
    );
    expect(screen.queryByRole("region")).toBeNull();
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("fixes everything broken with one button, goes on past the one that fails and says each", async () => {
    const look = vi.fn(() => Promise.resolve());
    ipc.answer = (cmd, args) => {
      if (cmd === "wire" && args.id === "codex")
        return Promise.reject({ code: "settingsPuzzling", name: "/home/someone/config.toml" });
      if (cmd === "agents_tisty_round") return Promise.resolve(skillsFor({ on: true }));
      return Promise.resolve([wiredOne(String(args.id), { wired: true, hooks })]);
    };
    const landing = [{ assistant: "claude", at: "/s", standing: "write" }];
    render(
      <Page
        known={knownFor({
          agent: open,
          agents: [
            lostOne("codex"),
            lostOne("claude-code", { hooks: lostHooks }),
            wiredOne("cursor", { wired: true, hooks: lostHooks }),
            wiredOne("zed", { wired: true }),
          ],
          skills: skillsFor({ on: true, landing }),
          lookAgents: look,
        })}
      />,
    );

    const todo = within(screen.getByRole("region", { name: "3 agents to fix" }));
    expect(todo.getByText("claude-code").parentElement?.textContent).toContain(
      "connection and hooks",
    );
    expect(todo.getAllByText("Points at /old/tisty, which is no longer there")).toHaveLength(3);
    expect(todo.getByText(/\.before-tisty/)).toBeTruthy();
    expect(ipc.calls).toHaveLength(0);

    await userEvent.click(todo.getByRole("button", { name: "Fix all" }));

    const told = within(await screen.findByRole("status"));
    expect(told.getByText("Some could not be fixed")).toBeTruthy();
    expect(told.getAllByText("✓")).toHaveLength(3);
    expect(told.getByText("✕")).toBeTruthy();
    expect(told.getByText("codex").parentElement?.textContent).toContain("config.toml");
    expect(told.getByText("Quit each one completely and open it again.")).toBeTruthy();
    expect(ids("wire")).toEqual(["codex", "claude-code"]);
    expect(ids("hook")).toEqual(["cursor"]);
    expect(sent("agents_tisty_round")).toHaveLength(1);
    expect(look).toHaveBeenCalled();

    await userEvent.click(told.getByRole("button", { name: "Got it" }));
    expect(screen.queryByRole("status")).toBeNull();
  });

  it.each([
    [() => Promise.resolve(skillsFor({ on: true })), "Fixed"],
    [() => Promise.reject({ code: "cannotWrite", name: "disk" }), "Some could not be fixed"],
  ])("brings the skills up to date on their own when only they are behind", async (round, said) => {
    ipc.answer = (cmd) => (cmd === "agents_tisty_round" ? round() : Promise.resolve(null));
    const landing = [{ assistant: "claude", at: "/s", standing: "write" }];
    render(
      <Page
        known={knownFor({
          agent: open,
          skills: skillsFor({ on: true, landing }),
        })}
      />,
    );
    const todo = screen.getByRole("region", { name: "Tisty's skills are out of date" });
    await userEvent.click(within(todo).getByRole("button", { name: "Fix" }));
    expect(await screen.findByText(said)).toBeTruthy();
  });

  const able = () => [
    wiredOne("claude-code", { name: "Claude Code", wired: true, hooks: noHooks }),
    wiredOne("codex", { name: "Codex", wired: true }),
  ];

  it("offers hooks and skills once, writing only what stays ticked after saying where", async () => {
    ipc.answer = () => Promise.resolve([]);
    const landing = [
      { assistant: "codex", at: "/home/someone/.agents/skills/tisty/SKILL.md", standing: "write" },
    ];
    render(
      <Page known={knownFor({ agent: open, agents: able(), skills: skillsFor({ landing }) })} />,
    );

    const offer = within(screen.getByRole("region", { name: t("extrasTitle") }));
    expect(offer.getByText("Hooks for Claude Code")).toBeTruthy();
    expect(offer.getByText("Tisty's skills for Claude Code, Codex")).toBeTruthy();
    await userEvent.click(offer.getByRole("button", { name: "Add hooks and turn on skills" }));
    expect(offer.getByText(/Tisty will add its hooks to \/home\/someone\/hooks.json/)).toBeTruthy();
    expect(offer.getByText(/installed_plugins.json/)).toBeTruthy();
    expect(offer.getByText("/home/someone/.agents/skills")).toBeTruthy();
    expect(document.activeElement).toBe(offer.getByRole("checkbox", { name: /^Hooks/ }));
    expect(ipc.calls).toHaveLength(0);

    await userEvent.click(offer.getByRole("checkbox", { name: /^Tisty's skills/ }));
    await userEvent.click(offer.getByRole("button", { name: "Add hooks" }));

    expect(await screen.findByText("Done")).toBeTruthy();
    expect(ids("hook")).toEqual(["claude-code"]);
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
    expect(offered()).toEqual(["claude-code", "codex", SKILLS_DECLINED]);
  });

  it("turns the skills on here, and says what could not be added", async () => {
    ipc.answer = (cmd) =>
      cmd === "hook"
        ? Promise.reject({ code: "cannotWrite", name: "disk" })
        : Promise.resolve(skillsFor({ on: true }));
    render(<Page known={knownFor({ agent: open, agents: able(), skills: skillsFor() })} />);
    await userEvent.click(screen.getByRole("button", { name: "Add hooks and turn on skills" }));
    await userEvent.click(screen.getByRole("button", { name: "Add hooks and turn on skills" }));

    expect(await screen.findByText("Some could not be done")).toBeTruthy();
    expect(sent("agents_tisty_skills_take")[0].args.on).toBe(true);
    expect(screen.getByText("Tisty's skills").parentElement?.textContent).toBe(
      "Tisty's skills · on",
    );
  });

  it.each(["foreign", "write"])(
    "never says the skills were added where one was left as it is",
    async (standing) => {
      const landing = [{ assistant: "claude", at: "/s", standing }];
      ipc.answer = () => Promise.resolve(skillsFor({ on: true, landing }));
      const agents = [wiredOne("codex", { wired: true })];
      render(<Page known={knownFor({ agent: open, agents, skills: skillsFor() })} />);
      await userEvent.click(screen.getByRole("button", { name: "Turn on skills" }));
      await userEvent.click(screen.getByRole("button", { name: "Turn on skills" }));
      expect(await screen.findByText("Some could not be done")).toBeTruthy();
    },
  );

  it.each([
    [[wiredOne("codex", { wired: true })], skillsFor(), "Turn on skills"],
    [able(), null, "Add hooks"],
  ])("names on its button only what there is to add", async (agents, skills, said) => {
    render(<Page known={knownFor({ agent: open, agents, skills })} />);
    expect(screen.getByRole("button", { name: said })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByRole("region")).toBeNull();
    expect(offered().length).toBeGreaterThan(0);
  });

  it("takes the keyboard there when it was reached from a notice", () => {
    render(
      <AgentsMending
        known={knownFor({ agents: [lostOne("codex")] })}
        busy={IDLE}
        quietly={quietly as never}
        sentAt={1}
      />,
    );
    expect(document.activeElement?.textContent).toContain("1 agent to fix");
  });
});

function Notice({
  all,
  onOpen,
  agent = open,
}: Readonly<{ all: Wired[]; onOpen: () => void; agent?: { on: boolean } }>) {
  const amissNow = useUnheeded(knownFor({ agent, agents: all }));
  return <AgentsAmiss amiss={amissNow} onOpen={onOpen} />;
}

describe("the notice in the window", () => {
  it.each([
    [[lostOne("Codex")], "Codex can no longer find Tisty"],
    [
      [wiredOne("Codex", { wired: true, hooks: lostHooks })],
      "Codex's hooks can no longer find Tisty",
    ],
    [[lostOne("Codex"), lostOne("Cursor")], "2 agents can no longer find Tisty"],
  ])("says who can no longer find Tisty, and leads there", async (all, said) => {
    const onOpen = vi.fn();
    render(<Notice all={all} onOpen={onOpen} />);
    expect(within(screen.getByRole("status")).getByText(said)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "See and fix" }));
    expect(onOpen).toHaveBeenCalled();
  });

  it("goes on «Not now», and says nothing of one that points at another Tisty still there", async () => {
    const { rerender } = render(<Notice all={[lostOne("Codex")]} onOpen={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByRole("status")).toBeNull();
    rerender(<Notice all={[lostOne("Zed", { gone: false })]} onOpen={() => {}} />);
    expect(screen.queryByRole("status")).toBeNull();
    rerender(<Notice all={[lostOne("Cursor")]} onOpen={() => {}} agent={{ on: false }} />);
    expect(screen.queryByRole("status")).toBeNull();
  });
});

describe("what the window knows of the agents", () => {
  it("looks at them only with the door open, and reads the skills once their round is done", async () => {
    let door = { on: false };
    ipc.answer = (cmd) => {
      if (cmd === "agent") return Promise.resolve(door);
      if (cmd === "wiring") return Promise.resolve([lostOne("codex")]);
      if (cmd === "agents_tisty_skills") return Promise.resolve(skillsFor({ on: true }));
      return Promise.resolve([]);
    };
    const { result } = renderHook(() => useKnownAgents());
    await act(() => result.current.lookIfOpen());
    expect(sent("wiring")).toHaveLength(0);

    door = { on: true };
    await act(() => result.current.lookIfOpen());
    expect(result.current.agents).toHaveLength(1);
    expect(sent("agents_tisty_skills")).toHaveLength(0);

    act(() => {
      window.dispatchEvent(new Event(AGENTS_CHANGED));
    });
    await waitFor(() => expect(result.current.skills?.on).toBe(true));
    await act(() => result.current.lookHands());
    expect(result.current.hands).toEqual([]);
  });

  it("asks nothing while another one already watches", () => {
    renderHook(() => useKnownAgents(true));
    window.dispatchEvent(new Event(AGENTS_CHANGED));
    expect(sent("agents_tisty_skills")).toHaveLength(0);
  });

  it("offers nothing more once declined", () => {
    const known = knownFor({
      agent: open,
      agents: [wiredOne("codex", { wired: true })],
      skills: skillsFor(),
    });
    const { result } = renderHook(() => useExtras(known));
    expect(result.current.extras?.skills).toHaveLength(1);
    act(() => {
      result.current.decline();
    });
    expect(result.current.extras).toBeNull();
  });
});
