import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  AGENTS_ASLEEP,
  type AgentsLook,
  type AgentsState,
  agentsLit,
  askToLight,
  lightAgents,
  onlySwitchedOff,
} from "../agentsSync";
import type { Folded, Papers } from "../core";
import { AGENTS, hide, isHidden, showAll } from "../hiding";
import { adopt } from "../locales";
import AgentsAsleep, { foundSaid } from "../ui/AgentsAsleep";
import AgentsKeeper from "../ui/AgentsKeeper";
import SideRail from "../ui/SideRail";
import Tree from "../ui/Tree";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  state: null as unknown,
  look: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "agents_state") return Promise.resolve(ipc.state);
    if (cmd === "agents_look") return Promise.resolve(ipc.look);
    if (cmd === "agents_rows") return Promise.resolve([]);
    if (cmd === "agents_turn") {
      const now = ipc.state as AgentsState;
      ipc.state = { ...now, allOn: args?.on as boolean };
    }
    return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
  },
}));

const desk = vi.hoisted(() => ({
  papers: { folders: [], docs: [] } as Papers,
  here: null as string | null,
  setHere: (_at: string | null) => {},
  papersChanged: () => {},
  setError: (_said: string | null) => {},
}));
vi.mock("../desk", () => ({ useDesk: () => desk }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);
const settle = () => act(() => new Promise((ready) => setTimeout(ready, 0)));

const folder = (id: string, name: string, more: Partial<Folded> = {}): Folded => ({
  id,
  name,
  parent: null,
  icon: null,
  holds: 1,
  archived: false,
  away: false,
  ...more,
});

const state = (more: Partial<AgentsState> = {}): AgentsState => ({
  folder: null,
  consented: false,
  allOn: true,
  assistants: [],
  ...more,
});

const look = (more: Partial<AgentsLook> = {}): AgentsLook => ({
  folder: "A",
  docs: 1,
  changes: 0,
  waiting: 0,
  unagreed: 0,
  proposed: 0,
  consented: true,
  allOn: true,
  asks: false,
  assistants: [],
  both: [],
  loose: [],
  unled: [],
  ...more,
});

const bare: Papers = { folders: [folder("W", "trabajo")], docs: [] };
const withAgents: Papers = {
  folders: [folder("A", "AGENTS", { icon: "robot", color: "purple" }), folder("W", "trabajo")],
  docs: [{ id: "D1", file: "f1", title: "Reglas", folder: "A", archived: false, away: false }],
};

const tree = (papers: Papers, more: Partial<React.ComponentProps<typeof Tree>> = {}) =>
  render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} {...more} />);

const rail = (papers: Papers) =>
  render(
    <SideRail
      lists={[]}
      papers={papers}
      counts={{}}
      chosen={{ named: "tasks" }}
      onChoose={vi.fn()}
      onReveal={vi.fn()}
    />,
  );

const listened = () => {
  const heard = vi.fn();
  window.addEventListener(AGENTS_ASLEEP, heard);
  return heard;
};

beforeEach(() => {
  adopt("en");
  showAll();
  ipc.calls = [];
  ipc.state = state();
  ipc.look = look();
  desk.papers = { folders: [], docs: [] };
});

afterEach(() => showAll());

describe("whether AGENTS is on", () => {
  it("is on only with its folder, this computer's yes and the switch on", () => {
    expect(agentsLit(state({ folder: "A", consented: true }))).toBe(true);
    expect(agentsLit(state({ consented: true }))).toBe(false);
    expect(agentsLit(state({ folder: "A" }))).toBe(false);
    expect(agentsLit(state({ folder: "A", consented: true, allOn: false }))).toBe(false);
  });

  it("only switches back on a folder this computer already allowed", () => {
    expect(onlySwitchedOff(state({ folder: "A", consented: true, allOn: false }))).toBe(true);
    expect(onlySwitchedOff(state({ folder: "A", allOn: false }))).toBe(false);
    expect(onlySwitchedOff(state({ consented: true, allOn: false }))).toBe(false);
    expect(onlySwitchedOff(state({ folder: "A", consented: true }))).toBe(false);
  });
});

describe("AGENTS in the sidebar while it has no folder", () => {
  it("stands there, off, and answers a click with the notice", async () => {
    const heard = listened();
    tree(bare);
    const row = screen.getByRole("button", { name: "AGENTS — off" });
    await userEvent.click(row);
    expect(heard).toHaveBeenCalledTimes(1);
    window.removeEventListener(AGENTS_ASLEEP, heard);
  });

  it("is reached with the arrows like any row", () => {
    tree(bare);
    const work = screen.getByRole("button", { name: "trabajo" });
    const agents = screen.getByRole("button", { name: "AGENTS — off" });
    act(() => work.focus());
    fireEvent.keyDown(work, { key: "ArrowDown" });
    expect(document.activeElement).toBe(agents);
    fireEvent.keyDown(agents, { key: "ArrowUp" });
    expect(document.activeElement).toBe(work);
    fireEvent.keyDown(agents, { key: "Enter" });
  });

  it("holds the tab stop when it is the only row before the loose documents", () => {
    tree({ folders: [], docs: [] });
    expect(screen.getByRole("button", { name: "AGENTS — off" }).tabIndex).toBe(0);
  });

  it("goes once hidden, and while the sidebar is sifted", () => {
    const { unmount } = tree(bare, { unfolded: true });
    expect(screen.queryByRole("button", { name: "AGENTS — off" })).toBeNull();
    unmount();
    hide(AGENTS);
    tree(bare);
    expect(screen.queryByRole("button", { name: "AGENTS — off" })).toBeNull();
  });

  it("asks nothing at all without a folder, so nothing of an agent's is looked at", async () => {
    tree(bare);
    await settle();
    expect(sent("agents_state")).toHaveLength(0);
    expect(sent("agents_look")).toHaveLength(0);
  });
});

