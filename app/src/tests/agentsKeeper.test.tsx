import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  type AgentsLook,
  type AgentsState,
  bringMoreAgents,
  type Rounded,
  syncAgents,
} from "../agentsSync";
import type { Filed, Folded, Papers } from "../core";
import { AGENTS, hide, isHidden, showAll } from "../hiding";
import { adopt } from "../locales";
import AgentsKeeper, { AGENTS_CHANGED, askAgents } from "../ui/AgentsKeeper";

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

const desk = vi.hoisted(() => ({
  papers: { folders: [], docs: [] } as Papers,
  here: null as string | null,
  setHere: (_at: string | null) => {},
  papersChanged: () => {},
  setError: (_said: string | null) => {},
}));

vi.mock("../desk", () => ({ useDesk: () => desk }));

const heard = vi.hoisted(() => ({ stirred: null as (() => void) | null }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, then: () => void) => {
    if (name === "stirred") heard.stirred = then;
    return Promise.resolve(() => undefined);
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const folder = (id: string, parent: string | null, more: Partial<Folded> = {}): Folded => ({
  id,
  name: id,
  parent,
  icon: null,
  holds: 0,
  archived: false,
  away: false,
  ...more,
});

const doc = (file: string, at: string, wrote = "1"): Filed =>
  ({ id: file, file, title: file, folder: at, archived: false, away: false, wrote }) as Filed;

const agents = [folder("a", null, { name: "AGENTS", icon: "robot" }), folder("claude", "a")];

const look = (more: Partial<AgentsLook> = {}): AgentsLook => ({
  folder: "a",
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

beforeEach(() => {
  adopt("es");
  ipc.calls = [];
  ipc.answer = (cmd) => {
    if (cmd === "agents_look") return Promise.resolve(look());
    return Promise.resolve({ told: [], both: [] });
  };
  desk.papers = { folders: [], docs: [] };
  desk.here = null;
  desk.setHere = vi.fn();
  desk.papersChanged = vi.fn();
  desk.setError = vi.fn();
});

describe("the keeper of AGENTS", () => {
  it("asks the first time someone enters a folder inside AGENTS", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve(look({ docs: 0, consented: false, asks: true }))
        : Promise.resolve(false);
    desk.papers = { folders: agents, docs: [] };
    desk.here = "claude";
    render(<AgentsKeeper />);
    await screen.findByText("Tus agentes, en un solo lugar");
  });

  it("does not ask when this computer said yes and AGENTS holds documents", async () => {
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    desk.here = "claude";
    render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_look")).toHaveLength(1));
    expect(screen.queryByText("Tus agentes, en un solo lugar")).toBeNull();
  });

  it("keeps an empty AGENTS, still rounds it, and asks at once about a file left behind", async () => {
    const loose = { at: "/h/.claude/rules/tone.md", assistant: "claude", changed: false };
    ipc.answer = (cmd) => {
      if (cmd === "agents_round")
        return Promise.resolve({ told: [], both: [], loose: [loose], unled: [] });
      if (cmd === "agents_look") return Promise.resolve(look({ docs: 0, loose: [loose] }));
      return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    };
    desk.papers = { folders: agents, docs: [] };
    render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_round")).toHaveLength(1), { timeout: 2000 });
    await screen.findByText("¿Quitar también el archivo?");
    expect(desk.setHere).not.toHaveBeenCalled();
    expect(sent("agents_empty")).toHaveLength(0);
    await userEvent.click(screen.getByRole("button", { name: "Quitar el archivo" }));
    await waitFor(() => expect(sent("agents_loose")).toHaveLength(1));
    expect(sent("agents_loose")[0].args).toEqual({ at: loose.at, away: true });
  });

  it("lets AGENTS go once its last document leaves while the window watches, and steps out", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "agents_empty");
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    desk.here = "claude";
    const { rerender } = render(<AgentsKeeper />);
    expect(sent("agents_empty")).toHaveLength(0);
    desk.papers = { folders: agents, docs: [] };
    rerender(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_empty")).toHaveLength(1));
    await waitFor(() => expect(desk.setHere).toHaveBeenCalledWith(null));
    expect(desk.papersChanged).toHaveBeenCalled();
  });

  it("asks about the last rule's file first, and lets AGENTS go once that is answered", async () => {
    const loose = { at: "/h/.claude/rules/tone.md", assistant: "claude", changed: false };
    let answered = false;
    ipc.answer = (cmd) => {
      if (cmd === "agents_round")
        return Promise.resolve({ told: [], both: [], loose: answered ? [] : [loose], unled: [] });
      if (cmd === "agents_look")
        return Promise.resolve(look({ docs: 0, asks: true, loose: answered ? [] : [loose] }));
      if (cmd === "agents_loose") {
        answered = true;
        return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
      }
      return Promise.resolve(cmd === "agents_empty");
    };
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    desk.here = "claude";
    const { rerender } = render(<AgentsKeeper />);
    desk.papers = { folders: agents, docs: [] };
    rerender(<AgentsKeeper />);
    await screen.findByText("¿Quitar también el archivo?");
    expect(sent("agents_empty")).toHaveLength(0);
    await userEvent.click(screen.getByRole("button", { name: "Quitar el archivo" }));
    await waitFor(() => expect(sent("agents_empty")).toHaveLength(1));
    expect(sent("agents_loose")[0].args).toEqual({ at: loose.at, away: true });
    await waitFor(() => expect(desk.setHere).toHaveBeenCalledWith(null));
  });

  it("keeps an AGENTS made empty the first time, however the folders around it change", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "agents_empty");
    desk.papers = { folders: [], docs: [] };
    const { rerender } = render(<AgentsKeeper />);
    desk.papers = { folders: [agents[0]], docs: [] };
    rerender(<AgentsKeeper />);
    desk.papers = { folders: agents, docs: [doc("x", "elsewhere")] };
    rerender(<AgentsKeeper />);
    await new Promise((ready) => setTimeout(ready, 50));
    expect(sent("agents_empty")).toHaveLength(0);
  });

  it("opens to bring another file when the AGENTS menu or Settings asks", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve(
            look({
              assistants: [
                {
                  id: "codex",
                  name: "Codex",
                  on: true,
                  files: [
                    {
                      at: "/h/.codex/AGENTS.override.md",
                      name: "AGENTS.override.md",
                      kind: "rule",
                      skip: null,
                      held: false,
                    },
                  ],
                },
              ],
            }),
          )
        : Promise.resolve(false);
    render(<AgentsKeeper />);
    act(() => bringMoreAgents());
    await screen.findByText("Traer otro archivo");
  });

  it("syncs once a change inside AGENTS settles, and says so when it fails", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_round" ? Promise.reject({ code: "internal" }) : Promise.resolve(false);
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_round")).toHaveLength(1), { timeout: 2000 });
    await waitFor(() => expect(desk.setError).toHaveBeenCalled());
  });

  it("rounds again when another computer's AGENTS arrives, so the two join", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "agents_round" ? { both: [] } : false);
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    const { rerender } = render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_round")).toHaveLength(1), { timeout: 2000 });

    const later = folder("b", null, { name: "AGENTS", icon: "robot" });
    desk.papers = { folders: [...agents, later], docs: [doc("x", "claude")] };
    rerender(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_round")).toHaveLength(2), { timeout: 2000 });
  });

  it("tells the rest of the window once a round is done", async () => {
    const heard = vi.fn();
    window.addEventListener(AGENTS_CHANGED, heard);
    ipc.answer = (cmd) => Promise.resolve(cmd === "agents_round" ? { both: [] } : false);
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    render(<AgentsKeeper />);
    await waitFor(() => expect(heard).toHaveBeenCalled(), { timeout: 2000 });
    window.removeEventListener(AGENTS_CHANGED, heard);
  });

  it("opens when Settings asks, and tells the rest of the window once it closes", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve(look({ folder: null, docs: 0, consented: false, asks: true }))
        : Promise.resolve(false);
    const heard = vi.fn();
    window.addEventListener(AGENTS_CHANGED, heard);
    render(<AgentsKeeper />);
    act(() => askAgents());
    await userEvent.click(await screen.findByRole("button", { name: "Cerrar" }));
    expect(heard).toHaveBeenCalled();
    expect(screen.queryByText("Tus agentes, en un solo lugar")).toBeNull();
    expect(sent("agents_bring")).toHaveLength(0);
    window.removeEventListener(AGENTS_CHANGED, heard);
  });
});

