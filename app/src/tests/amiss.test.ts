import { describe, expect, it, vi } from "vitest";
import { type Settled, whatWaits, whatWentAmiss } from "../core";
import { t } from "../locales";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

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
    const said = settled({ deferred: "budget" });

    expect(whatWentAmiss(said)).toBeNull();
    expect(whatWaits(said)).toBe("deferredBudget");
    expect(t("deferredBudget")).toMatch(/tomorrow/);
  });

  it("says why a round waits on the cloud, and nothing for one that retries by itself", () => {
    expect(whatWaits(settled({ deferred: "limit" }))).toBe("deferredLimit");
    expect(whatWaits(settled({ deferred: "offline" }))).toBe("deferredOffline");
    expect(whatWaits(settled({ deferred: "busy" }))).toBeNull();
    expect(whatWaits(settled({ deferred: "changed" }))).toBeNull();
    expect(whatWaits(settled({}))).toBeNull();
  });
});
