import { beforeEach, describe, expect, it, vi } from "vitest";
import { docNew } from "../core";
import { spawned } from "../making";

const ipc = vi.hoisted(() => ({ calls: [] as { cmd: string; args: Record<string, unknown> }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return Promise.resolve(cmd === "doc_new" ? { id: "d1", title: "" } : null);
  },
}));

beforeEach(() => {
  ipc.calls = [];
});

describe("making a document", () => {
  it("asks to open an assistant's one rules file only for «New document»", async () => {
    await docNew("f1", undefined, true);
    expect(ipc.calls[0]).toEqual({
      cmd: "doc_new",
      args: { folder: "f1", pageOf: undefined, existing: true },
    });
  });

  it("never asks for an existing document where it writes words of its own over it", async () => {
    await spawned("Plan", "f1");
    const made = ipc.calls.find((one) => one.cmd === "doc_new");
    expect(made?.args.existing).toBeFalsy();
  });
});
