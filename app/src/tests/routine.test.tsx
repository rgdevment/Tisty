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
  title: "take the pill",
  turns: [],
  kept: 0,
  dropped: 0,
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
        skipped: 2,
        streak: 1,
        longest: 1,
      }),
    );

    expect(screen.getByText("2026-08-02")).toBeTruthy();
    expect(screen.getByText("2026-08-03")).toBeTruthy();
  });

  it("says plainly when a cadence has no gaps to show, rather than showing zero", async () => {
    await shown(
      series({
        measurable: false,
        turns: [kept("01", "2026-08-01", "2026-08-01T08:10:00Z")],
        kept: 1,
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
        streak: 1,
        longest: 1,
      }),
    );

    expect(screen.getByText(/Not one date went by/i)).toBeTruthy();
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
        dropped: 1,
        streak: 1,
        longest: 1,
      }),
    );

    expect(screen.getByText("1/3")).toBeTruthy();
  });
});
