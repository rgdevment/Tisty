import { beforeEach, describe, expect, it, vi } from "vitest";
import { walkThrough } from "../apart";

const calls: string[] = [];

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  save: () => Promise.resolve("D:/backup.zip"),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    calls.push(cmd);
    return Promise.resolve(cmd === "tidy_merged" ? 3 : true);
  },
}));

beforeEach(() => {
  calls.length = 0;
});

describe("merging two histories", () => {
  it("tidies what came twice only once the other side has come in", async () => {
    await walkThrough("merge");

    expect(calls).toEqual(["merge_stores", "sync_now", "tidy_merged"]);
  });

  it("tidies nothing when the folder's history is simply taken", async () => {
    await walkThrough("theirs");

    expect(calls).not.toContain("tidy_merged");
  });
});
