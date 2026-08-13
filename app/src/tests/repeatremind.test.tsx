import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Fields from "../ui/Fields";

function task(over: Partial<Task>): Task {
  return {
    id: "t1",
    title: "tomar la paroxetina",
    status: "open",
    priority: 4,
    order: "a0",
    reminders: [{ at: "2026-08-13T09:00:00", has_time: true, tz: "Europe/Madrid" }],
    ...over,
  } as Task;
}

function put(one: Task) {
  render(<Fields task={one} lists={[]} known={[]} onPatch={vi.fn()} />);
}

describe("a reminder on a repeating task", () => {
  it("says the cadence, because that is what it does", () => {
    put(task({ repeat: { from: "done", each: { every: 1, unit: "day" } } }));

    expect(screen.getByText(/⏰↻ every day/)).toBeTruthy();
  });

  it("keeps the hour, which is the part that matters for a medicine", () => {
    put(task({ repeat: { from: "done", each: { every: 1, unit: "day" } } }));

    expect(screen.getByText(/9:00/)).toBeTruthy();
  });

  it("still names the day when the task does not repeat", () => {
    put(task({}));

    expect(screen.queryByText(/⏰↻/)).toBeNull();
  });
});
