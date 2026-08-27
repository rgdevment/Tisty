import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { List, Series } from "../core";
import Shelf from "../ui/Shelf";

const ipc = vi.hoisted(() => ({ all: [] as Series[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "icons" ? [] : ipc.all),
}));

const lists: List[] = [{ id: "01H", name: "Health", order: "a0", icon: "heart" }];

const series = (some: Partial<Series>): Series => ({
  last: "01T",
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

const shown = async (all: Series[]) => {
  ipc.all = all;
  const opened = vi.fn();
  render(<Shelf lists={lists} onOpen={opened} />);
  await screen.findByRole("list");
  return opened;
};

beforeEach(() => {
  ipc.all = [];
});

describe("the shelf of routines", () => {
  it("says nothing repeats yet instead of an empty box", async () => {
    ipc.all = [];
    render(<Shelf lists={lists} onOpen={() => {}} />);

    expect(await screen.findByText(/Nothing repeats yet/i)).toBeTruthy();
  });

  it("counts what was kept against what was owed, not against the turns that exist", async () => {
    await shown([
      series({
        turns: [{ id: "01", status: "done" }],
        kept: 26,
        owed: 30,
        skipped: 4,
        streak: 4,
      }),
    ]);

    expect(screen.getByText("26/30")).toBeTruthy();
    expect(screen.getByText(/4 with no record/i)).toBeTruthy();
  });

  it("uses the singular when only one date has no record", async () => {
    await shown([series({ kept: 1, owed: 2, skipped: 1, turns: [{ id: "01", status: "done" }] })]);

    expect(screen.getByText(/one with no record/i)).toBeTruthy();
  });

  it("keeps quiet about records it cannot measure", async () => {
    await shown([
      series({
        kept: 2,
        owed: 2,
        skipped: 3,
        measurable: false,
        turns: [{ id: "01", status: "done" }],
      }),
    ]);

    expect(screen.queryByText(/no record/i)).toBeNull();
  });

  it("marks a routine that is still running", async () => {
    await shown([series({ kept: 4, owed: 4, open: 1, turns: [{ id: "01", status: "open" }] })]);

    expect(screen.getByText(/still running/i)).toBeTruthy();
  });

  it("names the list and the tags it carries", async () => {
    await shown([series({ list: "01H", tags: ["control"], kept: 1, owed: 1 })]);

    expect(screen.getByText("@Health")).toBeTruthy();
    expect(screen.getByText("#control")).toBeTruthy();
  });

  it("opens the turn the archive can show", async () => {
    const user = userEvent.setup();
    const opened = await shown([series({ last: "01LAST", kept: 1, owed: 1 })]);

    await user.click(screen.getByRole("button"));

    expect(opened).toHaveBeenCalledWith("01LAST");
  });
});
