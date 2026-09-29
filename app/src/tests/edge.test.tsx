import { render } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import TaskList from "../ui/TaskList";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
}));

const made = (id: string): Task =>
  ({
    id,
    title: `una de tantas ${id}`,
    status: "open",
    priority: "unset",
    order: id,
    steps: [],
    log: [],
    volume: {},
  }) as unknown as Task;

let reached: number;

beforeEach(() => {
  reached = 0;
  class Watching {
    private seen: () => void;
    constructor(seen: (entries: { isIntersecting: boolean }[]) => void) {
      this.seen = () => seen([{ isIntersecting: true }]);
    }
    observe() {
      this.seen();
    }
    disconnect() {}
    unobserve() {}
  }
  vi.stubGlobal("IntersectionObserver", Watching);
});

const at = (extra: Partial<React.ComponentProps<typeof TaskList>> = {}) => (
  <TaskList
    tasks={[made("01A")]}
    lists={[]}
    title="Tareas"
    onSelect={vi.fn()}
    onReach={() => {
      reached += 1;
    }}
    {...extra}
  />
);

describe("the edge that asks for one more column", () => {
  it("asks when the rows it belongs to are the ones on screen", () => {
    render(at());

    expect(reached).toBe(1);
  });

  it("stays quiet on a screen whose rows were replaced by something else", () => {
    render(at({ instead: <p>otra cosa</p> }));

    expect(reached).toBe(0);
  });
});
