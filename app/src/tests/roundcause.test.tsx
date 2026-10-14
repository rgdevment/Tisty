import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { type Awry, carrying } from "../carrying";
import { cloudStanding } from "../cloudState";
import type { Carrying, CloudStatus, Settled } from "../core";
import { adopt, fill, t } from "../locales";
import type { Run } from "../relocation";
import { causeSaid, lastingCause, syncingSaid, syncSaid } from "../syncSaid";
import Clouded from "../ui/Clouded";
import { pauseSaid, waitingSaid } from "../ui/RelocatingSteps";

const ipc = vi.hoisted(() => ({
  answers: {} as Record<string, () => Promise<unknown>>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => ipc.answers[cmd]?.() ?? Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

let onLine = true;

beforeEach(() => {
  ipc.answers = {};
  adopt("es");
  onLine = true;
  Object.defineProperty(navigator, "onLine", { configurable: true, get: () => onLine });
});

afterEach(() => {
  vi.useRealTimers();
  Reflect.deleteProperty(navigator, "onLine");
});

const DRIVE = t("cloudGoogle");
const AT = 1_800_000_000;

const status = (last: Partial<NonNullable<CloudStatus["last"]>>): CloudStatus => ({
  provider: "google",
  account: "0123",
  email: "ana@example.com",
  joined: true,
  unitsSpent: 0,
  unitsADay: 20000,
  bytesSpent: 0,
  bytesADay: 0,
  pausedHere: false,
  owed: 0,
  last: { at: AT, code: "deferred-offline", left: 0, names: [], ...last },
});

const failing = (cause: "blocked" | "filtered" | "slow", rounds = 2) =>
  status({ cause, rounds, since: AT - 60 * (rounds - 1) });

describe("what stands in the way of the cloud, once it lasts", () => {
  it("says each cause in its own words, with the provider's name", () => {
    expect(cloudStanding(failing("filtered"), AT)?.told).toBe(t("roundFiltered"));
    expect(cloudStanding(failing("blocked"), AT)?.told).toBe(fill("roundBlocked", DRIVE));
    expect(cloudStanding(failing("slow"), AT)?.told).toBe(fill("roundSlow", DRIVE));
    expect(fill("roundBlocked", DRIVE)).toContain("desde hace un rato");
    expect(cloudStanding(failing("filtered"), AT)?.urgent).toBe(true);
  });

  it("stays the plain offline word for a single failed round", () => {
    const once = cloudStanding(failing("blocked", 1), AT);

    expect(once?.said).toBe("stateOffline");
    expect(once?.why).toBe("stateOfflineWhy");
    expect(once?.urgent).toBeUndefined();
  });

  it("counts a cause as lasting after two rounds in a row or two minutes failing", () => {
    expect(lastingCause({ cause: "slow", rounds: 1, since: AT, at: AT })).toBeNull();
    expect(lastingCause({ cause: "slow", rounds: 2, since: AT - 30, at: AT })).toBe(
      fill("roundSlow", DRIVE),
    );
    expect(lastingCause({ cause: "slow", rounds: 1, since: AT - 120, at: AT })).toBe(
      fill("roundSlow", DRIVE),
    );
    expect(lastingCause({ cause: "slow", rounds: 1, since: AT - 30 }, AT)).toBeNull();
    expect(lastingCause({ rounds: 5, since: AT - 600, at: AT })).toBeNull();
  });

  it("lets a computer with no internet say so instead of blaming a filter", () => {
    onLine = false;

    expect(causeSaid("filtered")).toBeNull();
    expect(cloudStanding(failing("filtered"), AT)?.said).toBe("stateOffline");
    expect(cloudStanding(failing("filtered"), AT)?.why).toBe("stateOfflineWhy");
  });

  it("leaves a provider failing on its side to its own words", () => {
    const troubled = status({ code: "deferred-troubled", rounds: 3, since: AT - 600 });

    expect(cloudStanding(troubled, AT)?.said).toBe("stateTroubled");
  });

  it("shows the lasting cause in red on the sync card", async () => {
    ipc.answers.cloud_status = () =>
      Promise.resolve(failing("filtered", 3) as CloudStatus & Record<string, unknown>);
    render(
      <Clouded
        state={{ mode: "cloud", cloud: "ana@example.com" } as Carrying}
        busy={null}
        run={vi.fn() as never}
        fail={() => {}}
        carry={() => {}}
        onMove={() => {}}
      />,
    );

    const said = await screen.findByText(t("roundFiltered"));
    expect(said.className).toContain("text-urgent");
  });
});

describe("a round that runs long", () => {
  it("says for how long once it passes a minute", () => {
    expect(syncingSaid(59)).toBe(t("syncing_"));
    expect(syncingSaid(60)).toBe(fill("syncingFor", "1"));
    expect(syncingSaid(185)).toBe("Sincronizando desde hace 3 min…");
  });

  it("tells a round turned away how long the other one has run", () => {
    const busy = (running?: number) =>
      ({ carried: "busy", undecided: [], running }) as unknown as Settled;

    expect(syncSaid(busy())).toBe(t("syncBusy"));
    expect(syncSaid(busy(30))).toBe(t("syncBusy"));
    expect(syncSaid(busy(130))).toBe(fill("syncingFor", "2"));
  });

  it("counts the minutes on the card's button while the round goes on", async () => {
    vi.useFakeTimers();
    ipc.answers.cloud_status = () => new Promise(() => undefined);
    render(
      <Clouded
        state={{ mode: "cloud", cloud: "ana@example.com" } as Carrying}
        busy="sync"
        run={vi.fn() as never}
        fail={() => {}}
        carry={() => {}}
        onMove={() => {}}
      />,
    );
    expect(screen.getByRole("button", { name: t("syncing_") })).toBeTruthy();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(70_000);
    });

    expect(screen.getByRole("button", { name: fill("syncingFor", "1") })).toBeTruthy();
  });
});

describe("rounds nobody asked for", () => {
  it("say a lasting cause on the bottom line, and nothing for a blip", async () => {
    vi.useFakeTimers();
    let rounds = 1;
    ipc.answers.sync_state = () => Promise.resolve({ chosen: "cloud", asked: true });
    ipc.answers.sync_now = () =>
      Promise.resolve({
        carried: "same",
        undecided: [],
        deferred: {
          reason: "offline",
          left: 0,
          names: [],
          cause: "blocked",
          rounds,
          since: Math.floor(Date.now() / 1000),
        },
      });
    const told: (Awry | null)[] = [];
    const carried = carrying(
      () => {},
      () => {},
      (why) => told.push(why),
    );
    await vi.advanceTimersByTimeAsync(0);
    expect(told[told.length - 1]).toBeNull();

    rounds = 2;
    window.dispatchEvent(new Event("focus"));
    await vi.advanceTimersByTimeAsync(0);
    carried.stop();

    expect(told[told.length - 1]).toEqual({ why: "away", said: fill("roundBlocked", DRIVE) });
  });
});

describe("a move paused by what stands in the way", () => {
  const NOW = 100_000;
  const run = (cause?: "filtered" | "slow"): Run =>
    ({
      toward: { kind: "cloud", account: "0123", email: "ana@example.com" },
      going: false,
      watched: true,
      waited: "offline",
      pausedUntil: NOW + 5_000,
      result: {
        state: "waiting",
        stage: "filling",
        left: { files: 1, bytes: 1 },
        reason: "offline",
        cause,
      },
    }) as Run;

  it("names the cause instead of a vague fault", () => {
    expect(pauseSaid(run("filtered"), NOW)).toBe(
      `${t("roundFiltered")} · ${fill("moveRetryIn", "5")}`,
    );
    expect(
      waitingSaid({
        state: "waiting",
        stage: "filling",
        left: { files: 1, bytes: 1 },
        reason: "offline",
        cause: "slow",
      }),
    ).toBe(`${t("moveWaiting")} · ${fill("roundSlow", DRIVE)}`);
  });

  it("still says there is no internet when this computer is offline", () => {
    onLine = false;

    expect(pauseSaid(run("filtered"), NOW)).toBe(
      `${t("moveOffline")} · ${fill("moveRetryIn", "5")}`,
    );
  });
});
