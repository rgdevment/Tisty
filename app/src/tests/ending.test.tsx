import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Detail from "../ui/Detail";
import type { Task } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const task = (repeat?: Task["repeat"]): Task =>
  ({
    id: "01T",
    title: "sacar la basura",
    status: "open",
    priority: 4,
    order: "a0",
    steps: [],
    log: [],
    tags: [],
    repeat,
  }) as unknown as Task;

const open = (one: Task) =>
  render(
    <Detail
      task={one}
      lists={[]}
      known={[]}
      expanded={false}
      onExpand={() => {}}
      onCollapse={() => {}}
      onPatch={() => {}}
      onStep={() => {}}
      onMark={() => {}}
      onDropStep={() => {}}
      onLog={() => {}}
      onDiscard={() => {}}
      onReopen={() => {}}
      onClose={() => {}}
    />,
  );

describe("ending a repeat", () => {
  it("says what it does when the task repeats", () => {
    open(task({ from: "due", each: { every: 1, unit: "week" } }));

    expect(screen.getByRole("button", { name: /end the repeat/i })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /not doing it/i })).toBeNull();
  });

  it("keeps its ordinary name on an ordinary task", () => {
    open(task());

    expect(screen.getByRole("button", { name: /not doing it/i })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /end the repeat/i })).toBeNull();
  });
});
