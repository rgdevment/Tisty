import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Repeat, Task } from "../core";
import Fields from "../ui/Fields";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (repeat?: Repeat, dated = true): Task =>
  ({
    id: "01T",
    title: "sacar la basura",
    status: "open",
    priority: "unset",
    order: "a0",
    tags: [],
    reminders: [],
    date: dated
      ? { at: "2026-08-11T10:00:00", tz: "America/Santiago", floating: true, has_time: true }
      : undefined,
    repeat,
  }) as unknown as Task;

const show = (one: Task, onPatch = () => {}) =>
  render(<Fields task={one} lists={[]} known={[]} onPatch={onPatch} />);

const weekly: Repeat = { from: "due", each: { every: 1, unit: "week" } };

describe("the repeat field in the detail", () => {
  it("shows the cadence a task already carries", () => {
    show(task(weekly));

    expect(screen.getByRole("button", { name: /every week/i })).toBeTruthy();
  });

  it("offers to set one on a task that has none", async () => {
    const patched = vi.fn();
    show(task(undefined), patched);

    await userEvent.click(screen.getByRole("button", { name: /repeat/i }));
    await userEvent.click(screen.getByText("every month"));

    expect(patched).toHaveBeenCalledWith({
      repeat: { from: "due", each: { every: 1, unit: "month" } },
    });
  });

  it("counts from completion when the task has no date", async () => {
    const patched = vi.fn();
    show(task(undefined, false), patched);

    await userEvent.click(screen.getByRole("button", { name: /repeat/i }));
    await userEvent.click(screen.getByText("every day"));

    expect(patched).toHaveBeenCalledWith({
      repeat: { from: "done", each: { every: 1, unit: "day" } },
    });
  });

  it("keeps how it counts when only the cadence changes", async () => {
    const patched = vi.fn();
    show(task({ from: "done", each: { every: 3, unit: "day" } }), patched);

    await userEvent.click(screen.getByRole("button", { name: /every 3 days/i }));
    await userEvent.click(screen.getByText("every week"));

    expect(patched).toHaveBeenCalledWith({
      repeat: { from: "done", each: { every: 1, unit: "week" } },
    });
  });

  it("can end the repeat, with the name the author asked for", async () => {
    const patched = vi.fn();
    show(task(weekly), patched);

    await userEvent.click(screen.getByRole("button", { name: /every week/i }));
    await userEvent.click(screen.getByText("End the repeat"));

    expect(patched).toHaveBeenCalledWith({ noRepeat: true });
  });

  it("does not offer to end one that was never set", async () => {
    show(task(undefined));

    await userEvent.click(screen.getByRole("button", { name: /repeat/i }));

    expect(screen.queryByText("End the repeat")).toBeNull();
  });
});
