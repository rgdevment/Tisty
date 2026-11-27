import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentGroups } from "../agentGroups";
import type { Filed, Wired } from "../core";
import { adopt } from "../locales";
import AgentGroupRows from "../ui/AgentGroupRows";
import TistyHooksRow from "../ui/HooksRow";
import type { AgentHook, HookLook, Tried } from "../userHooks";

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

const GROUPS: AgentGroups = {
  rules: [],
  ruleTargets: [],
  skills: [],
  hooks: [],
  left: [],
  emptied: new Set(),
};

const inHere = (agentId: string, name: string, more: Partial<AgentHook> = {}): AgentHook => ({
  agent: agentId,
  name,
  at: `/home/${agentId}/hooks.json`,
  takes: true,
  installed: true,
  present: false,
  here: false,
  apart: false,
  unread: false,
  trustedByHand: false,
  stuck: null,
  ...more,
});

const look = (more: Partial<HookLook> = {}): HookLook => ({
  file: "guard",
  name: "guard.sh",
  script: "echo guarded\nexit 2\n",
  event: "before-tool",
  tools: "shell",
  timeout: 10,
  agents: ["claude-code", "codex"],
  print: "print-1",
  state: "waiting",
  door: true,
  kept: false,
  here: [inHere("claude-code", "Claude Code"), inHere("codex", "Codex")],
  ...more,
});

const rows = (groups: Partial<AgentGroups>, onNewHook?: () => void) =>
  render(
    <ul>
      <AgentGroupRows
        groups={{ ...GROUPS, ...groups }}
        depth={0}
        step={12}
        paper={(one) => <li key={one.id}>{one.title}</li>}
        skill={() => null}
        rowOf={() => ({})}
        tistySkills={false}
        agents={[]}
        onNewHook={onNewHook}
      />
    </ul>,
  );

beforeEach(() => {
  adopt("en");
  ipc.calls.length = 0;
  ipc.answer = () => Promise.resolve(null);
});

