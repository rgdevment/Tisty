import { describe, expect, it, vi } from "vitest";
import { discard } from "../core";
import { holds, queued } from "../saving";

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
