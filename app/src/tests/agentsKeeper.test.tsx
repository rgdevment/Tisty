import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentsLook } from "../agentsSync";
import type { Filed, Folded, Papers } from "../core";
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
  consented: true,
  allOn: true,
  asks: false,
  assistants: [],
  both: [],
  ...more,
});

beforeEach(() => {
  adopt("es");
  ipc.calls = [];
  ipc.answer = (cmd) => {
    if (cmd === "agents_look") return Promise.resolve(look());
    if (cmd === "agents_empty") return Promise.resolve(false);
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

  it("lets an empty AGENTS go and steps out of it", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "agents_empty");
    desk.papers = { folders: agents, docs: [] };
    desk.here = "claude";
    render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_empty")).toHaveLength(1));
    await waitFor(() => expect(desk.setHere).toHaveBeenCalledWith(null));
    expect(desk.papersChanged).toHaveBeenCalled();
  });

  it("syncs once a change inside AGENTS settles, and says so when it fails", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_round" ? Promise.reject({ code: "internal" }) : Promise.resolve(false);
    desk.papers = { folders: agents, docs: [doc("x", "claude")] };
    render(<AgentsKeeper />);
    await waitFor(() => expect(sent("agents_round")).toHaveLength(1), { timeout: 2000 });
    await waitFor(() => expect(desk.setError).toHaveBeenCalled());
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
    await userEvent.click(await screen.findByRole("button", { name: "Ahora no" }));
    expect(heard).toHaveBeenCalled();
    expect(screen.queryByText("Tus agentes, en un solo lugar")).toBeNull();
    window.removeEventListener(AGENTS_CHANGED, heard);
  });
});