describe("AGENTS in the sidebar with its folder", () => {
  it("is dim while switched off, opens the notice and still unfolds its documents", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    const heard = listened();
    const onHere = vi.fn();
    tree(withAgents, { onHere });
    const row = await screen.findByRole("button", { name: "AGENTS — off" });
    await userEvent.click(row);
    expect(heard).toHaveBeenCalledTimes(1);
    expect(onHere).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Open AGENTS" }));
    expect(screen.getByRole("button", { name: "Reglas" })).toBeTruthy();
    expect(sent("agents_look")).toHaveLength(0);
    window.removeEventListener(AGENTS_ASLEEP, heard);
  });

  it("is drawn plain, never off, while its state is still on its way", async () => {
    ipc.state = new Promise(() => {});
    tree(withAgents);
    rail(withAgents);
    await settle();
    expect(screen.getAllByRole("button", { name: "AGENTS" })).toHaveLength(2);
    expect(screen.queryByRole("button", { name: "AGENTS — off" })).toBeNull();
    expect(screen.queryByText("off")).toBeNull();
    expect(sent("agents_look")).toHaveLength(0);
  });

  it("is dim on a computer that never said yes to it", async () => {
    ipc.state = state({ folder: "A", consented: false });
    tree(withAgents);
    expect(await screen.findByRole("button", { name: "AGENTS — off" })).toBeTruthy();
  });

  it("opens like any folder once on, and says how it stands", async () => {
    ipc.state = state({ folder: "A", consented: true });
    ipc.look = look({ changes: 2 });
    const onHere = vi.fn();
    tree(withAgents, { onHere });
    expect(await screen.findByText("2 changed")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "AGENTS" }));
    expect(onHere).toHaveBeenCalledWith("A");
  });

  it("goes when hidden by its own key or by its folder's", async () => {
    ipc.state = state({ folder: "A", consented: true });
    hide("A");
    const { unmount } = tree(withAgents);
    await settle();
    expect(screen.queryByRole("button", { name: /AGENTS/ })).toBeNull();
    unmount();
    showAll();
    hide(AGENTS);
    tree(withAgents);
    await settle();
    expect(screen.queryByRole("button", { name: /AGENTS/ })).toBeNull();
    expect(screen.getByRole("button", { name: "trabajo" })).toBeTruthy();
  });
});

describe("AGENTS in the folded sidebar", () => {
  it("stands there off without a folder, and answers with the notice", async () => {
    const heard = listened();
    rail(bare);
    await userEvent.click(screen.getByRole("button", { name: "AGENTS — off" }));
    expect(heard).toHaveBeenCalledTimes(1);
    window.removeEventListener(AGENTS_ASLEEP, heard);
  });

  it("dims its folder while off and opens it once on", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    const { unmount } = rail(withAgents);
    expect(await screen.findByRole("button", { name: "AGENTS — off" })).toBeTruthy();
    unmount();
    ipc.state = state({ folder: "A", consented: true });
    rail(withAgents);
    await settle();
    expect(screen.getByRole("button", { name: "AGENTS" })).toBeTruthy();
  });

  it("leaves AGENTS out once hidden", async () => {
    hide(AGENTS);
    rail(bare);
    await settle();
    expect(screen.queryByRole("button", { name: /AGENTS/ })).toBeNull();
  });
});

