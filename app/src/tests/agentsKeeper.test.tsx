import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { type AgentsLook, bringMoreAgents } from "../agentsSync";
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
    await userEvent.click(await screen.findByRole("button", { name: "Ahora no" }));
    expect(heard).toHaveBeenCalled();
    expect(screen.queryByText("Tus agentes, en un solo lugar")).toBeNull();
    window.removeEventListener(AGENTS_CHANGED, heard);
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
