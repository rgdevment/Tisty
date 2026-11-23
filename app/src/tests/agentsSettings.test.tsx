import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  AGENTS_FIRST,
  AGENTS_MORE,
  AGENTS_STAYED,
  AGENTS_SYNC,
  type AgentsLook,
  type AgentsState,
} from "../agentsSync";
import { AGENTS, hide, isHidden, showAll } from "../hiding";
import { adopt } from "../locales";
import { asksSure, type Buttons } from "../sure";
import AgentsSettings from "../ui/AgentsSettings";
import { LiveRows } from "./rowsShell";

const ipc = vi.hoisted(() => ({
  asked: [] as string[],
  calls: [] as { cmd: string; args: unknown }[],
  refuses: "",
  state: null as unknown,
  look: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => {
    ipc.asked.push(cmd);
    ipc.calls.push({ cmd, args });
    if (cmd === ipc.refuses) return Promise.reject(new Error("It failed"));
    if (cmd === "agents_state") return Promise.resolve(ipc.state);
    if (cmd === "agents_look") return Promise.resolve(ipc.look);
    return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
  },
}));

const state = (more: Partial<AgentsState> = {}): AgentsState => ({
  folder: null,
  held: 0,
  consented: false,
  allOn: true,
  assistants: [{ id: "claude", name: "Claude", on: true }],
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

const heard = (event: string) => {
  const one = vi.fn();
  window.addEventListener(event, one);
  return { one, off: () => window.removeEventListener(event, one) };
};

beforeEach(() => {
  adopt("en");
  showAll();
  ipc.asked = [];
  ipc.calls = [];
  ipc.refuses = "";
  ipc.state = state();
  ipc.look = look();
});

describe("the Agents card in Settings", () => {
  it("says what it found with nothing brought, and both buttons open the first time", async () => {
    const first = heard(AGENTS_FIRST);
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(await screen.findByText("I found Claude on this computer.")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Bring" }));
    await userEvent.click(screen.getByRole("button", { name: "Bring and sync" }));
    expect(first.one).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("button", { name: "Sync" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Stop syncing" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Hide" }));
    expect(isHidden(AGENTS)).toBe(true);
    first.off();
  });

  it("reads nothing of an agent's before this computer's yes", async () => {
    ipc.state = state({ folder: "A" });
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    await screen.findByText("I found Claude on this computer.");
    expect(ipc.asked).not.toContain("agents_look");
    expect(screen.queryByRole("switch")).toBeNull();
  });

  it("says a brought folder does not sync, and offers Sync, another file and Hide", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false, held: 2 });
    ipc.look = look({ allOn: false });
    const sync = heard(AGENTS_SYNC);
    const more = heard(AGENTS_MORE);
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(await screen.findByText("Not synced with your agents")).toBeTruthy();
    expect(screen.queryByText("I found Claude on this computer.")).toBeNull();
    expect(screen.queryByRole("switch")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Sync" }));
    await userEvent.click(screen.getByRole("button", { name: "Bring another file" }));
    expect(sync.one).toHaveBeenCalledTimes(1);
    expect(more.one).toHaveBeenCalledTimes(1);
    await userEvent.click(screen.getByRole("button", { name: "Hide" }));
    expect(isHidden(AGENTS)).toBe(true);
    sync.off();
    more.off();
  });

  it("offers no other file on a folder brought from another computer that never said yes", async () => {
    ipc.state = state({ folder: "A", consented: false, held: 1 });
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(await screen.findByText("Not synced with your agents")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Bring another file" })).toBeNull();
  });

  it("shows the knobs while it syncs, and stops syncing on a click", async () => {
    ipc.state = state({ folder: "A", consented: true });
    const more = heard(AGENTS_MORE);
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(await screen.findByRole("switch", { name: "Sync Claude" })).toBeTruthy();
    expect(document.querySelector('[data-mark="claude"]')).toBeTruthy();
    expect(screen.queryByText("I found Claude on this computer.")).toBeNull();
    expect(screen.queryByText("Not synced with your agents")).toBeNull();
    expect(screen.getByRole("button", { name: "Hide" })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Bring another file" }));
    expect(more.one).toHaveBeenCalledTimes(1);
    more.off();
    ipc.calls = [];
    await userEvent.click(screen.getByRole("button", { name: "Stop syncing" }));
    await waitFor(() =>
      expect(ipc.calls).toContainEqual({
        cmd: "agents_turn",
        args: { assistant: null, on: false },
      }),
    );
  });

  it("hides a syncing Agents only once the person agrees to stop syncing first", async () => {
    showAll();
    ipc.state = state({ folder: "A", consented: true });
    const asked: { said: string; buttons?: Buttons }[] = [];
    let yes = false;
    asksSure((said, buttons) => {
      asked.push({ said, buttons });
      return Promise.resolve(yes);
    });
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    const hiding = await screen.findByRole("button", { name: "Hide" });
    ipc.calls = [];
    await userEvent.click(hiding);
    await waitFor(() => expect(asked).toHaveLength(1));
    expect(asked[0]).toEqual({
      said: "To hide Agents, first stop syncing. Your agent files are not touched.",
      buttons: { yes: "Stop syncing and hide", no: "Cancel" },
    });
    expect(ipc.calls.some((one) => one.cmd === "agents_turn")).toBe(false);
    expect(isHidden(AGENTS)).toBe(false);
    yes = true;
    await userEvent.click(hiding);
    await waitFor(() => expect(isHidden(AGENTS)).toBe(true));
    expect(ipc.calls).toContainEqual({ cmd: "agents_turn", args: { assistant: null, on: false } });
    asksSure(null);
    showAll();
  });

  it("turns one assistant back on and says why it does not sync", async () => {
    ipc.state = state({ folder: "A", consented: true });
    ipc.look = look({ assistants: [{ id: "claude", name: "Claude", on: false, files: [] }] });
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    const knob = await screen.findByRole("switch", { name: "Sync Claude" });
    expect(screen.getByText(/Not synced: its files stay as they are/)).toBeTruthy();
    ipc.calls = [];
    await userEvent.click(knob);
    await waitFor(() =>
      expect(ipc.calls).toContainEqual({
        cmd: "agents_turn",
        args: { assistant: "claude", on: true },
      }),
    );
  });

  it("asks for a review of what waits while it syncs", async () => {
    ipc.state = state({ folder: "A", consented: true });
    ipc.look = look({
      both: [{ assistant: "claude", at: "/c/CLAUDE.md" }] as unknown as AgentsLook["both"],
      loose: [
        { assistant: "claude", at: "/c/a.md" },
        { assistant: "claude", at: "/c/b.md" },
      ] as unknown as AgentsLook["loose"],
      unled: ["claude"],
    });
    const first = heard(AGENTS_FIRST);
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(
      await screen.findByText("2 files no longer belong to any rule or skill in Agents."),
    ).toBeTruthy();
    const reviews = screen.getAllByRole("button", { name: "Review" });
    expect(reviews).toHaveLength(2);
    for (const one of reviews) await userEvent.click(one);
    expect(first.one).toHaveBeenCalledTimes(2);
    first.off();
    ipc.calls = [];
    await userEvent.click(screen.getByRole("button", { name: "Put the line back" }));
    await waitFor(() => expect(ipc.calls.map((one) => one.cmd)).toContain("agents_lead"));
  });

  it("still lists the files the person left once Agents is gone", async () => {
    ipc.state = state({ folder: null, consented: true });
    ipc.look = look({
      folder: null,
      docs: 0,
      loose: [{ assistant: "claude", at: "/c/a.md", changed: false, stayed: true }],
    });
    const stayed = heard(AGENTS_STAYED);
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(
      await screen.findByText("1 file you left with your agents when you deleted it from Agents."),
    ).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Review" }));
    expect(stayed.one).toHaveBeenCalledTimes(1);
    stayed.off();
  });

  it("lists apart the files the person let stay and reviews them when asked", async () => {
    ipc.state = state({ folder: "A", consented: true });
    ipc.look = look({
      loose: [
        { assistant: "claude", at: "/c/a.md", changed: false, stayed: true },
        { assistant: "claude", at: "/c/b.md", changed: false, stayed: true },
        { assistant: "claude", at: "/c/c.md", changed: false, stayed: false },
      ],
    });
    const stayed = heard(AGENTS_STAYED);
    const first = heard(AGENTS_FIRST);
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(
      await screen.findByText(
        "2 files you left with your agents when you deleted them from Agents.",
      ),
    ).toBeTruthy();
    expect(
      screen.getByText("1 file no longer belongs to any rule or skill in Agents."),
    ).toBeTruthy();
    const [loose, left] = screen.getAllByRole("button", { name: "Review" });
    await userEvent.click(left);
    expect(stayed.one).toHaveBeenCalledTimes(1);
    expect(first.one).not.toHaveBeenCalled();
    await userEvent.click(loose);
    expect(first.one).toHaveBeenCalledTimes(1);
    stayed.off();
    first.off();
  });

  it("says why stopping failed", async () => {
    ipc.state = state({ folder: "A", consented: true });
    ipc.refuses = "agents_turn";
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    await userEvent.click(await screen.findByRole("button", { name: "Stop syncing" }));
    await waitFor(() => expect(ipc.asked.filter((one) => one === "agents_turn")).toHaveLength(1));
    await settle();
    expect(document.body.textContent).toMatch(/fail/i);
  });

  it("says when no assistant is found while it syncs", async () => {
    ipc.state = state({ folder: "A", consented: true, assistants: [] });
    ipc.look = look({ assistants: [] });
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    expect(await screen.findByText("No agents found on this computer.")).toBeTruthy();
  });

  it("says a hidden Agents does not sync and shows it again", async () => {
    ipc.state = state({ folder: "A", consented: true, allOn: false, held: 1 });
    ipc.look = look({ allOn: false });
    hide("A");
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    const said =
      "Hidden from the sidebar on this computer, and not synced: it writes no agent file.";
    expect(await screen.findByText(said)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Hide" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Show again" }));
    expect(isHidden("A")).toBe(false);
    await settle();
    expect(screen.queryByText(said)).toBeNull();
  });

  it("offers no Hide for an Agents already hidden", async () => {
    hide(AGENTS);
    render(
      <LiveRows>
        {(cards) => <AgentsSettings fail={cards.fail} quietly={cards.quietly} />}
      </LiveRows>,
    );
    await screen.findByText("I found Claude on this computer.");
    expect(screen.queryByRole("button", { name: "Hide" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Show again" }));
    await waitFor(() => expect(isHidden(AGENTS)).toBe(false));
  });
});
