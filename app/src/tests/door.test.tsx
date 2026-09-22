import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Door from "../ui/Door";

const ipc = vi.hoisted(() => ({ sent: [] as string[], refuse: false }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.sent.push(cmd);
    if (ipc.refuse) return Promise.reject(new Error("the config would not take it"));
    return Promise.resolve(null);
  },
}));

function shown() {
  const said = { settled: 0, opened: 0, problems: [] as unknown[] };
  render(
    <Door
      apart="right-3"
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
