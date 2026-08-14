import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { carrying } from "../carrying";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> =>
    Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd, args ?? {});
  },
}));

const state = { chosen: undefined as string | undefined, asked: true, backsUp: true, loose: 0 };
let carried: ReturnType<typeof carrying> | undefined;

beforeEach(() => {
  vi.useFakeTimers();
  ipc.calls = [];
  state.chosen = "G:/My Drive/tisty";
  ipc.answer = (cmd) =>
    cmd === "sync_state"
      ? Promise.resolve({ ...state })
      : Promise.resolve({ carried: "came", undecided: [] });
});

afterEach(() => {
  carried?.stop();
  carried = undefined;
  vi.useRealTimers();
});

const settle = () => vi.advanceTimersByTimeAsync(0);
const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

describe("carrying on its own", () => {
  it("hands over the documents at odds instead of dropping them", async () => {
    ipc.answer = (cmd) =>
      cmd === "sync_state"
        ? Promise.resolve({ ...state })
        : Promise.resolve({ carried: "came", undecided: ["dev_a-0001", "dev_b-0002"] });
    const said: string[][] = [];

    carried = carrying(
      () => {},
      (ids) => said.push(ids),
    );
    await settle();

    expect(said).toEqual([["dev_a-0001", "dev_b-0002"]]);
  });

  it("says nothing about documents when none of them are at odds", async () => {
    const said: string[][] = [];

    carried = carrying(
      () => {},
      (ids) => said.push(ids),
    );
    await settle();

    expect(said).toEqual([]);
  });

  it("brings the others home as soon as the window opens", async () => {
    carried = carrying(() => {});
    await settle();

    expect(sent("sync_now").length).toBe(1);
    expect(sent("sync_now")[0].args.way).toBe("pull");
  });

  it("stays put when no folder was ever chosen", async () => {
    state.chosen = undefined;
    carried = carrying(() => {});
    await settle();
    window.dispatchEvent(new Event("focus"));
    await settle();

    expect(sent("sync_now").length).toBe(0);
  });

  it("waits for the typing to stop before pushing", async () => {
    carried = carrying(() => {});
    await settle();
    ipc.calls = [];

    carried.changed();
    carried.changed();
    carried.changed();
    await vi.advanceTimersByTimeAsync(1_000);
    expect(sent("sync_now").length).toBe(0);

    await vi.advanceTimersByTimeAsync(5_000);
    expect(sent("sync_now").length).toBe(1);
    expect(sent("sync_now")[0].args.way).toBe("push");
  });

  it("does not pile one run on top of another", async () => {
    let release: (came: boolean) => void = () => {};
    ipc.answer = (cmd) =>
      cmd === "sync_state"
        ? Promise.resolve({ ...state })
        : new Promise((resolve) => {
            release = resolve;
          });

    carried = carrying(() => {});
    await settle();
    window.dispatchEvent(new Event("focus"));
    window.dispatchEvent(new Event("focus"));
    await settle();

    expect(sent("sync_now").length).toBe(1);
    release(false);
  });

  it("reloads only when something came back", async () => {
    const brought = vi.fn();
    ipc.answer = (cmd) =>
      cmd === "sync_state" ? Promise.resolve({ ...state }) : Promise.resolve("same");

    carried = carrying(brought);
    await settle();

    expect(sent("sync_now").length).toBe(1);
    expect(brought).not.toHaveBeenCalled();
  });

  it("swallows a folder that is not there", async () => {
    ipc.answer = (cmd) =>
      cmd === "sync_state"
        ? Promise.resolve({ ...state })
        : Promise.reject(new Error("noMeetingPlace"));

    carried = carrying(() => {});
    await settle();
    window.dispatchEvent(new Event("focus"));
    await settle();

    expect(sent("sync_now").length).toBe(2);
  });

  it("stops carrying once the folder is turned off", async () => {
    carried = carrying(() => {});
    await settle();
    state.chosen = undefined;
    carried.recheck();
    await settle();
    ipc.calls = [];

    window.dispatchEvent(new Event("focus"));
    await settle();

    expect(sent("sync_now").length).toBe(0);
  });

  it("remembers which direction it owed, not just that it owed one", async () => {
    let release: (answer: string) => void = () => {};
    ipc.answer = (cmd) =>
      cmd === "sync_state"
        ? Promise.resolve({ ...state })
        : new Promise((resolve) => {
            release = resolve;
          });

    carried = carrying(() => {});
    await settle();
    expect(sent("sync_now").length).toBe(1);

    window.dispatchEvent(new Event("focus"));
    await settle();
    release("same");
    await settle();

    expect(sent("sync_now").length).toBe(2);
    expect(sent("sync_now")[1].args.way).toBe("pull");
  });

  it("brings the folder in the moment it is switched for another", async () => {
    carried = carrying(() => {});
    await settle();
    ipc.calls = [];

    state.chosen = "D:/another/folder";
    carried.recheck();
    await settle();

    expect(sent("sync_now").length).toBe(1);
  });

  it("leaves no timer behind when it is stopped mid-carry", async () => {
    ipc.answer = (cmd) =>
      cmd === "sync_state" ? Promise.resolve({ ...state }) : new Promise(() => {});

    carried = carrying(() => {});
    await settle();
    carried.stop();
    carried = undefined;

    expect(vi.getTimerCount()).toBe(0);
  });

  it("does not treat a busy backend as a finished sync", async () => {
    const brought = vi.fn();
    ipc.answer = (cmd) =>
      cmd === "sync_state" ? Promise.resolve({ ...state }) : Promise.resolve("busy");

    carried = carrying(brought);
    await settle();

    expect(brought).not.toHaveBeenCalled();
  });
});
