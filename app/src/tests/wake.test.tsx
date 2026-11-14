import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Wake from "../ui/Wake";

const ipc = vi.hoisted(() => ({ sent: [] as { cmd: string; args?: unknown }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.sent.push({ cmd, args });
    return Promise.resolve(null);
  },
}));

function shown(theirs = false) {
  const said = { settled: 0, problems: [] as unknown[] };
  render(
    <Wake
      apart="right-3"
      due={{ offered: true, wakes: false, theirs }}
      onSettled={() => (said.settled += 1)}
      onError={(problem) => said.problems.push(problem)}
    />,
  );
  return said;
}

const sent = () => ipc.sent.map((one) => one.cmd);

beforeEach(() => {
  ipc.sent = [];
});

describe("the card that offers to start with the computer", () => {
  it("asks nothing of the machine until it is answered", () => {
    shown();

    expect(screen.getByRole("status")).toBeTruthy();
    expect(ipc.sent).toEqual([]);
  });

  it("turns starting with the computer on and never asks again", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /open at sign-in/i }));

    expect(ipc.sent[0]).toEqual({ cmd: "wake_for", args: { wanted: true } });
    await vi.waitFor(() => expect(sent()).toEqual(["wake_for", "wake_asked"]));
    expect(said.settled).toBe(1);
  });

  it("goes for good on a refusal, leaving the computer as it was", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /no, thanks/i }));

    expect(sent()).toEqual(["wake_asked"]);
    expect(said.settled).toBe(1);
  });

  it("is only put off by the cross, which is not an answer", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /not now/i }));

    expect(ipc.sent).toEqual([]);
    expect(said.settled).toBe(1);
  });

  it("says where to turn it back on when Windows holds the switch", async () => {
    shown(true);

    expect(screen.getByText(/settings → apps → startup/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /open at sign-in/i })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: /got it/i }));

    expect(sent()).toEqual(["wake_asked"]);
  });
});
