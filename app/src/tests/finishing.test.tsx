import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const task = (extra: Partial<Task> = {}): Task =>
  ({
    id: "01T",
    title: "write the report",
    status: "open",
    priority: "do",
    order: "a0",
    steps: [],
    log: [],
    tags: [],
    ...extra,
  }) as unknown as Task;

const open = (one: Task, expanded = false) => {
  const done = vi.fn();
  render(
    <Detail
      task={one}
      lists={[]}
      known={[]}
      expanded={expanded}
      onExpand={() => {}}
      onCollapse={() => {}}
      onPatch={() => {}}
      onStep={() => {}}
      onMark={() => {}}
      onDropStep={() => {}}
      onLog={() => {}}
      onComplete={done}
      onDiscard={() => {}}
      onReopen={() => {}}
      onClose={() => {}}
    />,
  );
  return done;
};

describe("finishing a task from the panel", () => {
  it("completes it from the column", async () => {
    const done = open(task());

    await userEvent.click(screen.getByRole("button", { name: "✓ Done" }));

    expect(done).toHaveBeenCalled();
  });

  it("completes it full-screen too", async () => {
    const done = open(task(), true);

    await userEvent.click(screen.getByRole("button", { name: "✓ Done" }));

    expect(done).toHaveBeenCalled();
  });

  it("keeps the way out of a task it will not do", () => {
    open(task());

    expect(screen.getByRole("button", { name: /not doing it/i })).toBeTruthy();
  });

  it("offers to reopen instead once the task is settled", () => {
    open(task({ status: "done" }));

    expect(screen.queryByRole("button", { name: "✓ Done" })).toBeNull();
    expect(screen.getByRole("button", { name: /reopen/i })).toBeTruthy();
  });
});
