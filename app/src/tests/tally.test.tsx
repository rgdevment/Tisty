import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Series } from "../core";
import Tally from "../ui/Tally";

const ipc = vi.hoisted(() => ({ told: [] as Series[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(ipc.told),
}));

const series = (some: Partial<Series>): Series => ({
  last: "01A",
  title: "tomar píldoras",
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

const shown = async (counts: Record<string, number>, told: Series[] = []) => {
  ipc.told = told;
  const drawn = render(<Tally counts={counts} />);
  if (counts.archive) await screen.findByText("closed");
  return drawn;
};

beforeEach(() => {
  ipc.told = [];
});

describe("what the archive says before it is read", () => {
  it("says nothing at all when nothing has been closed", async () => {
    const { container } = await shown({ archive: 0, stories: 0 });

    expect(container.textContent).toBe("");
  });

  it("says nothing when it was not even told what the archive holds", async () => {
    const { container } = await shown({});

    expect(container.textContent).toBe("");
  });

  it("reads a layer nobody counted as none of it, and lays the rest at the routines", async () => {
    await shown({ archive: 4 });

    expect(screen.getAllByText("4")).toHaveLength(2);
    expect(screen.getAllByText("0")).toHaveLength(2);
  });

  it("counts what each layer holds, and stays quiet about routines nobody keeps", async () => {
    await shown({ archive: 23, stories: 5, routines: 1, traces: 13 });

    expect(screen.getByText("23")).toBeTruthy();
    expect(screen.getByText("13")).toBeTruthy();
    expect(screen.getByText(/in one routine/)).toBeTruthy();
    expect(screen.queryByText(/longest run/)).toBeNull();
    expect(screen.queryByText(/days missed/)).toBeNull();
  });

  it("adds what the routines came to once there are any", async () => {
    await shown({ archive: 23, stories: 5, routines: 2, traces: 13 }, [
      series({ kept: 5, owed: 5, longest: 5, streak: 5 }),
      series({ kept: 3, owed: 4, longest: 2, skipped: 1 }),
    ]);

    expect(await screen.findByText("8/9")).toBeTruthy();
    expect(screen.getByText(/longest run/)).toBeTruthy();
    expect(screen.getByText(/days missed/)).toBeTruthy();
  });

  it("counts no gap in a series that is not measured against a calendar", async () => {
    await shown({ archive: 5, stories: 0, routines: 1, traces: 0 }, [
      series({ kept: 5, owed: 5, longest: 5, skipped: 4, measurable: false }),
    ]);

    expect(await screen.findByText("5/5")).toBeTruthy();
    expect(screen.queryByText(/days missed/)).toBeNull();
  });

  it("counts turns where a layer counts series, so the four of them add up", async () => {
    await shown({ archive: 23, stories: 5, routines: 1, traces: 13 });

    const said = screen.getAllByRole("term").map((one) => Number(one.textContent));
    const [closed, ...layers] = said;

    expect(closed).toBe(23);
    expect(layers.slice(0, 3).reduce((a, b) => a + b, 0)).toBe(closed);
  });
});
