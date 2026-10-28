import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { banded, regrouped } from "../archive";
import type { Standing, Task } from "../core";
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

const show = (tasks: Task[], standings: Record<string, Standing> = {}) =>
  render(
    <TaskList
      tasks={tasks}
      lists={[]}
      standings={standings}
      title="Open"
      bands="day"
      onSelect={() => {}}
    />,
  );

describe("the day headings", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  it("separates what is late from what is due today", () => {
    show([task("1", "pagar la luz", "2026-08-04"), task("2", "llamar al dentista", "2026-08-11")], {
      "1": "late",
      "2": "today",
    });

    expect(screen.getByText("Late")).toBeTruthy();
    expect(screen.getByText("For today")).toBeTruthy();
  });

  it("puts every late day under one heading", () => {
    show(
      [
        task("1", "pagar la luz", "2026-08-01"),
        task("2", "responder el correo", "2026-08-04"),
        task("3", "llamar al dentista", "2026-08-11"),
      ],
      { "1": "late", "2": "late", "3": "today" },
    );

    expect(screen.getAllByText("Late")).toHaveLength(1);
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

  it("puts what claims today first by urgency, and keeps the core's order after it", () => {
    const rows = banded(
      [
        task("1", "leer el libro"),
        task("2", "llamar al dentista", "2026-08-11"),
        task("3", "pagar la luz", "2026-08-04"),
        task("4", "migración", "2026-07-06"),
        task("5", "entregar el informe"),
        task("6", "renovar el seguro"),
        task("7", "reunión", "2026-08-12"),
      ],
      { "2": "today", "3": "late", "4": "underway", "5": "overdue", "6": "due_soon", "7": "ahead" },
    );

    expect(rows.map((row) => row.key)).toEqual(["5", "6", "3", "2", "4", "1", "7"]);
    expect(rows.map((row) => row.band)).toEqual([
      "Overdue",
      "Due soon",
      "Late",
      "For today",
      "Underway",
      "Someday",
      "Tomorrow",
    ]);
  });

  it("gathers what an agent says is done into its own band, at the end", () => {
    const rows = banded(
      [
        spoken(task("1", "pasar biome sobre el front", "2026-08-20")),
        task("2", "llamar al dentista", "2026-08-11"),
        spoken(task("3", "revisar el icono")),
      ],
      { "1": "ahead", "2": "today" },
    );

    expect(rows.map((row) => row.key)).toEqual(["2", "1", "3"]);
    expect(rows.map((row) => row.band)).toEqual(["For today", "To confirm", "To confirm"]);
  });

  it("leaves what is due today or already late where the person looks for it", () => {
    const rows = banded(
      [
        spoken(task("1", "renovar la licencia", "2026-08-04")),
        spoken(task("2", "responder a la notaria", "2026-08-11")),
        spoken(task("3", "revisar el icono")),
      ],
      { "1": "late", "2": "today" },
    );

    expect(rows.map((row) => row.band)).toEqual(["Late", "For today", "To confirm"]);
  });

  it("leaves a task the person already finished where the date put it", () => {
    const shut = { ...spoken(task("1", "pasar biome", "2026-08-04")), status: "done" } as Task;

    const rows = banded([shut]);

    expect(rows).toHaveLength(1);
    expect(rows[0].band).toBe("Tue, Aug 4");
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

  // The band is named for what it asks of the person, not for who filed what is in it: every
  // task an agent filed is signed by name already, so «Agents» would say nothing.
  it("is named for what waits on the person", () => {
    show([
      task("1", "llamar al dentista", "2026-08-11"),
      spoken(task("2", "pasar biome sobre el front")),
    ]);

    const band = screen.getByText("To confirm");
    expect(band.parentElement?.textContent).toContain("1");
    expect(screen.queryByText(/^Agents$/)).toBeNull();
  });

  it("does not call a band of its own making a queue of confirmations", () => {
    show(
      [
        spoken(task("1", "renovar la licencia", "2026-08-04")),
        task("2", "llamar al dentista", "2026-08-11"),
        spoken(task("3", "pasar biome sobre el front")),
      ],
      { "1": "late", "2": "today" },
    );

    expect(screen.getAllByText("To confirm")).toHaveLength(1);
    expect(screen.getByText("Late").parentElement?.textContent).not.toContain("confirm");
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

describe("today, as A3 sees it", () => {
  beforeEach(() => vi.setSystemTime(new Date("2026-10-27T09:00:00")));
  afterEach(() => vi.useRealTimers());

  const spec = (day: string) => ({
    at: `${day}T10:00:00`,
    tz: "America/Santiago",
    floating: true,
    has_time: false,
  });
  const a3 = {
    ...task("A3", "A3 · Migración de Sodimac Uruguay", "2026-10-06"),
    deadline: spec("2026-12-04"),
  } as Task;
  const ranOut = { ...task("R", "pagar la patente"), deadline: spec("2026-10-20") } as Task;
  const planned = task("P", "llamar al banco", "2026-10-06");
  const today = { A3: "underway", R: "overdue", P: "late" } as const;

  it("keeps A3 underway, folded, and out of what is overdue", () => {
    show([ranOut, planned, a3], today);

    const underway = screen.getByRole("button", { name: /Underway/ });
    expect(underway.getAttribute("aria-expanded")).toBe("false");
    expect(underway.textContent).toContain("1");
    expect(screen.queryByText("A3 · Migración de Sodimac Uruguay")).toBeNull();
    expect(screen.getByText("Overdue").parentElement?.textContent).not.toContain("A3");
  });

  it("says when A3 is due, without red, once it is unfolded", () => {
    show([ranOut, planned, a3], today);

    fireEvent.click(screen.getByRole("button", { name: /Underway/ }));

    expect(screen.getByText("⚑ due in 38 days").className).not.toContain("text-urgent");
    expect(screen.getByText(/⚑ Tue, Oct 20/).className).toContain("text-urgent");
    expect(screen.getByText("since Tue, Oct 6").className).toContain("text-hue-amber");
  });
});

describe("what the review asked of the day headings", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  it("keeps an overdue task an agent spoke for in its list, not among confirmations", () => {
    const work = { id: "L", name: "Trabajo", order: "a0" };
    const ranOut = { ...spoken(task("1", "renovar la licencia")), list: "L" } as Task;

    const rows = regrouped([ranOut], "list", [work], { "1": "overdue" });

    expect(rows.map((row) => row.band)).toEqual(["Trabajo"]);
  });

  it("folds only the time band, never a list that happens to share its name", () => {
    const named = [
      { id: "L", name: "Underway", order: "a0" },
      { id: "M", name: "Casa", order: "a1" },
    ];
    render(
      <TaskList
        tasks={[
          { ...task("1", "migrar el lector"), list: "L" } as Task,
          { ...task("2", "regar"), list: "M" } as Task,
        ]}
        lists={named}
        axis="list"
        title="Open"
        onSelect={() => {}}
      />,
    );

    const band = screen.getByRole("button", { name: /Underway/ });
    expect(band.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("migrar el lector")).toBeTruthy();
  });
});
