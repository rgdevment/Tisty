import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DoorDue } from "../core";
import Door from "../ui/Door";
import { knowAgents } from "../who";

const ipc = vi.hoisted(() => ({ sent: [] as string[], refuse: false }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.sent.push(cmd);
    if (ipc.refuse) return Promise.reject(new Error("the config would not take it"));
    return Promise.resolve(null);
  },
}));

function shown(due: DoorDue = { why: "installed", by: null }) {
  const said = { settled: 0, opened: 0, problems: [] as unknown[] };
  render(
    <Door
      apart="right-3"
      due={due}
      onSettled={() => (said.settled += 1)}
      onOpen={() => (said.opened += 1)}
      onError={(problem) => said.problems.push(problem)}
    />,
  );
  return said;
}

beforeEach(() => {
  ipc.sent = [];
  ipc.refuse = false;
  knowAgents({}, { clients: { "claude-code": "Claude Code" } });
});

describe("the card that offers the assistant a door", () => {
  it("announces itself and asks nothing until it is answered", () => {
    shown();

    expect(screen.getByRole("status")).toBeTruthy();
    expect(ipc.sent).toEqual([]);
  });

  it("walks to the settings rather than opening the door itself", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /open settings/i }));

    expect(said.opened).toBe(1);
    expect(ipc.sent).toEqual(["door_done"]);
    expect(said.settled).toBe(1);
  });

  it("goes for good on a refusal too, without walking anywhere", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /don't show again/i }));

    expect(said.opened).toBe(0);
    expect(ipc.sent).toEqual(["door_done"]);
    expect(said.settled).toBe(1);
  });

  it("is only silenced by the cross, which is not an answer", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /not now/i }));

    expect(said.settled).toBe(1);
    expect(said.opened).toBe(0);
    expect(ipc.sent).toEqual([]);
  });

  it("is silenced by Escape from anywhere, not only from inside it", async () => {
    const said = shown();

    document.body.focus();
    await userEvent.keyboard("{Escape}");

    expect(said.settled).toBe(1);
    expect(ipc.sent).toEqual([]);
  });

  it("says so when the answer cannot be written down", async () => {
    ipc.refuse = true;
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /don't show again/i }));

    expect(said.problems.length).toBe(1);
    expect(String(said.problems[0])).toMatch(/would not take it/);
  });
});

describe("the card for assistants wired to a closed door", () => {
  const mute: DoorDue = { why: "mute", by: null };

  it("says the door is closed and lets them in right there", async () => {
    const said = shown(mute);

    expect(screen.getByText(/connected, but the door is closed/i)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /let it in/i }));

    await screen.findByText("Your assistant can work with you now.");
    expect(said.opened).toBe(0);
    expect(ipc.sent).toEqual(["agent_turn", "door_done"]);
    expect(said.settled).toBe(0);
  });

  it("still walks to the settings from its link", async () => {
    const said = shown(mute);

    await userEvent.click(screen.getByRole("button", { name: /see in settings/i }));

    expect(said.opened).toBe(1);
    expect(said.settled).toBe(1);
    expect(ipc.sent).toEqual(["door_done"]);
  });

  it("says so when the door cannot be opened, and stays", async () => {
    ipc.refuse = true;
    const said = shown(mute);

    await userEvent.click(screen.getByRole("button", { name: /let it in/i }));

    await waitFor(() => expect(said.problems.length).toBe(1));
    expect(screen.queryByText(/can work with you now/i)).toBeNull();
    expect(screen.getByText(/connected, but the door is closed/i)).toBeTruthy();
    expect(said.settled).toBe(0);
  });

  it("can be told not to come back", async () => {
    const said = shown(mute);

    await userEvent.click(screen.getByRole("button", { name: /don't show again/i }));

    expect(said.opened).toBe(0);
    expect(ipc.sent).toEqual(["door_done"]);
  });
});

describe("the card for an assistant turned away at the door", () => {
  const knocked: DoorDue = { why: "knocked", by: "claude-code" };

  it("names who tried, in the name the person knows it by", () => {
    shown(knocked);

    expect(screen.getByText("Claude Code tried to come in.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /don't show again/i })).toBeNull();
  });

  it("speaks of an assistant when it gave no name", () => {
    shown({ why: "knocked", by: null });

    expect(screen.getByText("An assistant tried to come in.")).toBeTruthy();
  });

  it("opens the door right there, names who came in and counts the knock as heard", async () => {
    const said = shown(knocked);

    await userEvent.click(screen.getByRole("button", { name: /let it in/i }));

    await screen.findByText("Claude Code can work with you now.");
    expect(ipc.sent).toEqual(["agent_turn", "door_heard"]);
    expect(said.opened).toBe(0);
  });

  it("goes on any click once it has said so", async () => {
    const said = shown(knocked);
    await userEvent.click(screen.getByRole("button", { name: /let it in/i }));
    await screen.findByText(/can work with you now/i);

    await userEvent.click(document.body);

    expect(said.settled).toBe(1);
  });

  it("goes by itself after a moment", async () => {
    const said = shown(knocked);
    await userEvent.click(screen.getByRole("button", { name: /let it in/i }));
    await screen.findByText(/can work with you now/i);

    await waitFor(() => expect(said.settled).toBe(1), { timeout: 4000 });
  });

  it("walks to the assistants from its link and counts the knock as heard", async () => {
    const said = shown(knocked);

    await userEvent.click(screen.getByRole("button", { name: /see in settings/i }));

    expect(said.opened).toBe(1);
    expect(said.settled).toBe(1);
    expect(ipc.sent).toEqual(["door_heard"]);
  });

  it("is heard as well when put off, so the same knock does not return", async () => {
    const said = shown(knocked);

    await userEvent.click(screen.getByRole("button", { name: /not now/i }));

    expect(said.opened).toBe(0);
    expect(said.settled).toBe(1);
    expect(ipc.sent).toEqual(["door_heard"]);
  });
});

describe("the card for a door Tisty opened on its own", () => {
  const opened: DoorDue = { why: "opened", by: null };

  it("says the door is open and walks to the settings to close it", async () => {
    const said = shown(opened);

    expect(screen.getByText(/door is open/i)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /open settings/i }));

    expect(said.opened).toBe(1);
    expect(ipc.sent).toEqual(["door_done"]);
  });

  it("never opens or closes the door itself", async () => {
    shown(opened);

    await userEvent.click(screen.getByRole("button", { name: /don't show again/i }));

    expect(ipc.sent).toEqual(["door_done"]);
  });
});
