import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Pick, Rift } from "../core";
import { decideAll, decidesByBlock } from "../deciding";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
}));

const torn = vi.hoisted(() => ({
  said: { rifts: [] as Rift[], print: "" },
  refuses: false,
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
    if (cmd === "paper_rifts") return Promise.resolve(torn.said);
    if (cmd === "weave_paper" && torn.refuses) return Promise.reject(new Error("cannotWeave"));
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
  torn.said = { rifts: [], print: "" };
  torn.refuses = false;
  decidesByBlock(null);
  ipc.calls = [];
  asked.held = [];
  asked.said = [];
  asked.waiting = false;
});

const settled = () => ipc.calls.filter((one) => one.cmd === "settle_paper");

const woven = () => ipc.calls.filter((one) => one.cmd === "weave_paper");

describe("deciding a document block by block", () => {
  const rift: Rift = { was: ["antes"], mine: ["lo mio"], theirs: ["lo suyo"] };

  it("carries the print of what was shown back with the answers", async () => {
    torn.said = { rifts: [rift], print: "huella-1" };
    decidesByBlock(async () => ["mine"] as Pick[]);

    await decideAll(["dev_a-0001"]);

    expect(woven()).toHaveLength(1);
    expect(woven()[0].args.print).toBe("huella-1");
    expect(woven()[0].args.picks).toEqual(["mine"]);
  });

  it("writes nothing when the person closes without answering", async () => {
    torn.said = { rifts: [rift], print: "huella-1" };
    decidesByBlock(async () => null);

    await decideAll(["dev_a-0001"]);

    expect(woven()).toHaveLength(0);
    expect(settled()).toHaveLength(0);
  });

  it("asks outright rather than leaving the person stuck when the weave is refused", async () => {
    torn.said = { rifts: [rift], print: "huella-1" };
    torn.refuses = true;
    decidesByBlock(async () => ["mine"] as Pick[]);

    await decideAll(["dev_a-0001"]);

    expect(settled()).toHaveLength(1);
  });

  it("falls back to asking outright when the blocks cannot be worked out", async () => {
    torn.said = { rifts: [], print: "" };
    decidesByBlock(async () => ["mine"] as Pick[]);

    await decideAll(["dev_a-0001"]);

    expect(woven()).toHaveLength(0);
    expect(settled()).toHaveLength(1);
  });
});

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
    for (const answer of asked.held) answer(true);
    await carrier;
    await panel;

    expect(settled()).toHaveLength(1);
  });
});
