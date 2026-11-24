import { act, render, renderHook, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useEffect } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  allParts,
  extrasIds,
  extrasOf,
  filesOf,
  forgetHealed,
  hush,
  hushedHere,
  inState,
  type Offering,
  partsIn,
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
import { knownFor, lostOne, skillsFor, strayHooks, wiredOne } from "./knownFor";

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

const HOOKS_AT = "/home/someone/hooks.json";
const hooksOn = { on: true, at: HOOKS_AT };
const noHooks = { on: false, at: HOOKS_AT };
const open = { on: true, called: "espino 3" };
const landing = [
  { assistant: "claude", at: "/home/someone/.claude/skills/tisty/SKILL.md", standing: "write" },
  { assistant: "codex", at: "/home/someone/.agents/skills/tisty/SKILL.md", standing: "write" },
];
const skillsHere = (more = {}) => skillsFor({ landing, ...more });

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
  ipc.answer = () => Promise.resolve(null);
  offeredHere.put([]);
  hushedHere.put({ said: [] });
  showAll();
});

describe("what is wrong with an agent", () => {
  it("is lost when a part leads to a Tisty that is gone, and elsewhere when it leads to another", () => {
    const healthy = wiredOne("a", { wired: true, hooks: hooksOn });
    expect(inState([healthy], "lost")).toEqual([]);
    expect(inState([lostOne("a")], "lost")).toHaveLength(1);
    expect(inState([lostOne("a", { gone: false })], "elsewhere")).toHaveLength(1);
    const byHooks = wiredOne("a", { wired: true, hooks: strayHooks() });
    expect(partsIn(byHooks, "lost")).toEqual({ connection: undefined, hooks: ["/old/tisty"] });
    expect(inState([byHooks], "elsewhere")).toEqual([]);
  });

  it("is never said of one whose own writing nothing here can tell, nor of one no longer here", () => {
    const apart = wiredOne("a", { apart: true, points: "tisty", hooks: strayHooks() });
    expect(partsIn(apart, "lost").connection).toBeUndefined();
    expect(inState([lostOne("a", { installed: false })], "lost")).toEqual([]);
    expect(inState([lostOne("a", { maybe: true })], "lost")).toEqual([]);
    expect(inState(null, "lost")).toEqual([]);
  });

  it("names every file a fix writes, once", () => {
    const both = lostOne("gemini", { at: "/s.json", hooks: { ...strayHooks(), at: "/s.json" } });
    expect(filesOf(both, allParts(both))).toEqual(["/s.json"]);
    const two = lostOne("claude", { at: "/a.json", hooks: strayHooks() });
    expect(filesOf(two, allParts(two))).toEqual(["/a.json", HOOKS_AT]);
    expect(filesOf(two, { hooks: [] })).toEqual([]);
  });

  it("stays quiet once put off, and is said again once it healed and broke the same way", () => {
    const all = [lostOne("a")];
    expect(unheeded(all, "lost", hushedHere.current().said)).toEqual(all);
    hush(all, "lost");
    expect(unheeded(all, "lost", hushedHere.current().said)).toEqual([]);
    forgetHealed(all);
    expect(unheeded(all, "lost", hushedHere.current().said)).toEqual([]);
    const moved = [lostOne("a", { points: "/older/tisty" })];
    expect(unheeded(moved, "lost", hushedHere.current().said)).toEqual(moved);

    for (const away of [
      [],
      [wiredOne("a", { unread: true })],
      [lostOne("a", { installed: false })],
      [wiredOne("a", { wired: true, hooks: { ...noHooks, unread: true } })],
    ])
      forgetHealed(away);
    expect(unheeded(all, "lost", hushedHere.current().said)).toEqual([]);

    forgetHealed([wiredOne("a", { wired: true })]);
    expect(unheeded(all, "lost", hushedHere.current().said)).toEqual(all);
  });
});

