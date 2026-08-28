import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Series, Turn } from "../core";
import Routine from "../ui/Routine";

const ipc = vi.hoisted(() => ({ told: null as Series | null }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(ipc.told),
}));

const kept = (id: string, on: string, at: string, gaps?: string[]): Turn => ({
  id,
  status: "done",
  due: { at: `${on}T00:00:00`, tz: "UTC", floating: true, has_time: false },
  closed: at,
  gaps,
});

const series = (some: Partial<Series>): Series => ({
  last: "01A",
  title: "take the pill",
  turns: [],
  kept: 0,
  owed: 0,
  dropped: 0,
  open: 0,
  skipped: 0,
  streak: 0,
  longest: 0,
  measurable: true,
  ...some,
});

beforeEach(() => {
  ipc.told = null;
});

const shown = async (told: Series) => {
  ipc.told = told;
  render(<Routine task="01A" />);
  await screen.findByText(/the longest run/);
};

describe("a routine reads as behaviour, not as turns", () => {
  it("names the dates that went by instead of only counting them", async () => {
    await shown(
      series({
        turns: [
          kept("01", "2026-08-01", "2026-08-01T08:10:00Z"),
          kept("02", "2026-08-04", "2026-08-04T08:14:00Z", ["2026-08-02", "2026-08-03"]),
        ],
        kept: 2,
        owed: 4,
        skipped: 2,
        streak: 1,
        longest: 1,
      }),
    );

    const holes = screen.getAllByRole("listitem").map((one) => one.textContent);
    expect(holes).toHaveLength(2);
    expect(holes.every((one) => one && !/^\d{4}-/.test(one))).toBe(true);
    expect(holes.join(" ")).toMatch(/2/);
  });

  it("says plainly when a cadence has no gaps to show, rather than showing zero", async () => {
    await shown(
      series({
        measurable: false,
        turns: [kept("01", "2026-08-01", "2026-08-01T08:10:00Z")],
        kept: 1,
        owed: 1,
        streak: 1,
        longest: 1,
      }),
    );

    expect(screen.getByText(/counts from the day it was closed/i)).toBeTruthy();
    expect(screen.queryByText(/days missed/i)).toBeNull();
  });

  it("tells a clean run from an empty one", async () => {
    await shown(
      series({
        turns: [kept("01", "2026-08-01", "2026-08-01T08:10:00Z")],
        kept: 1,
        owed: 1,
        streak: 1,
        longest: 1,
      }),
    );

    expect(screen.getByText(/Every date the cadence asked for has a record/i)).toBeTruthy();
  });

  it("shows nothing at all when the task is not part of a routine", () => {
    ipc.told = null;
    const { container } = render(<Routine task="01A" />);

    expect(container.textContent).toBe("");
  });

  it("hides the hour of day, and its warning, when no turn was ever closed", async () => {
    await shown(series({ turns: [{ id: "01", status: "open" }], open: 1, owed: 0 }));

    expect(screen.queryByText(/What hour they close at/i)).toBeNull();
    expect(screen.queryByText(/travelling does not move these bars/i)).toBeNull();
    expect(screen.queryByText(/the usual hour/i)).toBeNull();
  });

  it("says the hour is the one on the clock where it was closed", async () => {
    await shown(
      series({
        turns: [kept("01", "2026-08-01", "2026-08-01T08:10:00Z")],
        kept: 1,
        owed: 1,
      }),
    );

    expect(screen.getByText(/travelling does not move these bars/i)).toBeTruthy();
  });

  it("reads each hour where the turn was closed, not where the reader is", async () => {
    const away: Turn = {
      ...kept("01", "2026-08-01", "2026-08-01T23:30:00Z"),
      zone: "Europe/Madrid",
    };
    await shown(series({ turns: [away], kept: 1, owed: 1 }));

    expect(screen.getAllByText("01:00").length).toBeGreaterThan(0);
    expect(screen.queryByText("23:00")).toBeNull();
  });

  it("leaves out the turns that were ticked days later", async () => {
    const late: Turn = {
      ...kept("01", "2026-08-01", "2026-08-10T09:00:00Z"),
      filled: true,
    };
    await shown(series({ turns: [late], kept: 1, owed: 1 }));

    expect(screen.queryByText(/What hour they close at/i)).toBeNull();
  });

  it("says a routine has no end when none was set", async () => {
    await shown(
      series({
        repeat: { from: "due", each: { every: 1, unit: "day" } },
        turns: [kept("01", "2026-08-01", "2026-08-01T08:10:00Z")],
        kept: 1,
        owed: 1,
      }),
    );

    expect(screen.getByText(/no end set/i)).toBeTruthy();
  });

  it("says when a routine ends, if it was given an end", async () => {
    await shown(
      series({
        repeat: { from: "due", each: { every: 1, unit: "day" }, until: "2026-12-31" },
        turns: [kept("01", "2026-08-01", "2026-08-01T08:10:00Z")],
        kept: 1,
        owed: 1,
      }),
    );

    expect(screen.getByText(/ends /i)).toBeTruthy();
  });

  it("averages only the turns that ran late, never the early ones", async () => {
    await shown(
      series({
        repeat: { from: "due", each: { every: 1, unit: "day" } },
        turns: [
          { ...kept("01", "2026-08-01", "2026-08-01T08:10:00Z"), late: 4 },
          { ...kept("02", "2026-08-02", "2026-08-02T08:10:00Z"), late: -2 },
        ],
        kept: 2,
        owed: 2,
      }),
    );

    expect(screen.getByText(/4 days late on average/i)).toBeTruthy();
  });

  it("counts every turn, kept or not, so the total is honest", async () => {
    await shown(
      series({
        turns: [
          kept("01", "2026-08-01", "2026-08-01T08:10:00Z"),
          { id: "02", status: "dropped" },
          { id: "03", status: "open" },
        ],
        kept: 1,
        owed: 2,
        dropped: 1,
        open: 1,
        streak: 1,
        longest: 1,
      }),
    );

    expect(
      screen.getByText("1/2"),
      "the turn still running is not something you failed",
    ).toBeTruthy();
  });
});