describe("the notice an AGENTS that is off answers with", () => {
  it("names the assistants found here, one, two or more", () => {
    const named = (...names: string[]) =>
      foundSaid(state({ assistants: names.map((name) => ({ id: name, name })) }));
    expect(named()).toBe("I found no assistant on this computer.");
    expect(named("Claude")).toBe("I found Claude on this computer.");
    expect(named("Claude", "Codex")).toBe("I found Claude and Codex on this computer.");
    expect(named("Claude", "Codex", "Gemini")).toBe(
      "I found Claude, Codex and Gemini on this computer.",
    );
    adopt("es");
    expect(named("Claude", "Codex")).toBe("Encontré Claude y Codex en este equipo.");
    expect(named()).toBe("No encontré ningún asistente en este equipo.");
  });

  it("offers turn on, not now and hide", async () => {
    const onLight = vi.fn();
    const onLater = vi.fn();
    const onHide = vi.fn();
    render(
      <AgentsAsleep
        state={state()}
        busy={false}
        trouble="It failed"
        onLight={onLight}
        onLater={onLater}
        onHide={onHide}
      />,
    );
    expect(screen.getByText("It failed")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Turn on" }));
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    await userEvent.click(screen.getByRole("button", { name: "Hide" }));
    expect([onLight, onLater, onHide].map((one) => one.mock.calls.length)).toEqual([1, 1, 1]);
  });
});

describe("what the keeper does with the notice", () => {
  const assistants = [
    { id: "claude", name: "Claude" },
    { id: "codex", name: "Codex" },
  ];
  const opened = async () => {
    render(<AgentsKeeper />);
    act(() => askToLight());
    return within(await screen.findByRole("dialog", { name: "AGENTS" }));
  };

  it("says what it found and goes with Not now, leaving AGENTS off", async () => {
    ipc.state = state({ assistants });
    const notice = await opened();
    expect(notice.getByText("I found Claude and Codex on this computer.")).toBeTruthy();
    await userEvent.click(notice.getByRole("button", { name: "Not now" }));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(sent("agents_turn")).toHaveLength(0);
    expect(sent("agents_look")).toHaveLength(0);
  });

  it("hides AGENTS with Hide", async () => {
    const notice = await opened();
    await userEvent.click(notice.getByRole("button", { name: "Hide" }));
    expect(isHidden(AGENTS)).toBe(true);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("only switches sync back on for a folder this computer allowed", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    const notice = await opened();
    await userEvent.click(notice.getByRole("button", { name: "Turn on" }));
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    expect(sent("agents_turn")[0].args).toEqual({ assistant: null, on: true });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("goes through the first time when there is no folder, and turns the switch on after", async () => {
    ipc.state = state({ consented: true, allOn: false });
    ipc.look = look({ folder: null, consented: false, asks: true });
    const notice = await opened();
    await userEvent.click(notice.getByRole("button", { name: "Turn on" }));
    const first = await screen.findByRole("dialog", { name: "Your agents, in one place" });
    expect(sent("agents_turn")).toHaveLength(0);
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    await userEvent.click(within(first).getByRole("button", { name: "Not now" }));
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    expect(sent("agents_turn")[0].args).toEqual({ assistant: null, on: true });
  });

  it("leaves the switch alone when the first time closes without a yes", async () => {
    ipc.look = look({ folder: null, consented: false, asks: true });
    const notice = await opened();
    await userEvent.click(notice.getByRole("button", { name: "Turn on" }));
    const first = await screen.findByRole("dialog", { name: "Your agents, in one place" });
    await userEvent.click(within(first).getByRole("button", { name: "Not now" }));
    await settle();
    expect(sent("agents_turn")).toHaveLength(0);
  });

  it("shows AGENTS again when turning it on, since on means seen", async () => {
    hide(AGENTS);
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    const notice = await opened();
    await userEvent.click(notice.getByRole("button", { name: "Turn on" }));
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    expect(isHidden(AGENTS)).toBe(false);
  });

  it("shows again an AGENTS hidden while on before hiding needed it off, and leaves it on", async () => {
    desk.papers = withAgents;
    hide("A");
    ipc.state = state({ folder: "A", consented: true });
    render(<AgentsKeeper />);
    await waitFor(() => expect(isHidden("A")).toBe(false));
    expect(sent("agents_turn")).toHaveLength(0);
  });

  it("leaves an AGENTS hidden while off where it is", async () => {
    desk.papers = withAgents;
    hide(AGENTS);
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    render(<AgentsKeeper />);
    await settle();
    expect(isHidden(AGENTS)).toBe(true);
  });

  it("keeps AGENTS hidden when the first time closes without a yes", async () => {
    hide(AGENTS);
    ipc.look = look({ folder: null, consented: false, asks: true });
    render(<AgentsKeeper />);
    act(() => lightAgents());
    const first = await screen.findByRole("dialog", { name: "Your agents, in one place" });
    await userEvent.click(within(first).getByRole("button", { name: "Not now" }));
    await settle();
    expect(isHidden(AGENTS)).toBe(true);
  });

  it("shows AGENTS once the first time leaves it on", async () => {
    hide(AGENTS);
    ipc.look = look({ folder: null, consented: false, asks: true });
    render(<AgentsKeeper />);
    act(() => lightAgents());
    const first = await screen.findByRole("dialog", { name: "Your agents, in one place" });
    ipc.state = state({ folder: "A", consented: true });
    await userEvent.click(within(first).getByRole("button", { name: "Not now" }));
    await waitFor(() => expect(isHidden(AGENTS)).toBe(false));
    expect(sent("agents_turn")).toHaveLength(0);
  });

  it("opens no notice for an AGENTS that turns out to be on", async () => {
    ipc.state = state({ folder: "A", consented: true });
    render(<AgentsKeeper />);
    act(() => askToLight());
    await settle();
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("turns on straight away from Settings", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    render(<AgentsKeeper />);
    act(() => lightAgents());
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});
