import { beforeEach, describe, expect, it, vi } from "vitest";
import { decideAll } from "../deciding";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
}));

const asked = vi.hoisted(() => ({
  held: [] as ((sure: boolean) => void)[],
  waiting: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () =>
    asked.waiting
      ? new Promise<boolean>((resolve) => asked.held.push(resolve))
      : Promise.resolve(true),
}));

beforeEach(() => {
  ipc.calls = [];
  asked.held = [];
  asked.waiting = false;
});

const settled = () => ipc.calls.filter((one) => one.cmd === "settle_paper");

describe("deciding what to do with a document written on both sides", () => {
  it("keeps both when that is the answer", async () => {
    await decideAll(["dev_a-0001"]);

    expect(settled()).toHaveLength(1);
    expect(settled()[0].args.keep).toBe("both");
  });

  it("asks once for a document two syncs both call undecided", async () => {
    asked.waiting = true;
    const carrier = decideAll(["dev_a-0001"]);
    const panel = decideAll(["dev_a-0001"]);
    await Promise.resolve();
    asked.held.forEach((answer) => answer(true));
    await carrier;
    await panel;

    expect(settled()).toHaveLength(1);
  });
});
