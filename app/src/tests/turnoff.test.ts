import { beforeEach, describe, expect, it, vi } from "vitest";
import { turnedOff } from "../apart";
import type { Settings } from "../core";

const calls: { cmd: string; args: unknown }[] = [];
let refusals = 0;
let carried = "came";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: unknown) => {
    calls.push({ cmd, args });
    if (cmd === "choose_sync" && refusals > 0) {
      refusals -= 1;
      return Promise.reject({ code: "sharedAwayToLeave" });
    }
    if (cmd === "keep_settings") return Promise.resolve((args as { settings: Settings }).settings);
    if (cmd === "sync_now")
      return Promise.resolve({
        carried,
        undecided: [],
        unreadable: [],
        disowned: [],
        unconfirmed: [],
        astray: [],
        unprojected: false,
        joined: [],
      });
    return Promise.resolve(null);
  },
}));

const kept = { quiet: [], attachUpTo: 5_000_000, holds: "shared" } as unknown as Settings;

const holdsSet = () =>
  calls
    .filter((one) => one.cmd === "keep_settings")
    .map((one) => (one.args as { settings: Settings }).settings.holds);

beforeEach(() => {
  calls.length = 0;
  refusals = 0;
  carried = "came";
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
    expect(holdsSet()).toEqual(["everywhere"]);
  });

  it("just turns it off when nothing waits in the folder", async () => {
    await turnedOff(kept);

    expect(calls.map((one) => one.cmd)).toEqual(["choose_sync"]);
  });

  it("does not turn off while another round holds the folder, and puts the setting back", async () => {
    refusals = 1;
    carried = "busy";

    await expect(turnedOff(kept)).rejects.toEqual({ code: "sharedAwayToLeave" });
    expect(holdsSet()).toEqual(["everywhere", "shared"]);
    expect(calls.filter((one) => one.cmd === "choose_sync")).toHaveLength(1);
  });

  it("puts the setting back when the second attempt is refused too", async () => {
    refusals = 2;

    await expect(turnedOff(kept)).rejects.toEqual({ code: "sharedAwayToLeave" });
    expect(holdsSet()).toEqual(["everywhere", "shared"]);
  });

  it("passes any other refusal on untouched", async () => {
    refusals = 1;

    await expect(turnedOff(null)).rejects.toEqual({ code: "sharedAwayToLeave" });
  });
});
