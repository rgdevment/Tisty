import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { banded } from "../archive";
import type { Task } from "../core";
import TaskList from "../ui/TaskList";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const NOW = new Date("2026-08-11T09:00:00");

const task = (id: string, title: string, day?: string): Task =>
  ({
    id,
    title,
    status: "open",
    priority: "unset",
    order: "a0",
    tags: [],
    reminders: [],
    date: day
      ? { at: `${day}T10:00:00`, tz: "America/Santiago", floating: true, has_time: false }
      : undefined,
  }) as unknown as Task;

const spoken = (task: Task): Task => ({
  ...task,
  resolved: { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1" },
});

const show = (tasks: Task[]) =>
  render(<TaskList tasks={tasks} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

describe("the day headings", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  it("separates what is late from what is due today", () => {
    show([task("1", "pagar la luz", "2026-08-04"), task("2", "llamar al dentista", "2026-08-11")]);

    expect(screen.getByText("Overdue")).toBeTruthy();
    expect(screen.getByText("Today")).toBeTruthy();
  });

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
        onSelect={() => {}}
      />,
    );

    expect(screen.queryByText("Overdue")).toBeNull();
  });
});

describe("banded", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  it("keeps the order the core gave it", () => {
    const rows = banded([
      task("1", "pagar la luz", "2026-08-04"),
      task("2", "llamar al dentista", "2026-08-11"),
      task("3", "leer el libro"),
    ]);

    expect(rows.map((row) => row.key)).toEqual(["1", "2", "3"]);
    expect(rows.map((row) => row.band)).toEqual(["Overdue", "Today", "Someday"]);
  });

  it("gathers what an agent says is done into its own band, at the end", () => {
    const rows = banded([
      spoken(task("1", "pasar biome sobre el front", "2026-08-20")),
      task("2", "llamar al dentista", "2026-08-11"),
      spoken(task("3", "revisar el icono")),
    ]);

    expect(rows.map((row) => row.key)).toEqual(["2", "1", "3"]);
    expect(rows.map((row) => row.band)).toEqual(["Today", "Agents", "Agents"]);
  });

  it("leaves what is due today or already late where the person looks for it", () => {
    const rows = banded([
      spoken(task("1", "renovar la licencia", "2026-08-04")),
      spoken(task("2", "responder a la notaria", "2026-08-11")),
      spoken(task("3", "revisar el icono")),
    ]);

    expect(rows.map((row) => row.band)).toEqual(["Overdue", "Today", "Agents"]);
  });

  it("leaves a task the person already finished where the date put it", () => {
    const shut = { ...spoken(task("1", "pasar biome", "2026-08-04")), status: "done" } as Task;

    const rows = banded([shut]);

    expect(rows).toHaveLength(1);
    expect(rows[0].band).toBe("Overdue");
  });

  it("puts every task in one band and only one", () => {
    const rows = banded([
      spoken(task("1", "pasar biome", "2026-08-04")),
      task("2", "llamar al dentista", "2026-08-11"),
      { ...spoken(task("3", "revisar el icono")), status: "done" } as Task,
    ]);

    expect(rows).toHaveLength(3);
    expect(new Set(rows.map((row) => row.key)).size).toBe(3);
  });
});

describe("the band of what an agent says is done", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  it("counts what is waiting instead of how many there are", () => {
    show([
      task("1", "llamar al dentista", "2026-08-11"),
      spoken(task("2", "pasar biome sobre el front")),
    ]);

    expect(screen.getByText("1 to confirm")).toBeTruthy();
  });

  it("does not call a band of its own making a queue of confirmations", () => {
    show([
      spoken(task("1", "renovar la licencia", "2026-08-04")),
      task("2", "llamar al dentista", "2026-08-11"),
      spoken(task("3", "pasar biome sobre el front")),
    ]);

    expect(screen.getAllByText("1 to confirm")).toHaveLength(1);
    expect(screen.getByText("Overdue").parentElement?.textContent).not.toContain("to confirm");
  });

  it("says when it was said rather than what the task carries", () => {
    show([
      task("1", "llamar al dentista", "2026-08-11"),
      spoken(task("2", "pasar biome sobre el front")),
    ]);

    const said = screen.getByTitle(/^An agent said this was done — .+/);
    expect(said.textContent).toMatch(/6/);
    expect(screen.queryByText("0/0")).toBeNull();
  });
});
