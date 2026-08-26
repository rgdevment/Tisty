import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Page, Story } from "../core";
import Trail from "../ui/Trail";

const ipc = vi.hoisted(() => ({ told: { id: "01A", pages: [] } as Story }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(ipc.told),
}));

const day = (at: string) => ({ at, tz: "UTC", floating: true, has_time: false });

type Drop<T> = T extends unknown ? Omit<T, "n"> : never;
type Draft = Drop<Page>;

const pages = (...some: Draft[]): Page[] => some.map((one, n) => ({ ...one, n }) as Page);

beforeEach(() => {
  ipc.told = { id: "01A", pages: [] };
});

const shown = async (some: Draft[]) => {
  ipc.told = { id: "01A", pages: pages(...some) };
  render(<Trail task="01A" lists={[{ id: "01L", name: "Work", order: "a0" }]} />);
  await screen.findByRole("list");
};

describe("the trail", () => {
  it("keeps a deadline that moved, which the task itself no longer remembers", async () => {
    await shown([
      { at: "2026-07-30T11:02:00Z", by: "dev_a", chapter: "bounded", to: day("2026-08-12") },
      {
        at: "2026-08-12T08:55:00Z",
        by: "dev_a",
        chapter: "bounded",
        from: day("2026-08-12"),
        to: day("2026-08-19"),
      },
    ]);

    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText(/moves to/i)).toBeTruthy();
  });

  it("says a move was undone instead of hiding it", async () => {
    await shown([
      { at: "2026-08-19T21:14:00Z", by: "dev_a", chapter: "closed" },
      { at: "2026-08-19T21:20:00Z", by: "dev_a", undoing: true, chapter: "reopened" },
    ]);

    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText(/undone/i)).toBeTruthy();
  });

  it("quotes what was written, not just that something was", async () => {
    await shown([
      {
        at: "2026-08-05T22:05:00Z",
        by: "dev_a",
        chapter: "wrote",
        body: "the certificate took nine days to issue",
      },
    ]);

    expect(screen.getByText("the certificate took nine days to issue")).toBeTruthy();
  });

  it("names the list a task moved into", async () => {
    await shown([{ at: "2026-07-28T18:40:00Z", by: "dev_a", chapter: "filed", to: "01L" }]);

    expect(screen.getByText(/Work/)).toBeTruthy();
  });
});
