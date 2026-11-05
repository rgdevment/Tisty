import { describe, expect, it, vi } from "vitest";
import { discard, keptForTheEnd } from "../core";
import { holds, queued, settledThen } from "../saving";

const ipc = vi.hoisted(() => ({ calls: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.calls.push(cmd);
    return Promise.resolve(null);
  },
}));

describe("letting a task go", () => {
  it("waits for the last words typed on its page to land first", async () => {
    let land = () => {};
    const landed = new Promise<void>((done) => {
      land = done;
    });
    const flushed = vi.fn(() => {
      queued("words:t1", async () => {
        await landed;
        ipc.calls.push("doc_write");
      });
    });
    const release = holds(flushed);

    const going = discard("t1");
    await Promise.resolve();
    expect(flushed).toHaveBeenCalled();
    expect(ipc.calls).toEqual([]);
    land();
    await going;

    expect(ipc.calls).toEqual(["doc_write", "discard"]);
    release();
  });
});

describe("being warned that Windows is closing the window", () => {
  it("says it kept what was typed only once the last words have landed", async () => {
    ipc.calls.length = 0;
    let land = () => {};
    const landed = new Promise<void>((done) => {
      land = done;
    });
    const flushed = vi.fn(() => {
      queued("words:t2", async () => {
        await landed;
        ipc.calls.push("doc_write");
      });
    });
    const release = holds(flushed);

    settledThen(() => keptForTheEnd(1))();
    await Promise.resolve();
    expect(flushed).toHaveBeenCalled();
    expect(ipc.calls).toEqual([]);
    land();

    await vi.waitFor(() => expect(ipc.calls).toEqual(["doc_write", "kept_for_the_end"]));
    release();
  });

  it("still answers when the words could not land, so Windows is not kept waiting", async () => {
    ipc.calls.length = 0;
    const release = holds(() => {
      queued("words:t3", () => Promise.reject(new Error("the disk refused"))).catch(() => {});
    });

    settledThen(() => keptForTheEnd(1))();

    await vi.waitFor(() => expect(ipc.calls).toEqual(["kept_for_the_end"]));
    release();
  });
});
