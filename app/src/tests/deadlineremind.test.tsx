import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Fields from "../ui/Fields";

function ahead(hours: number): string {
  const at = new Date(Date.now() + hours * 3600_000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}T${pad(at.getHours())}:00:00`;
}

function put(over: Partial<Task>) {
  const one = {
    id: "t1",
    title: "entregar el informe",
    status: "open",
    priority: 4,
    order: "a0",
    ...over,
  } as Task;
  render(<Fields task={one} lists={[]} known={[]} onPatch={vi.fn()} />);
  fireEvent.click(screen.getByText(/⏰ reminder/));
}

const spec = (at: string) => ({ at, has_time: true, tz: "Europe/Madrid" });

describe("what a reminder counts back from", () => {
  it("uses the deadline when the task has no date of its own", () => {
    put({ deadline: spec(ahead(6)) as Task["deadline"] });

    expect(screen.getByText("At the deadline")).toBeTruthy();
    expect(screen.getByText("An hour before")).toBeTruthy();
  });

  it("uses the date when there is one, deadline or not", () => {
    put({ date: spec(ahead(6)) as Task["date"], deadline: spec(ahead(30)) as Task["deadline"] });

    expect(screen.getByText("At the time")).toBeTruthy();
    expect(screen.queryByText("At the deadline")).toBeNull();
  });

  it("offers only the calendar when the task has neither", () => {
    put({});

    expect(screen.queryByText("At the time")).toBeNull();
    expect(screen.queryByText("An hour before")).toBeNull();
    expect(screen.getByText("Pick a day and time…")).toBeTruthy();
  });
});