describe("what may be added", () => {
  const offering = (over: Partial<Offering> = {}): Offering => ({
    agent: open,
    agents: [wiredOne("claude-code", { wired: true, hooks: noHooks })],
    skills: skillsHere(),
    declined: [],
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
  });

  it("waits only for what was lost, never for one that leads to another Tisty or skills to bring up to date", () => {
    const able = wiredOne("claude-code", { wired: true, hooks: noHooks });
    expect(named({ agents: [able, lostOne("codex")] })).toBeNull();
    expect(named({ agents: [able, lostOne("codex", { gone: false })] })).not.toBeNull();
    expect(named({ skills: skillsHere({ on: true }) })).toEqual([["claude-code"], []]);
  });

  it("leaves out what is there, what cannot be read, what the plugin brings and what was turned down", () => {
    const agents = (hooks: object) => [
      wiredOne("claude-code", { wired: true, hooks: { ...noHooks, ...hooks } }),
    ];
    expect(
      named({ agents: agents({ plugged: true }), skills: skillsHere({ plugged: ["claude"] }) }),
    ).toBeNull();
    expect(named({ agents: agents({ unread: true }), skills: null })).toBeNull();
    expect(named({ skills: skillsFor() })).toEqual([["claude-code"], []]);
    expect(named({ declined: [SKILLS_DECLINED] })).toEqual([["claude-code"], []]);
    expect(named({ declined: ["claude-code#hooks"] })).toEqual([[], ["claude-code"]]);
    expect(named({ agent: { on: true, hooks: false } })).toEqual([[], ["claude-code"]]);
    expect(named({ declined: ["claude-code"] })).toBeNull();
  });

  it("turns down the hooks and the skills each on their own", () => {
    const both = extrasOf(offering());
    expect(both && extrasIds(both)).toEqual(["claude-code#hooks", SKILLS_DECLINED]);
    const onlyHooks = extrasOf(offering({ skills: null }));
    expect(onlyHooks && extrasIds(onlyHooks)).toEqual(["claude-code#hooks"]);
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

const fixed = (id: string) => ({ seen: [wiredOne(id, { wired: true, hooks: hooksOn })] });

describe("fixing one agent", () => {
  const asked = (one: Wired) => ({ one, parts: partsIn(one, "lost") });

  it("asks for exactly the parts it named, and takes out hooks the plugin brings too", async () => {
    ipc.answer = () => Promise.resolve(fixed("claude-code"));
    const one = lostOne("claude-code", { hooks: { ...strayHooks(), plugged: true } });
    const { said } = await mendOne(asked(one));

    expect(sent("mend")[0].args).toEqual({
      id: "claude-code",
      ask: { connection: "/old/tisty", hooks: ["/old/tisty"], hooksOut: true },
    });
    expect(said.done).toBe("connection · hooks removed, the plugin brings them");

    const alone = wiredOne("claude-code", {
      wired: true,
      hooks: { ...strayHooks(), plugged: true },
    });
    expect((await mendOne(asked(alone))).said.done).toBe("hooks removed, the plugin brings them");
  });

  it("leaves a connection of the person's own writing out of what it asks", async () => {
    ipc.answer = () => Promise.resolve(fixed("cursor"));
    const one = wiredOne("cursor", { apart: true, points: "tisty", hooks: strayHooks() });
    await mendOne(asked(one));

    expect(sent("mend")[0].args.ask).toEqual({
      connection: undefined,
      hooks: ["/old/tisty"],
      hooksOut: false,
    });
  });

  it("says fixed only of what reads fixed afterwards", async () => {
    const one = lostOne("codex", { hooks: strayHooks() });
    ipc.answer = () => Promise.resolve({ seen: [wiredOne("codex", { hooks: strayHooks() })] });
    const { said } = await mendOne(asked(one));

    expect(said.done).toBeUndefined();
    expect(said.failed).toEqual([
      { what: "connection", why: "it did not end up fixed" },
      { what: "hooks", why: "it did not end up fixed" },
    ]);
  });

  it("counts each half on its own when one cannot be written", async () => {
    const one = lostOne("claude-code", { hooks: strayHooks() });
    ipc.answer = () =>
      Promise.resolve({
        seen: [wiredOne("claude-code", { wired: true, hooks: strayHooks() })],
        hooks: { code: "hooksPuzzling", name: "/home/someone/settings.json" },
      });
    const { said } = await mendOne(asked(one));

    expect(said.done).toBe("connection");
    expect(said.failed?.[0].what).toBe("hooks");
    expect(said.failed?.[0].why).toContain("/home/someone/settings.json");
    expect(said.failed?.[0].why).not.toContain("By hand");
  });

  it("says why when nothing could be done at all", async () => {
    ipc.answer = () => Promise.reject({ code: "noSuchAgent" });
    const { said, seen } = await mendOne(asked(lostOne("codex")));
    expect(seen).toBeNull();
    expect(said.failed?.[0].why).toBe(t("noSuchAgent"));
  });
});

const quietly = <T,>(
  _card: string,
  work: Promise<T> | (() => Promise<T>),
  then: (answer: T) => void,
) => {
  void (typeof work === "function" ? work() : work).then(then);
};

function Page({ known, sentAt }: Readonly<{ known: KnownAgents; sentAt?: number }>) {
  return <AgentsMending known={known} busy={IDLE} quietly={quietly as never} sentAt={sentAt} />;
}

function Window() {
  const known = useKnownAgents();
  const { lookIfOpen } = known;
  useEffect(() => void lookIfOpen(), [lookIfOpen]);
  return <Page known={known} />;
}

describe("the agents' page", () => {
  const able = () => [
    wiredOne("claude-code", { name: "Claude Code", wired: true, hooks: noHooks }),
    wiredOne("codex", { name: "Codex", wired: true }),
  ];

  it("is quiet with nothing to fix or add, and with the door closed", () => {
    const { rerender } = render(
      <Page known={knownFor({ agent: open, agents: [wiredOne("codex", { wired: true })] })} />,
    );
    expect(screen.queryByRole("region")).toBeNull();
    rerender(<Page known={knownFor({ agent: { on: false }, agents: [lostOne("codex")] })} />);
    expect(screen.queryByRole("region")).toBeNull();
  });

  it("shows every file and folder it would write before its one button, and says what each got", async () => {
    const look = vi.fn(() => Promise.resolve());
    ipc.answer = (cmd, args) => {
      if (cmd === "agents_tisty_round") return Promise.resolve(skillsFor({ on: true }));
      if (args.id === "codex")
        return Promise.resolve({
          seen: [lostOne("codex")],
          connection: { code: "settingsPuzzling", name: "/home/someone/config.toml" },
        });
      return Promise.resolve(fixed(String(args.id)));
    };
    render(
      <Page
        known={knownFor({
          agent: open,
          agents: [
            lostOne("codex"),
            lostOne("claude-code", { hooks: strayHooks() }),
            wiredOne("cursor", { wired: true, hooks: strayHooks() }),
            lostOne("zed", { gone: false }),
          ],
          skills: skillsHere({ on: true }),
          lookAgents: look,
        })}
      />,
    );

    const todo = within(screen.getByRole("region", { name: "3 agents to fix" }));
    for (const at of [
      "/home/someone/codex.json",
      "/home/someone/claude-code.json",
      "/home/someone/.claude/skills",
      "/home/someone/.agents/skills",
    ])
      expect(todo.getByText(at)).toBeTruthy();
    expect(todo.getAllByText(HOOKS_AT)).toHaveLength(2);
    expect(todo.getByText(/\.before-tisty-last/)).toBeTruthy();
    expect(todo.queryByText(/could not/)).toBeNull();
    expect(ipc.calls).toHaveLength(0);

    await userEvent.click(todo.getByRole("button", { name: "Fix all" }));

    const told = within(await screen.findByRole("status"));
    expect(told.getByText("Some could not be fixed")).toBeTruthy();
    expect(ids("mend")).toEqual(["codex", "claude-code", "cursor"]);
    expect(told.getByText("codex").closest("li")?.textContent).toContain("config.toml");
    expect(told.getByText("claude-code").closest("li")?.textContent).toContain(
      "connection and hooks",
    );
    expect(told.getByText("Tisty's skills").closest("li")?.textContent).toContain("up to date");
    expect(told.getByText("Quit each one completely and open it again.")).toBeTruthy();
    expect(look).toHaveBeenCalled();
    await userEvent.click(told.getByRole("button", { name: "Got it" }));
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("says before and after that hooks the plugin brings are taken out", () => {
    const hooks = { ...strayHooks(), plugged: true };
    const agents = [wiredOne("claude-code", { wired: true, hooks })];
    render(<Page known={knownFor({ agent: open, agents })} />);
    expect(screen.getByText(t("mendHooksGo"))).toBeTruthy();
  });

  it("finds the plugin out by its first fix and takes the hooks out with the next one, said before", async () => {
    const stray = wiredOne("claude-code", { wired: true, hooks: strayHooks() });
    ipc.answer = (cmd, args) => {
      if (cmd === "wiring_watched") return Promise.resolve({ agent: open, seen: [stray] });
      if (cmd === "wiring") return Promise.resolve([stray]);
      if (cmd === "mend")
        return (args.ask as { hooksOut: boolean }).hooksOut
          ? Promise.resolve(fixed("claude-code"))
          : Promise.resolve({ seen: [stray], hooks: { code: "hooksByPlugin" } });
      return Promise.resolve(null);
    };
    render(<Window />);
    const todo = await screen.findByRole("region", { name: "1 agent to fix" });
    expect(within(todo).getByText(/installed_plugins.json/)).toBeTruthy();
    await userEvent.click(within(todo).getByRole("button", { name: "Fix" }));

    expect(await screen.findByText(t("mendHooksGo"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Fix" }));
    await waitFor(() => expect(sent("mend")).toHaveLength(2));
    expect((sent("mend")[1].args.ask as { hooksOut: boolean }).hooksOut).toBe(true);
    expect(await screen.findByText(/hooks removed, the plugin brings them/)).toBeTruthy();
  });

  it("says before taking hooks out that it reads the plugin's records, and forgets a plugin no longer there", async () => {
    const stray = wiredOne("claude-code", { wired: true, hooks: strayHooks() });
    ipc.answer = (cmd, args) => {
      if (cmd === "wiring_watched") return Promise.resolve({ agent: open, seen: [stray] });
      if (cmd === "wiring") return Promise.resolve([stray]);
      const out = (args.ask as { hooksOut?: boolean } | undefined)?.hooksOut;
      const code = out ? "hooksNotByPlugin" : "hooksByPlugin";
      return Promise.resolve(cmd === "mend" ? { seen: [stray], hooks: { code } } : null);
    };
    render(<Window />);
    await userEvent.click(await screen.findByRole("button", { name: "Fix" }));
    expect((await screen.findByText(t("mendHooksGo"))).textContent).toContain("settings.json");
    await userEvent.click(screen.getByRole("button", { name: "Fix" }));
    await waitFor(() => expect(sent("mend")).toHaveLength(2));
    await waitFor(() => expect(screen.queryByText(t("mendHooksGo"))).toBeNull());
    await userEvent.click(screen.getByRole("button", { name: "Fix" }));
    await waitFor(() => expect(sent("mend")).toHaveLength(3));
    expect((sent("mend")[2].args.ask as { hooksOut: boolean }).hooksOut).toBe(false);
  });

  it("asks to quit only an agent whose own files were written", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_tisty_round") return Promise.resolve(skillsFor({ on: true }));
      if (cmd === "hook") return Promise.reject({ code: "hooksByPlugin" });
      return Promise.resolve([]);
    };
    const { rerender } = render(
      <Page known={knownFor({ agent: open, agents: [], skills: skillsHere({ on: true }) })} />,
    );
    await userEvent.click(screen.getByRole("button", { name: "Fix" }));
    expect((await screen.findByRole("status")).textContent).toContain("up to date");
    expect(screen.queryByText(t("mendRestart"))).toBeNull();

    rerender(<Page known={knownFor({ agent: open, agents: able() })} />);
    await userEvent.click(screen.getByRole("button", { name: "Add hooks" }));
    await userEvent.click(screen.getByRole("button", { name: "Add hooks" }));
    expect(await screen.findByText(/the Tisty plugin already brings its hooks/)).toBeTruthy();
    expect(screen.queryByText(t("mendRestart"))).toBeNull();
  });

  it("never offers again hooks the plugin turned out to bring", async () => {
    ipc.answer = (cmd) =>
      cmd === "hook" ? Promise.reject({ code: "hooksByPlugin" }) : Promise.resolve([]);
    render(<Page known={knownFor({ agent: open, agents: able() })} />);
    await userEvent.click(screen.getByRole("button", { name: "Add hooks" }));
    await userEvent.click(screen.getByRole("button", { name: "Add hooks" }));
    expect(await screen.findByText(/the Tisty plugin already brings its hooks/)).toBeTruthy();
    expect(offered()).toEqual(["claude-code#hooks"]);
  });

  it("keeps one that leads to another Tisty apart, with its own «Not now», and fixes none of them", async () => {
    const agents = [
      lostOne("zed", { gone: false, points: "/other/tisty" }),
      wiredOne("claude-code", { wired: true, hooks: noHooks }),
    ];
    render(<Page known={knownFor({ agent: open, agents, skills: skillsHere() })} />);

    const apart = within(screen.getByRole("region", { name: "zed points to another Tisty" }));
    expect(apart.getByText(/\/other\/tisty, which is not this Tisty/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^Fix/ })).toBeNull();
    expect(screen.getByRole("region", { name: t("extrasTitle") })).toBeTruthy();
    await userEvent.click(apart.getByRole("button", { name: "Not now" }));
    expect(screen.queryByRole("region", { name: /another Tisty/ })).toBeNull();
  });

  it.each([
    [() => Promise.resolve(skillsFor({ on: true })), "Fixed"],
    [() => Promise.resolve(skillsHere({ on: true })), "Some could not be fixed"],
    [() => Promise.reject({ code: "cannotWrite", name: "disk" }), "Some could not be fixed"],
  ])(
    "brings the skills up to date on their own, without saying they failed before trying",
    async (round, said) => {
      ipc.answer = (cmd) => (cmd === "agents_tisty_round" ? round() : Promise.resolve(null));
      render(<Page known={knownFor({ agent: open, skills: skillsHere({ on: true }) })} />);
      const todo = within(
        screen.getByRole("region", { name: "Tisty's skills to bring up to date" }),
      );
      expect(todo.queryByText(/could not|before-tisty/)).toBeNull();
      await userEvent.click(todo.getByRole("button", { name: "Fix" }));
      expect(await screen.findByText(said)).toBeTruthy();
    },
  );

  const ask = async (known: Partial<KnownAgents> = {}) => {
    render(
      <Page known={knownFor({ agent: open, agents: able(), skills: skillsHere(), ...known })} />,
    );
    const offer = within(screen.getByRole("region", { name: t("extrasTitle") }));
    await userEvent.click(offer.getByRole("button", { name: "Add hooks and turn on skills" }));
    return offer;
  };

  it("names the agents the skills reach, and every file and folder, tied to its own tick", async () => {
    const offer = await ask();
    const skills = offer.getByRole("checkbox", {
      name: "Tisty's skills on this whole computer: Claude Code, Codex · Copilot · Gemini",
    });
    const about = document.getElementById(skills.getAttribute("aria-describedby") ?? "");
    expect(about?.textContent).toContain("/home/someone/.agents/skills");
    const hooks = offer.getByRole("checkbox", { name: "Hooks for Claude Code" });
    const where = document.getElementById(hooks.getAttribute("aria-describedby") ?? "");
    expect(where?.textContent).toContain(HOOKS_AT);
    expect(where?.textContent).toContain("installed_plugins.json");
    expect(document.activeElement).toBe(hooks);
    expect(ipc.calls).toHaveLength(0);
  });

  it("writes only what stays ticked, and offers again neither it nor what was unticked", async () => {
    ipc.answer = () => Promise.resolve([]);
    const offer = await ask();
    await userEvent.click(offer.getByRole("checkbox", { name: /^Tisty's skills/ }));
    await userEvent.click(offer.getByRole("button", { name: "Add hooks" }));

    expect(await screen.findByText("Done")).toBeTruthy();
    expect(ids("hook")).toEqual(["claude-code"]);
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
    expect(offered()).toEqual(["claude-code#hooks", SKILLS_DECLINED]);
  });

  it("turns the skills on first, so hooks the plugin turns out to bring are never added", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_tisty_skills_take")
        return Promise.resolve(skillsFor({ on: true, plugged: ["claude"] }));
      if (cmd === "wiring")
        return Promise.resolve([
          wiredOne("claude-code", { wired: true, hooks: { ...noHooks, plugged: true } }),
        ]);
      return Promise.resolve([]);
    };
    const offer = await ask();
    await userEvent.click(offer.getByRole("button", { name: "Add hooks and turn on skills" }));

    expect(await screen.findByText(/the Tisty plugin already brings its hooks/)).toBeTruthy();
    expect(sent("hook")).toHaveLength(0);
    expect(screen.getByText("Tisty's skills").closest("li")?.textContent).toContain("on");
  });

  it("says what could not be done and offers that again", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "hook") return Promise.reject({ code: "cannotWrite", name: "disk" });
      if (cmd === "wiring") return Promise.reject({ code: "x" });
      return Promise.resolve(
        skillsFor({ on: true, landing: [{ ...landing[0], standing: "foreign" }] }),
      );
    };
    const offer = await ask();
    await userEvent.click(offer.getByRole("button", { name: "Add hooks and turn on skills" }));

    expect(await screen.findByText("Some could not be done")).toBeTruthy();
    expect(screen.getByText(/A skill that is not Tisty's/)).toBeTruthy();
    expect(offered()).toEqual([]);
  });

  it("does not offer for ever hooks whose plugin's records cannot be read", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "hook") return Promise.reject({ code: "hooksRecordUnread", name: "/s.json" });
      if (cmd === "wiring") return Promise.reject({ code: "x" });
      return Promise.resolve(skillsFor({ on: true }));
    };
    const offer = await ask();
    await userEvent.click(offer.getByRole("button", { name: "Add hooks and turn on skills" }));

    expect(await screen.findByText(/\/s\.json could not be read/)).toBeTruthy();
    expect(offered()).toEqual(["claude-code#hooks"]);
  });

  it("says why when the skills could not be turned on", async () => {
    ipc.answer = () => Promise.reject({ code: "cannotWrite", name: "disk" });
    const agents = [wiredOne("codex", { wired: true })];
    render(<Page known={knownFor({ agent: open, agents, skills: skillsHere() })} />);
    await userEvent.click(screen.getByRole("button", { name: "Turn on skills" }));
    await userEvent.click(screen.getByRole("button", { name: "Turn on skills" }));
    expect(await screen.findByText("Some could not be done")).toBeTruthy();
  });

  it.each([
    [[wiredOne("codex", { wired: true })], skillsHere(), "Turn on skills"],
    [able(), null, "Add hooks"],
  ])("names on its button only what there is", async (agents, skills, said) => {
    render(<Page known={knownFor({ agent: open, agents, skills })} />);
    expect(screen.getByRole("button", { name: said })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByRole("region")).toBeNull();
    expect(offered().length).toBeGreaterThan(0);
  });

  it("takes the keyboard there when it was reached from a notice, and not with nothing to show", () => {
    const { unmount } = render(
      <Page known={knownFor({ agent: open, agents: [lostOne("codex")] })} sentAt={1} />,
    );
    expect(document.activeElement?.textContent).toContain("1 agent to fix");
    unmount();
    render(<Page known={knownFor({ agent: open })} sentAt={2} />);
    expect(document.activeElement).toBe(document.body);
  });
});

function Notice({
  all,
  onOpen,
  agent = open,
}: Readonly<{ all: Wired[]; onOpen: () => void; agent?: { on: boolean } }>) {
  return <AgentsAmiss amiss={useUnheeded(knownFor({ agent, agents: all }))} onOpen={onOpen} />;
}

describe("the notice in the window", () => {
  it("says who was lost and leads there", async () => {
    const onOpen = vi.fn();
    render(<Notice all={[lostOne("Codex")]} onOpen={onOpen} />);
    const said = within(screen.getByRole("status"));
    expect(said.getByText("Codex can no longer find Tisty")).toBeTruthy();
    expect(said.getByText("The Tisty they point to is no longer on this computer.")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "See and fix" }));
    expect(onOpen).toHaveBeenCalled();
  });

  it("goes on «Not now», and says nothing with the door shut nor of one that leads to another Tisty", async () => {
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
  it("never offers Claude Code's hooks while nobody asked whether the plugin brings them", async () => {
    const unknown = wiredOne("claude-code", {
      wired: true,
      hooks: { on: false, at: "/home/someone/settings.json", pluginUnknown: true },
    });
    ipc.answer = (cmd) =>
      Promise.resolve(cmd === "wiring_watched" ? { agent: open, seen: [unknown] } : []);
    const { result } = renderHook(() => useKnownAgents());
    await act(() => result.current.lookIfOpen());
    const asked = {
      agent: result.current.agent,
      agents: result.current.agents,
      skills: null,
      declined: [],
    };
    expect(extrasOf(asked)).toBeNull();
  });

  it("keeps what the page found out of the plugin when the window looks again", async () => {
    const hooks = { on: false, at: "/home/someone/settings.json" };
    ipc.answer = (cmd) => {
      if (cmd === "wiring")
        return Promise.resolve([wiredOne("claude-code", { hooks: { ...hooks, plugged: true } })]);
      if (cmd === "wiring_watched")
        return Promise.resolve({
          agent: open,
          seen: [wiredOne("claude-code", { hooks: { ...hooks, pluginUnknown: true } })],
        });
      return Promise.resolve([]);
    };
    const { result } = renderHook(() => useKnownAgents());
    await act(() => result.current.lookAgents());
    await act(() => result.current.lookIfOpen());
    expect(result.current.agents?.[0].hooks).toMatchObject({ plugged: true, pluginUnknown: false });
  });

  it("is told only with the door open, and reads the skills once their round is done", async () => {
    let watched: unknown = { agent: { on: false }, seen: null };
    ipc.answer = (cmd) => {
      if (cmd === "wiring_watched") return Promise.resolve(watched);
      if (cmd === "agents_tisty_skills") return Promise.resolve(skillsFor({ on: true }));
      return Promise.resolve([]);
    };
    const { result } = renderHook(() => useKnownAgents());
    await act(() => result.current.lookIfOpen());
    expect(result.current.agent).toEqual({ on: false });
    expect(result.current.agents).toBeNull();

    watched = { agent: { on: true }, seen: [lostOne("codex")] };
    await act(() => result.current.lookIfOpen());
    expect(result.current.agents).toHaveLength(1);
    expect(sent("wiring")).toHaveLength(0);
    expect(sent("agent")).toHaveLength(0);
    expect(sent("agents_tisty_skills")).toHaveLength(0);
    watched = null;
    await act(() => result.current.lookIfOpen());
    expect(result.current.agents).toHaveLength(1);

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
      skills: skillsHere(),
    });
    const { result } = renderHook(() => useExtras(known));
    expect(result.current.extras?.skills).toHaveLength(1);
    act(() => {
      result.current.decline();
    });
    expect(result.current.extras).toBeNull();
  });
});
