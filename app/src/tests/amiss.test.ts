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

  it("keeps the plain notice when nothing of it is held back", () => {
    expect(whatWentAmiss(settled({ unconfirmed: ["dev_w"] }))).toBe("someoneUnconfirmed");
  });
});
