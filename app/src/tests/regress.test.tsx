import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import TaskList from "../ui/TaskList";
import { nothing } from "../views";
import type { Task } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (id: string, title: string, day?: string, done = false): Task =>
  ({
    id,
    title,
    status: done ? "done" : "open",
    priority: 4,
    order: "a0",
    tags: [],
    reminders: [],
    completed_at: done ? "2026-08-05T09:00:00Z" : undefined,
    date: day
      ? { at: `${day}T10:00:00`, tz: "America/Santiago", floating: true, has_time: false }
      : undefined,
  }) as unknown as Task;

const stops = () =>
  screen.getAllByRole("listitem").filter((row) => row.getAttribute("tabindex") === "0");

describe("the list never loses its way in", () => {
  /// Completing the focused task took it out of the list, and every remaining
  /// row was left at tabindex -1: the list became unreachable by keyboard.
  it("keeps a tab stop when the focused task leaves the list", () => {
    const three = [task("1", "una"), task("2", "dos"), task("3", "tres")];
    const shown = render(
      <TaskList tasks={three} lists={[]} title="Open" onSelect={() => {}} />,
    );

    screen.getAllByRole("listitem")[2].focus();
    shown.rerender(
      <TaskList
        tasks={[task("1", "una"), task("2", "dos")]}
        lists={[]}
        title="Open"
        onSelect={() => {}}
      />,
    );

    expect(stops().length).toBe(1);
  });

  /// The archive folds repeats, so the first task can be inside a closed group
  /// and never rendered — the fallback pointed at a row that is not there.
  it("keeps a tab stop when the first task sits inside a folded group", () => {
    render(
      <TaskList
        tasks={[
          task("1", "regar", undefined, true),
          task("2", "regar", undefined, true),
          task("3", "otra", undefined, true),
        ]}
        lists={[]}
        title="Archive"
        bands="month"
        onSelect={() => {}}
      />,
    );

    expect(stops().length).toBe(1);
  });
});

describe("day bands", () => {
  /// Archived tasks are appended after the open ones, so a band that already
  /// appeared came back a second time.
  it("never repeats a heading when open and closed tasks are mixed", () => {
    render(
      <TaskList
        tasks={[
          task("1", "abierta vencida", "2026-08-04"),
          task("2", "abierta sin fecha"),
          task("3", "cerrada vencida", "2026-08-03", true),
          task("4", "cerrada sin fecha", undefined, true),
        ]}
        lists={[]}
        title="Tags"
        bands="day"
        onSelect={() => {}}
      />,
    );

    for (const band of ["Overdue", "Someday"]) {
      expect(screen.queryAllByText(band).length).toBeLessThanOrEqual(1);
    }
  });
});

describe("what an empty tag view says", () => {
  /// Choosing tags sets `named` AND `tags`, and the order of the checks made
  /// the screen say «no tags yet» with the chosen tags drawn right above it.
  it("talks about the filter, not about having no tags", () => {
    expect(nothing({ named: "tags", tags: ["work", "home"] }, false)).not.toMatch(/no tags/i);
  });

  it("still says there are none when there really are none", () => {
    expect(nothing({ named: "tags" }, false)).toMatch(/no tags/i);
  });

  /// The archive hides the scope chips it tells you to widen.
  it("does not send you to a control the archive hides", () => {
    expect(nothing({ named: "archive" }, true)).not.toMatch(/scope/i);
    expect(nothing({ named: "search" }, true)).toMatch(/scope/i);
  });
});
