import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import TaskList from "../ui/TaskList";
import type { Task } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const done = (id: string, title: string, at: string): Task =>
  ({
    id,
    title,
    status: "done",
    order: "a0",
    steps: [],
    journal: [],
    tags: [],
    created: at,
    completed_at: at,
  }) as unknown as Task;

const archive = (tasks: Task[]) =>
  render(
    <TaskList tasks={tasks} lists={[]} title="Archive" bands="month" onSelect={() => {}} />,
  );

describe("the archive folds repetitions", () => {
  it("shows one line with a count instead of three rows", async () => {
    archive([
      done("1", "sacar la basura", "2026-08-25T09:00:00Z"),
      done("2", "sacar la basura", "2026-08-18T09:00:00Z"),
      done("3", "sacar la basura", "2026-08-11T09:00:00Z"),
    ]);

    expect(screen.getAllByText("sacar la basura")).toHaveLength(1);
    expect(screen.getByText(/3 times/i)).toBeTruthy();
  });

  it("opens to show every one of them", async () => {
    archive([
      done("1", "sacar la basura", "2026-08-25T09:00:00Z"),
      done("2", "sacar la basura", "2026-08-18T09:00:00Z"),
    ]);

    await userEvent.click(screen.getByRole("button", { expanded: false }));

    expect(screen.getAllByText("sacar la basura").length).toBeGreaterThan(1);
  });

  /// A one-off must not grow a counter it does not need.
  it("leaves a single closing as a plain row", () => {
    archive([done("1", "comprar pan", "2026-08-25T09:00:00Z")]);

    expect(screen.getByText("comprar pan")).toBeTruthy();
    expect(screen.queryByText(/\d+ times/i)).toBeNull();
  });

  /// Without the month heading the fold would read as one long streak.
  it("keeps two months apart", () => {
    archive([
      done("1", "sacar la basura", "2026-09-01T09:00:00Z"),
      done("2", "sacar la basura", "2026-08-25T09:00:00Z"),
    ]);

    expect(screen.getAllByText("sacar la basura")).toHaveLength(2);
    expect(screen.queryByText(/\d+ times/i)).toBeNull();
  });
});
