import { beforeEach, describe, expect, it, vi } from "vitest";
import { decideAll } from "../deciding";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
}));

const asked = vi.hoisted(() => ({
  held: [] as ((sure: boolean) => void)[],
  said: [] as string[],
  waiting: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "docs") {
      return Promise.resolve({
        folders: [],
        docs: [
          {
            id: "1",
            file: "dev_a-0001",
            title: "Kit de transmisión",
            folder: null,
            archived: false,
          },
        ],
      });
    }
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: (said: string) => {
    asked.said.push(said);
    return asked.waiting
      ? new Promise<boolean>((resolve) => asked.held.push(resolve))
      : Promise.resolve(true);
  },
}));

beforeEach(() => {
  ipc.calls = [];
  asked.held = [];
  asked.said = [];
  asked.waiting = false;
});

const settled = () => ipc.calls.filter((one) => one.cmd === "settle_paper");

describe("deciding what to do with a document written on both sides", () => {
  it("keeps both when that is the answer", async () => {
    await decideAll(["dev_a-0001"]);

    expect(settled()).toHaveLength(1);
    expect(settled()[0].args.keep).toBe("both");
  });

  it("calls the document by its title, never by the name of its file", async () => {
    await decideAll(["dev_a-0001"]);

    expect(asked.said[0]).toMatch(/Kit de transmisión/);
    expect(asked.said[0]).not.toMatch(/dev_a-0001/);
  });

  it("says untitled rather than a file name when the document has no title", async () => {
    await decideAll(["dev_a-0002"]);

    expect(asked.said[0]).not.toMatch(/dev_a-0002/);
    expect(asked.said[0]).toMatch(/Untitled/i);
  });

  it("asks once for a document two syncs both call undecided", async () => {
    asked.waiting = true;
    const carrier = decideAll(["dev_a-0001"]);
    const panel = decideAll(["dev_a-0001"]);
    await vi.waitFor(() => expect(asked.held.length).toBeGreaterThan(0));
    asked.held.forEach((answer) => answer(true));
    await carrier;
    await panel;

    expect(settled()).toHaveLength(1);
  });
});
