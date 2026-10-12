import { describe, expect, it, vi } from "vitest";
import { type Settled, whatWentAmiss } from "../core";
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

  it("says a day's budget ran out without calling it an error, and when syncing goes on", () => {
    const said = whatWentAmiss(settled({ deferred: "budget" }));

    expect(said).toBe("deferredBudget");
    expect(t("deferredBudget")).toMatch(/tomorrow/);
  });

  it("names every reason a cloud round can leave work for later", () => {
    for (const [reason, word] of [
      ["limit", "deferredLimit"],
      ["offline", "deferredOffline"],
      ["busy", "deferredBusy"],
      ["changed", "deferredChanged"],
    ] as const) {
      expect(whatWentAmiss(settled({ deferred: reason }))).toBe(word);
    }
  });

  it("lets what needs the person speak before what only waits", () => {
    expect(whatWentAmiss(settled({ astray: ["uno-0001"], deferred: "budget" }))).toBe(
      "someDocsAstray",
    );
  });
});
