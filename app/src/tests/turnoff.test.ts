import { beforeEach, describe, expect, it, vi } from "vitest";
import { turnedOff } from "../apart";
import type { Settings } from "../core";

const calls: { cmd: string; args: unknown }[] = [];
let refusals = 0;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: unknown) => {
    calls.push({ cmd, args });
    if (cmd === "choose_sync" && refusals > 0) {
      refusals -= 1;
      return Promise.reject({ code: "sharedAwayToLeave" });
    }
    if (cmd === "keep_settings") return Promise.resolve((args as { settings: Settings }).settings);
    return Promise.resolve(null);
  },
}));

const kept = { quiet: [], attachUpTo: 5_000_000, holds: "shared" } as unknown as Settings;

beforeEach(() => {
  calls.length = 0;
  refusals = 0;
});

describe("turning the shared folder off", () => {
  it("brings large attachments home first, then turns it off, in one click", async () => {
    refusals = 1;

    await turnedOff(kept);

    expect(calls.map((one) => one.cmd)).toEqual([
      "choose_sync",
      "keep_settings",
      "sync_now",
      "choose_sync",
    ]);
    expect((calls[1].args as { settings: Settings }).settings.holds).toBe("everywhere");
  });

  it("just turns it off when nothing waits in the folder", async () => {
    await turnedOff(kept);

    expect(calls.map((one) => one.cmd)).toEqual(["choose_sync"]);
  });

  it("passes any other refusal on untouched", async () => {
    refusals = 1;

    await expect(turnedOff(null)).rejects.toEqual({ code: "sharedAwayToLeave" });
  });
});
