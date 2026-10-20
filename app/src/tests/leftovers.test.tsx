import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Machine, Reviewed, Unvouching } from "../core";
import { fill, t } from "../locales";
import type { Word } from "../ui/Card";
import Leftovers, { lookAt } from "../ui/Leftovers";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
}));

const audit: Reviewed = {
  tasks: 7,
  lists: 2,
  agrees: true,
  loose: 0,
  looseBytes: 0,
  astray: [],
  unread: null,
  stranded: [],
  missing: [],
  events: 42,
  machines: [],
  logBytes: 0,
  docsBytes: 0,
  heldBytes: 0,
  heldFiles: 0,
};

const ONE = "5c2a9d37e104b6ca88f3d2504e8b71f0a3f19c2d7b40e81a6239ce4f0dab5712";
const OTHER = "b7102fae93c4d0167a51e8c2403f9bd68e2174ac55d930bf1c6e480729ab35df";

const astray: Machine = {
  id: "win1-0002",
  called: "salvia 07",
  name: "Portátil",
  when: 1_754_000_000,
  mine: false,
  signs: OTHER,
  confirmed: ONE,
  confirmedWhen: 1_754_000_000,
  turnedAway: null,
} as Machine;

const shown = (
  reviewed: Reviewed,
  more: { unproven?: Unvouching | null; said?: Word; onSync?: () => void } = {},
) =>
  render(
    <Leftovers
      audit={reviewed}
      twice={[]}
      unproven={more.unproven ?? null}
      busy={null}
      secs={0}
      said={more.said}
      run={() => {}}
      tell={() => {}}
      fail={() => {}}
      setAudit={() => {}}
      relook={() => {}}
      onSync={more.onSync ?? (() => {})}
    />,
  );

describe("what Maintenance asks you to look at", () => {
  it("shows nothing at all when the review found nothing", () => {
    shown(audit);

    expect(screen.queryByText(t("toLookAt"))).toBeNull();
  });

  it("names the document it could not read in a card of its own", () => {
    shown({ ...audit, unread: "Recetas" });

    expect(screen.getByText(t("unreadTitle"))).toBeTruthy();
    expect(screen.getByText(fill("looseUnread", "Recetas"))).toBeTruthy();
  });

  it("calls an untitled document by what the window calls it", () => {
    shown({ ...audit, unread: "" });

    expect(screen.getByText(fill("looseUnread", t("untitledDoc")))).toBeTruthy();
  });

  it("says when the attachments to verify could not all be read", () => {
    shown(audit, { unproven: { found: [], unread: "Recetas" } });

    expect(screen.getByText(fill("unvouchedUnread", "Recetas"))).toBeTruthy();
    expect(screen.queryByText(t("unvouchedTitle"))).toBeNull();
  });

  it("shows only the cards that have something, without a heading over each one", () => {
    shown({
      ...audit,
      stranded: [{ file: "mac0-0042", title: "Minutes", bytes: 21_000, when: 1_754_000_000 }],
    });

    expect(screen.getByText(t("upkeepLook"))).toBeTruthy();
    expect(screen.getAllByText(t("upkeepLook"))).toHaveLength(1);
    expect(screen.queryByText(t("missingTitle"))).toBeNull();
    expect(screen.queryByText(t("cacheOffTitle"))).toBeNull();
  });

  it("says each result once, on the card it belongs to", () => {
    shown(
      {
        ...audit,
        agrees: false,
        stranded: [{ file: "mac0-0042", title: "Minutes", bytes: 21_000, when: 1_754_000_000 }],
        missing: [{ id: "01M1", file: "win1-0007", title: "Recipes" }],
      },
      { said: { card: "recover", text: "Recovered as Minutes" } },
    );

    expect(screen.getAllByText("Recovered as Minutes")).toHaveLength(1);
  });

  it("keeps a card that just emptied long enough to say what happened", () => {
    shown(audit, { said: { card: "recover", text: fill("upkeepTakenIn", "Minutes") } });

    expect(screen.getByText(t("upkeepLook"))).toBeTruthy();
    expect(screen.getByText(fill("upkeepTakenIn", "Minutes"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("upkeepTakeInAll") })).toBeNull();
  });

  it("offers to rebuild the quick index when it does not match the history", () => {
    shown({ ...audit, agrees: false });

    expect(screen.getByText(t("cacheOffTitle"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("reviewRedo") })).toBeTruthy();
  });

  it("sends a computer that signs with another key to Your computers", async () => {
    const onSync = vi.fn();
    shown({ ...audit, machines: [astray] }, { onSync });

    expect(screen.getByText(t("keysTitle"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("keysSee") }));
    expect(onSync).toHaveBeenCalledTimes(1);
  });

  it("counts one thing to look at per card that has something", () => {
    expect(Object.values(lookAt(audit, [], null)).filter(Boolean)).toHaveLength(0);
    const found = lookAt(
      { ...audit, agrees: false, unread: "Recetas", machines: [astray] },
      [{ name: "Trabajo", lists: 2, tasks: 4 }],
      { found: [{ at: "attachments/a.pdf", bytes: 1 }], unread: "Recetas" },
    );
    expect(Object.values(found).filter(Boolean)).toHaveLength(5);
  });
});
