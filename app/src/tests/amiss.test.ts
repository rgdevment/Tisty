import { describe, expect, it, vi } from "vitest";
import { type Settled, type Waiting, whatWaits, whatWentAmiss } from "../core";
import { fill, t } from "../locales";
import { whatsLeft } from "../syncSaid";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const waiting = (reason: Waiting["reason"], left = 0, names: string[] = []): Waiting => ({
  reason,
  left,
  names,
});

const settled = (over: Partial<Settled>): Settled => ({
  carried: "same",
  undecided: [],
  unreadable: [],
  disowned: [],
  unconfirmed: [],
  waiting: [],
  astray: [],
  unprojected: false,
  joined: [],
  ...over,
});

describe("what a sync says went amiss", () => {
  it("says documents wait with a computer still waiting to be confirmed, and the way out", () => {
    const said = whatWentAmiss(settled({ unconfirmed: ["dev_w"], waiting: ["uno-0001"] }));

    expect(said).toBe("someoneUnconfirmedHolds");
    expect(t("someoneUnconfirmedHolds")).toMatch(/Maintenance/);
  });

  it("says it on a background sync too, which leaves the waiting machines to the sidebar", () => {
    expect(whatWentAmiss(settled({ unconfirmed: [], waiting: ["uno-0001"] }))).toBe(
      "someoneUnconfirmedHolds",
    );
  });

  it("keeps the plain notice when nothing of it is held back", () => {
    expect(whatWentAmiss(settled({ unconfirmed: ["dev_w"] }))).toBe("someoneUnconfirmed");
  });

  it("says a day's budget ran out as something that waits, never as what went wrong", () => {
    const said = settled({ deferred: waiting("budget") });

    expect(whatWentAmiss(said)).toBeNull();
    expect(whatWaits(said)).toBe("deferredBudget");
    expect(t("deferredBudget")).toMatch(/tomorrow/);
  });

  it("says why a round waits on the cloud, and nothing for one that retries by itself", () => {
    expect(whatWaits(settled({ deferred: waiting("limit") }))).toBe("deferredLimit");
    expect(whatWaits(settled({ deferred: waiting("offline") }))).toBe("deferredOffline");
    expect(whatWaits(settled({ deferred: waiting("busy") }))).toBeNull();
    expect(whatWaits(settled({ deferred: waiting("changed") }))).toBeNull();
    expect(whatWaits(settled({}))).toBeNull();
  });
});

describe("what a round left for later", () => {
  it("names the attachments by the name the person gave them, and counts the rest", () => {
    const said = settled({
      deferred: waiting("budget", 5, [
        "attachments/ab/foto.png",
        "attachments/cd/informe%20final.pdf",
        "attachments/ef/video.mp4",
        "attachments/gh/otra.png",
      ]),
    });

    expect(whatsLeft(said)).toBe(
      fill(
        "deferredLeft",
        "5",
        fill("deferredMore", "foto.png, informe final.pdf, video.mp4", "2"),
      ),
    );
  });

  it("names them all when there are few, and only counts what has no name", () => {
    expect(whatsLeft(settled({ deferred: waiting("limit", 1, ["attachments/ab/a.png"]) }))).toBe(
      fill("deferredLeft", "1", "a.png"),
    );
    expect(whatsLeft(settled({ deferred: waiting("changed", 4) }))).toBe(
      fill("deferredCount", "4"),
    );
    expect(whatsLeft(settled({ deferred: waiting("offline") }))).toBeNull();
    expect(whatsLeft(settled({}))).toBeNull();
  });
});
