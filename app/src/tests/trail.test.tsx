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
    expect(screen.getByText(/moves:/i)).toBeTruthy();
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

  it("has a phrase for every kind of chapter the core can emit", async () => {
    await shown([
      { at: "2026-07-27T09:12:00Z", by: "d", chapter: "born", title: "ship it" },
      { at: "2026-07-27T09:13:00Z", by: "d", chapter: "retitled", from: "ship it", to: "ship 0.3" },
      { at: "2026-07-28T09:00:00Z", by: "d", chapter: "dated", to: day("2026-08-01") },
      { at: "2026-07-28T10:00:00Z", by: "d", chapter: "dated" },
      { at: "2026-07-29T09:00:00Z", by: "d", chapter: "bounded" },
      { at: "2026-07-30T09:00:00Z", by: "d", chapter: "placed", from: "unset", to: "do" },
      { at: "2026-07-31T09:00:00Z", by: "d", chapter: "filed" },
      { at: "2026-08-01T09:00:00Z", by: "d", chapter: "tagged", added: ["release"], gone: [] },
      { at: "2026-08-02T09:00:00Z", by: "d", chapter: "tagged", added: [], gone: ["release"] },
      {
        at: "2026-08-03T09:00:00Z",
        by: "d",
        chapter: "cadenced",
        to: { from: "due", each: { every: 1, unit: "day" } },
      },
      { at: "2026-08-04T09:00:00Z", by: "d", chapter: "cadenced" },
      { at: "2026-08-05T09:00:00Z", by: "d", chapter: "described", emptied: false },
      { at: "2026-08-06T09:00:00Z", by: "d", chapter: "described", emptied: true },
      { at: "2026-08-07T09:00:00Z", by: "d", chapter: "rewrote", body: "again" },
      { at: "2026-08-08T09:00:00Z", by: "d", chapter: "planned", text: "sign it" },
      { at: "2026-08-09T09:00:00Z", by: "d", chapter: "ticked", text: "sign it" },
      { at: "2026-08-10T09:00:00Z", by: "d", chapter: "unticked", text: "sign it" },
      {
        at: "2026-08-11T09:00:00Z",
        by: "d",
        chapter: "reworded",
        from: "sign it",
        to: "sign msix",
      },
      { at: "2026-08-12T09:00:00Z", by: "d", chapter: "unplanned", text: "sign msix" },
      { at: "2026-08-13T09:00:00Z", by: "d", chapter: "dropped" },
    ]);

    const said = screen
      .getAllByRole("listitem")
      .map((one) => one.textContent?.replace(/^\s*\S+\s+\d+:\d+\s*/, "").trim() ?? "");

    expect(said).toHaveLength(20);
    for (const one of said) {
      expect(one.length, `a chapter rendered with no words: ${said.join(" | ")}`).toBeGreaterThan(
        1,
      );
    }
  });

  it("keeps its footing when a list it names is no longer there", async () => {
    await shown([{ at: "2026-07-28T18:40:00Z", by: "d", chapter: "filed", to: "01GONE" }]);

    expect(screen.getAllByRole("listitem")).toHaveLength(1);
  });

  it("names the list a task moved into", async () => {
    await shown([{ at: "2026-07-28T18:40:00Z", by: "dev_a", chapter: "filed", to: "01L" }]);

    expect(screen.getByText(/Work/)).toBeTruthy();
  });
});