describe("Global's Hooks group", () => {
  it("draws the heading with its plus and the person's hooks with Register and Verify", async () => {
    ipc.answer = () => Promise.resolve(look());
    const onNewHook = vi.fn();
    rows({ hooks: [doc("guard")] }, onNewHook);
    expect(screen.getByText("Hooks")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "New hook" }));
    expect(onNewHook).toHaveBeenCalledOnce();
    expect(screen.getByText("guard")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("Not registered here")).toBeTruthy());
    expect(sent("agent_hook_look")[0].args).toEqual({ file: "guard" });
    expect((screen.getByRole("button", { name: "Register" }) as HTMLButtonElement).disabled).toBe(
      false,
    );
    expect(screen.getByRole("button", { name: "Verify" })).toBeTruthy();
  });

  it("draws no Hooks heading when there is no plus, no hook and no Tisty hooks", () => {
    rows({});
    expect(screen.queryByText("Hooks")).toBeNull();
  });

  it("keeps Register off for a hook that is for another computer", async () => {
    ipc.answer = () => Promise.resolve(look({ state: "elsewhere" }));
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("For another computer")).toBeTruthy());
    expect((screen.getByRole("button", { name: "Register" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("shows the whole script and the files before the yes and registers the print it looked at", async () => {
    ipc.answer = (cmd) =>
      Promise.resolve(cmd === "agent_hook_register" ? look({ state: "registered" }) : look());
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Not registered here")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Register" }));
    await waitFor(() =>
      expect(screen.getByText("Register guard.sh on this computer")).toBeTruthy(),
    );
    expect(screen.getByText(/echo guarded/)).toBeTruthy();
    expect(screen.getByText("It writes in:")).toBeTruthy();
    expect(screen.getByText("Claude Code · /home/claude-code/hooks.json")).toBeTruthy();
    expect(screen.getByText("Codex · /home/codex/hooks.json")).toBeTruthy();
    expect(sent("agent_hook_register")).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: "Yes, register" }));
    await waitFor(() => expect(sent("agent_hook_register")).toHaveLength(1));
    expect(sent("agent_hook_register")[0].args).toEqual({ file: "guard", print: "print-1" });
    await waitFor(() => expect(screen.getByText("Registered")).toBeTruthy());
  });

  it("lists in the register ask only the agents that take the hook and are installed", async () => {
    ipc.answer = () =>
      Promise.resolve(
        look({
          here: [
            inHere("claude-code", "Claude Code"),
            inHere("codex", "Codex", { takes: false }),
            inHere("cursor", "Cursor", { installed: false }),
          ],
        }),
      );
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Not registered here")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Register" }));
    await waitFor(() => expect(screen.getByText("It writes in:")).toBeTruthy());
    expect(screen.getByText("Claude Code · /home/claude-code/hooks.json")).toBeTruthy();
    expect(screen.queryByText(/Codex ·/)).toBeNull();
    expect(screen.queryByText(/Cursor ·/)).toBeNull();
  });

  it("says in verify what each agent holds and the Codex approval only where it is present", async () => {
    ipc.answer = () =>
      Promise.resolve(
        look({
          state: "registered",
          kept: true,
          here: [
            inHere("claude-code", "Claude Code", { present: true, here: true }),
            inHere("codex", "Codex", { present: true, here: true, trustedByHand: true }),
            inHere("cursor", "Cursor", { installed: false }),
            inHere("copilot", "Copilot", { takes: false }),
            inHere("gemini-cli", "Gemini", { present: true, here: false }),
            inHere("antigravity", "Antigravity", { apart: true }),
          ],
        }),
      );
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Registered")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    await waitFor(() => expect(screen.getAllByText(/Registered, runs this Tisty/)).toHaveLength(2));
    expect(screen.getAllByText("Approve it in Codex with /hooks")).toHaveLength(1);
    expect(screen.getByText(/Not installed here/)).toBeTruthy();
    expect(screen.getByText(/Has no such moment/)).toBeTruthy();
    expect(screen.getByText(/Runs another Tisty/)).toBeTruthy();
  });

  it("asks before trying it once and shows what it said and how it ended", async () => {
    const tried: Tried = {
      code: 2,
      said: "no rm here",
      timedOut: false,
      tookMs: 41,
      missing: null,
    };
    ipc.answer = (cmd) =>
      Promise.resolve(cmd === "agent_hook_try" ? tried : look({ state: "registered", kept: true }));
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Registered")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "Try it once" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Try it once" }));
    expect(sent("agent_hook_try")).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: "Run it" }));
    await waitFor(() =>
      expect(screen.getByText(/It ended with 2: it stops the call/)).toBeTruthy(),
    );
    expect(sent("agent_hook_try")[0].args).toEqual({ file: "guard", print: "print-1" });
    expect(screen.getByText(/41 ms/)).toBeTruthy();
    expect(screen.getByText("no rm here")).toBeTruthy();
  });

  it("shows the script again before trying a hook that is not registered yet", async () => {
    ipc.answer = () => Promise.resolve(look());
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Not registered here")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "Try it once" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Try it once" }));
    expect(screen.getByText(/echo guarded/)).toBeTruthy();
  });

  it("says a missing interpreter and a time out in words", async () => {
    let answer: Tried = { code: null, said: "", timedOut: false, tookMs: 0, missing: "ps1" };
    ipc.answer = (cmd) =>
      Promise.resolve(
        cmd === "agent_hook_try" ? answer : look({ state: "registered", kept: true }),
      );
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Registered")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    await waitFor(() => screen.getByRole("button", { name: "Try it once" }));
    fireEvent.click(screen.getByRole("button", { name: "Try it once" }));
    fireEvent.click(screen.getByRole("button", { name: "Run it" }));
    await waitFor(() => expect(screen.getByText(/ps1/)).toBeTruthy());
    answer = { code: null, said: "", timedOut: true, tookMs: 10000, missing: null };
    fireEvent.click(screen.getByRole("button", { name: "Try it once" }));
    fireEvent.click(screen.getByRole("button", { name: "Run it" }));
    await waitFor(() => expect(screen.getByText(/10000 ms/)).toBeTruthy());
  });

  it("takes a registered hook out of the agents from verify, once confirmed", async () => {
    ipc.answer = (cmd) =>
      Promise.resolve(
        cmd === "agent_hook_unregister" ? look() : look({ state: "registered", kept: true }),
      );
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Registered")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    await waitFor(() => screen.getByRole("button", { name: "Take it out of the agents" }));
    fireEvent.click(screen.getByRole("button", { name: "Take it out of the agents" }));
    expect(sent("agent_hook_unregister")).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: "Yes, take it out" }));
    await waitFor(() => expect(sent("agent_hook_unregister")).toHaveLength(1));
    expect(sent("agent_hook_unregister")[0].args).toEqual({ file: "guard" });
  });

  it("warns in verify when the door is shut or the kept script is gone", async () => {
    ipc.answer = () => Promise.resolve(look({ state: "registered", door: false, kept: false }));
    rows({ hooks: [doc("guard")] });
    await waitFor(() => expect(screen.getByText("Registered")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    await waitFor(() => expect(screen.getByText(/The agents' door is closed/)).toBeTruthy());
    expect(screen.getByText("Its script is no longer here: register it again")).toBeTruthy();
  });
});

describe("the row of Tisty's own hooks", () => {
  const tisty = (agents: Wired[]) =>
    render(
      <ul>
        <TistyHooksRow agents={agents} depth={0} step={12} onOpen={vi.fn()} />
      </ul>,
    );

  it("is one row whatever the agents are, with no Codex approval line", () => {
    tisty([agent("claude", "Claude"), agent("codex", "Codex")]);
    expect(screen.getAllByRole("button")).toHaveLength(1);
    expect(screen.queryByText("Approve it in Codex with /hooks")).toBeNull();
  });

  it("sits apart from the person's hooks, in the From Tisty group", () => {
    render(
      <ul>
        <AgentGroupRows
          groups={{ ...GROUPS, hooks: [doc("guard")] }}
          depth={0}
          step={12}
          paper={(one) => <li key={one.id}>{one.title}</li>}
          skill={() => null}
          rowOf={() => ({})}
          tistySkills={false}
          agents={[agent("claude", "Claude")]}
          onHooks={vi.fn()}
        />
      </ul>,
    );
    const person = screen.getByText("guard");
    const from = screen.getByText("From Tisty");
    const row = screen.getByRole("button", { name: /^Tisty's hooks:/ });
    const follows = (a: Node, b: Node) =>
      Boolean(a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING);
    expect(follows(person, from)).toBe(true);
    expect(follows(from, row)).toBe(true);
  });
});