describe("«Sync» from the folder's line, its menu or Settings", () => {
  const state = (more: Partial<AgentsState> = {}): AgentsState => ({
    folder: "a",
    held: 1,
    consented: true,
    allOn: false,
    assistants: [],
    ...more,
  });
  const syncing = (
    now: AgentsState,
    round: Rounded = { told: [], both: [], loose: [], unled: [] },
  ) =>
    (ipc.answer = (cmd) => {
      if (cmd === "agents_state") return Promise.resolve(now);
      if (cmd === "agents_turn") return Promise.resolve(round);
      if (cmd === "agents_look")
        return Promise.resolve(look({ consented: now.consented, allOn: now.allOn }));
      return Promise.resolve(false);
    });
  const settle = () => act(() => new Promise((ready) => setTimeout(ready, 0)));

  beforeEach(() => showAll());

  it("does nothing when Agents already syncs", async () => {
    syncing(state({ allOn: true }));
    render(<AgentsKeeper />);
    act(() => syncAgents());
    await waitFor(() => expect(sent("agents_state").length).toBeGreaterThan(0));
    await settle();
    expect(sent("agents_turn")).toHaveLength(0);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("opens the first time on a computer that never said yes", async () => {
    syncing(state({ consented: false }));
    render(<AgentsKeeper />);
    act(() => syncAgents());
    expect(await screen.findByText("Tus agentes, en un solo lugar")).toBeTruthy();
    expect(sent("agents_turn")).toHaveLength(0);
  });

  it("opens the first time while there is no folder", async () => {
    syncing(state({ folder: null }));
    render(<AgentsKeeper />);
    act(() => syncAgents());
    expect(await screen.findByText("Tus agentes, en un solo lugar")).toBeTruthy();
    expect(sent("agents_turn")).toHaveLength(0);
  });

  it("syncs at once where this computer said yes, shows it again and asks nothing more", async () => {
    syncing(state());
    hide(AGENTS);
    hide("a");
    const told = vi.fn();
    window.addEventListener(AGENTS_CHANGED, told);
    render(<AgentsKeeper />);
    act(() => syncAgents());
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    expect(sent("agents_turn")[0].args).toEqual({ assistant: null, on: true });
    expect(isHidden(AGENTS)).toBe(false);
    expect(isHidden("a")).toBe(false);
    await waitFor(() => expect(told).toHaveBeenCalled());
    expect(desk.papersChanged).toHaveBeenCalled();
    window.removeEventListener(AGENTS_CHANGED, told);
    await settle();
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("asks about a file on both sides as soon as syncing finds one", async () => {
    const both = {
      at: "/h/.claude/CLAUDE.md",
      assistant: "claude",
      composed: false,
      lines: [],
    };
    const round = { told: [], both: [both], loose: [], unled: [] };
    syncing(state(), round);
    const answer = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "agents_look"
        ? Promise.resolve(look({ consented: true, allOn: true, both: [both] }))
        : answer(cmd, args);
    render(<AgentsKeeper />);
    act(() => syncAgents());
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    expect(await screen.findByRole("dialog", { name: "Existe en los dos lados" })).toBeTruthy();
  });

  it("opens no review when syncing finds nothing waiting", async () => {
    syncing(state());
    render(<AgentsKeeper />);
    act(() => syncAgents());
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    await settle();
    expect(sent("agents_look")).toHaveLength(0);
  });

  it("says why syncing failed", async () => {
    ipc.answer = () => Promise.reject({ code: "internal" });
    render(<AgentsKeeper />);
    act(() => syncAgents());
    await waitFor(() => expect(desk.setError).toHaveBeenCalled());
  });

  it("shows again an Agents hidden while it syncs, and leaves the sync on", async () => {
    syncing(state({ allOn: true }));
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    hide("a");
    render(<AgentsKeeper />);
    await waitFor(() => expect(isHidden("a")).toBe(false));
    expect(sent("agents_turn")).toHaveLength(0);
  });

  it("shows again a folder inside Agents hidden while it syncs", async () => {
    syncing(state({ allOn: true }));
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    hide("claude");
    render(<AgentsKeeper />);
    await waitFor(() => expect(isHidden("claude")).toBe(false));
  });

  it("keeps Agents hidden once hiding stopped its sync, though the state it held still synced", async () => {
    syncing(state({ allOn: true }));
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_state").length).toBeGreaterThan(0));
    await settle();
    syncing(state({ allOn: false }));
    act(() => hide(AGENTS));
    await waitFor(() => expect(sent("agents_state").length).toBeGreaterThan(1));
    await settle();
    expect(isHidden(AGENTS)).toBe(true);
  });

  it("leaves an Agents hidden while it does not sync where it is", async () => {
    syncing(state());
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    hide(AGENTS);
    render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_state").length).toBeGreaterThan(0));
    await settle();
    expect(isHidden(AGENTS)).toBe(true);
    expect(sent("agents_turn")).toHaveLength(0);
  });
});

describe("the log stirring from outside", () => {
  it("tells the cards and the sidebar to look again", () => {
    render(<AgentsKeeper />);
    const told = vi.fn();
    window.addEventListener(AGENTS_CHANGED, told);
    heard.stirred?.();
    window.removeEventListener(AGENTS_CHANGED, told);
    expect(told).toHaveBeenCalledTimes(1);
  });
});
