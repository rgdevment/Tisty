import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import TaskList from "../ui/TaskList";
import { banded } from "../archive";
import type { Task } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const NOW = new Date("2026-08-11T09:00:00");

const task = (id: string, title: string, day?: string): Task =>
  ({
    id,
    title,
    status: "open",
    priority: 4,
    order: "a0",
    tags: [],
    reminders: [],
    date: day
      ? { at: `${day}T10:00:00`, tz: "America/Santiago", floating: true, has_time: false }
      : undefined,
  }) as unknown as Task;

const show = (tasks: Task[]) =>
  render(
    <TaskList tasks={tasks} lists={[]} title="Open" bands="day" centred onSelect={() => {}} />,
  );

describe("the day headings", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  /// «Today» used to be overdue + today + everything undated as one flat wall.
  it("separates what is late from what is due today", () => {
    show([
      task("1", "pagar la luz", "2026-08-04"),
      task("2", "llamar al dentista", "2026-08-11"),
    ]);

    expect(screen.getByText("Overdue")).toBeTruthy();
    expect(screen.getByText("Today")).toBeTruthy();
  });

  /// A heading per late day would be the same wall with more lines in it.
  it("puts every late day under one heading", () => {
    show([
      task("1", "pagar la luz", "2026-08-01"),
      task("2", "responder el correo", "2026-08-04"),
      task("3", "llamar al dentista", "2026-08-11"),
    ]);

    expect(screen.getAllByText("Overdue")).toHaveLength(1);
  });

  it("names tomorrow and dates the days after it", () => {
    show([task("1", "reunión", "2026-08-12"), task("2", "revisión", "2026-08-20")]);

    expect(screen.getByText("Tomorrow")).toBeTruthy();
    expect(screen.queryByText("Overdue")).toBeNull();
  });

  it("gathers what has no date at all", () => {
    show([task("1", "llamar al dentista", "2026-08-11"), task("2", "leer el libro")]);

    expect(screen.getByText("Someday")).toBeTruthy();
  });

  /// One heading over the whole list says nothing and costs a line.
  it("stays out of the way when everything sits in one band", () => {
    show([task("1", "leer el libro"), task("2", "ordenar el cajón")]);

    expect(screen.queryByText("Someday")).toBeNull();
  });

  it("says nothing on a list that was never banded", () => {
    render(
      <TaskList
        tasks={[task("1", "pagar la luz", "2026-08-04")]}
        lists={[]}
        title="Search"
        centred
        onSelect={() => {}}
      />,
    );

    expect(screen.queryByText("Overdue")).toBeNull();
  });
});

describe("banded", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  /// The core sorts dated before undated, so labelling in place is enough —
  /// and reordering here would fight the drag-and-drop indices.
  it("keeps the order the core gave it", () => {
    const rows = banded([
      task("1", "pagar la luz", "2026-08-04"),
      task("2", "llamar al dentista", "2026-08-11"),
      task("3", "leer el libro"),
    ]);

    expect(rows.map((row) => row.key)).toEqual(["1", "2", "3"]);
    expect(rows.map((row) => row.band)).toEqual(["Overdue", "Today", "Someday"]);
  });
});
