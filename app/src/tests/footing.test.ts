import { describe, expect, it } from "vitest";
import type { Carrying, CloudStatus } from "../core";
import { footing, offered, type Seen } from "../footing";

const NOW = 1_800_000_000;

const carrying = (over: Partial<Carrying> = {}): Carrying => ({
  asked: true,
  open: 0,
  archived: 0,
  lists: 0,
  attachments: 0,
  weight: 0,
  carries: 0,
  mode: "cloud",
  cloudProvider: "google",
  cloudOffered: true,
  clouds: [],
  freesSpace: false,
  ...over,
});

const cloud = (over: Partial<CloudStatus> = {}): CloudStatus => ({
  provider: "google",
  account: "a",
  email: "a@example.invalid",
  joined: true,
  unitsSpent: 0,
  unitsADay: 0,
  bytesSpent: 0,
  bytesADay: 0,
  pausedHere: false,
  owed: 0,
  ...over,
});

const round = (code: string, at = NOW) => ({ at, code, left: 0, names: [] });

const seen = (over: Partial<Seen> = {}): Seen => ({
  state: carrying(),
  status: cloud(),
  coming: null,
  mine: false,
  offline: false,
  knocked: false,
  putOff: 0,
  stuck: null,
  now: NOW,
  ...over,
});

describe("the sync foot", () => {
  it("is not there when nothing syncs", () => {
    expect(footing(seen({ state: carrying({ mode: "alone" }) }))).toBeNull();
    expect(footing(seen({ state: carrying({ mode: "later" }) }))).toBeNull();
    expect(footing(seen({ state: carrying({ mode: "folder" }) }))).toBeNull();
    expect(footing(seen({ state: null }))).toBeNull();
  });

  it("is up to date and offers a sync when the last round went well", () => {
    const foot = footing(seen({ status: cloud({ last: round("ok") }) }));
    expect(foot).toEqual({ kind: "ok" });
    expect(foot && offered(foot)).toBe(true);
  });

  it("reads the same for a folder", () => {
    const foot = footing(
      seen({ state: carrying({ mode: "folder", chosen: "D:/Sync" }), status: null }),
    );
    expect(foot).toEqual({ kind: "ok" });
  });

  it("counts a round while it runs, and never offers another", () => {
    const foot = footing(seen({ coming: { done: 3, whole: 9 } }));
    expect(foot).toEqual({ kind: "syncing", done: 3, whole: 9 });
    expect(foot && offered(foot)).toBe(false);
    expect(footing(seen({ mine: true }))).toEqual({ kind: "syncing", done: 0, whole: 0 });
  });

  it("waits, without offering a sync, when the cloud asks to", () => {
    for (const code of ["cloudNotNow", "deferred-limit", "deferred-busy"]) {
      const foot = footing(seen({ status: cloud({ last: round(code) }) }));
      expect(foot).toEqual({ kind: "wait", troubled: false });
      expect(foot && offered(foot)).toBe(false);
    }
    expect(footing(seen({ status: cloud({ last: round("deferred-troubled") }) }))).toEqual({
      kind: "wait",
      troubled: true,
    });
  });

  it("says until when Tisty paused every computer", () => {
    const foot = footing(seen({ status: cloud({ haltedUntil: NOW + 600 }) }));
    expect(foot).toEqual({ kind: "halted", until: NOW + 600 });
    expect(foot && offered(foot)).toBe(false);
    expect(footing(seen({ status: cloud({ haltedUntil: NOW - 1 }) }))).toEqual({ kind: "ok" });
  });

  it("holds today's limit until the day turns", () => {
    expect(footing(seen({ status: cloud({ last: round("cloudSpent") }) }))).toEqual({
      kind: "spent",
    });
    expect(
      footing(seen({ status: cloud({ last: round("deferred-budget", NOW - 86_400) }) })),
    ).toEqual({ kind: "ok" });
  });

  it("is offline when the computer is, or the last round could not reach the cloud", () => {
    expect(footing(seen({ offline: true }))).toEqual({ kind: "offline" });
    expect(footing(seen({ status: cloud({ last: round("deferred-offline") }) }))).toEqual({
      kind: "offline",
    });
    const folder = carrying({ mode: "folder", chosen: "D:/Sync" });
    expect(footing(seen({ state: folder, status: null, offline: true }))).toEqual({ kind: "ok" });
  });

  it("asks to sign in again when the access is gone", () => {
    const foot = footing(seen({ status: cloud({ last: round("cloudLost") }) }));
    expect(foot).toEqual({ kind: "lost" });
    expect(foot && offered(foot)).toBe(true);
    expect(footing(seen({ stuck: { code: "cloudElsewhere" } }))).toEqual({ kind: "lost" });
  });

  it("offers to resume when the person paused it, above what the last round said", () => {
    const foot = footing(seen({ status: cloud({ pausedHere: true, last: round("cloudNotNow") }) }));
    expect(foot).toEqual({ kind: "paused" });
    expect(foot && offered(foot)).toBe(true);
  });

  it("points at what waits for the person before any wait that passes by itself", () => {
    const waiting = cloud({ last: round("cloudNotNow") });
    expect(footing(seen({ status: waiting, knocked: true }))).toEqual({
      kind: "look",
      putOff: false,
    });
    expect(footing(seen({ status: waiting, putOff: 2 }))).toEqual({ kind: "look", putOff: true });
    expect(footing(seen({ stuck: { code: "wouldReset" } }))).toEqual({
      kind: "look",
      putOff: false,
    });
    expect(footing(seen({ status: cloud({ last: round("cloudFull") }) }))).toEqual({
      kind: "look",
      putOff: false,
    });
  });
});
