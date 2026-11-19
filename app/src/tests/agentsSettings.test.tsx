import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AGENTS_LIGHT, type AgentsLook, type AgentsState } from "../agentsSync";
import { AGENTS, hide, isHidden, showAll } from "../hiding";
import { adopt } from "../locales";
import AgentsSettings from "../ui/AgentsSettings";

const ipc = vi.hoisted(() => ({
  asked: [] as string[],
  state: null as unknown,
  look: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.asked.push(cmd);
    if (cmd === "agents_state") return Promise.resolve(ipc.state);
    if (cmd === "agents_look") return Promise.resolve(ipc.look);
    return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
  },
}));

const state = (more: Partial<AgentsState> = {}): AgentsState => ({
  folder: null,
  consented: false,
  allOn: true,
  assistants: [{ id: "claude", name: "Claude" }],
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
  assistants: [{ id: "claude", name: "Claude", on: true, files: [] }],
  both: [],
  loose: [],
  unled: [],
  ...more,
});

const settle = () => act(() => new Promise((ready) => setTimeout(ready, 0)));

beforeEach(() => {
  adopt("en");
  showAll();
  ipc.asked = [];
  ipc.state = state();
  ipc.look = look();
});

describe("the AGENTS card in Settings", () => {
  it("says what it found and offers the notice's own actions while off", async () => {
    const lit = vi.fn();
    window.addEventListener(AGENTS_LIGHT, lit);
    render(<AgentsSettings busy={null} />);
    expect(await screen.findByText("I found Claude on this computer.")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Turn on" }));
    expect(lit).toHaveBeenCalledTimes(1);
    await userEvent.click(screen.getByRole("button", { name: "Hide" }));
    expect(isHidden(AGENTS)).toBe(true);
    window.removeEventListener(AGENTS_LIGHT, lit);
  });

  it("reads nothing of an agent's before this computer's yes", async () => {
    ipc.state = state({ folder: "A" });
    render(<AgentsSettings busy={null} />);
    await screen.findByText("I found Claude on this computer.");
    expect(ipc.asked).not.toContain("agents_look");
    expect(screen.queryByRole("switch")).toBeNull();
  });

  it("keeps the switch and the found line together while switched off", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    ipc.look = look({ allOn: false });
    render(<AgentsSettings busy={null} />);
    expect(await screen.findByText("I found Claude on this computer.")).toBeTruthy();
    expect(await screen.findByRole("switch", { name: "Sync the AGENTS folder" })).toBeTruthy();
  });

  it("drops the found line once on", async () => {
    ipc.state = state({ folder: "A", consented: true });
    render(<AgentsSettings busy={null} />);
    await screen.findByRole("switch", { name: "Sync the AGENTS folder" });
    expect(screen.queryByText("I found Claude on this computer.")).toBeNull();
    expect(screen.queryByRole("button", { name: "Turn on" })).toBeNull();
  });

  it("says when no assistant is found while on", async () => {
    ipc.state = state({ folder: "A", consented: true, assistants: [] });
    ipc.look = look({ assistants: [] });
    render(<AgentsSettings busy={null} />);
    expect(await screen.findByText("No agents found on this computer.")).toBeTruthy();
  });

  it("says a hidden AGENTS is off and shows it again", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false });
    ipc.look = look({ allOn: false });
    hide("A");
    render(<AgentsSettings busy={null} />);
    expect(
      await screen.findByText(
        "Hidden from the sidebar on this computer, and off: it writes no agent file.",
      ),
    ).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Show again" }));
    expect(isHidden("A")).toBe(false);
    await settle();
    expect(
      screen.queryByText(
        "Hidden from the sidebar on this computer, and off: it writes no agent file.",
      ),
    ).toBeNull();
  });

  it("offers no Hide for an AGENTS already hidden", async () => {
    hide(AGENTS);
    render(<AgentsSettings busy={null} />);
    await screen.findByText("I found Claude on this computer.");
    expect(screen.queryByRole("button", { name: "Hide" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Show again" }));
    await waitFor(() => expect(isHidden(AGENTS)).toBe(false));
  });
});
