import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Coming, Habit, Series, Task } from "../core";
import { t } from "../locales";
import Ahead from "../ui/Ahead";

const dayAway = (away: number): number => {
  const at = new Date();
  at.setDate(at.getDate() + away);
  return at.getDate();
};

const stamp = (away: number): string => {
  const at = new Date();
  at.setDate(at.getDate() + away);
  const month = String(at.getMonth() + 1).padStart(2, "0");
  const day = String(at.getDate()).padStart(2, "0");
  return `${at.getFullYear()}-${month}-${day}`;
};

const spec = (away: number, clock: string) => ({
  at: `${stamp(away)}T${clock}`,
  tz: "America/Santiago",
  floating: true,
  has_time: clock !== "",
});

const made = (id: string, title: string, away: number, clock = "", due = false): Task =>
  ({
    id,
    title,
    status: "open",
    priority: "unset",
    order: "a0",
    steps: [],
    log: [],
    volume: {},
    ...(due ? { deadline: spec(away, clock) } : { date: spec(away, clock) }),
  }) as unknown as Task;

const coming = (id: string, title: string, away: number, clock = "", due = false): Coming => ({
  task: made(id, title, away, clock, due),
  on: stamp(away),
  due,
});

const routine = (id: string, title: string, clock: string): Habit => ({
  task: {
    ...made(id, title, 0, clock),
    repeat: { from: "due", each: { every: 1, unit: "day" } },
  } as unknown as Task,
});

const told = (streak: number, marks: string[]): Series =>
  ({
    last: "01B",
    title: "Tomar píldoras",
    turns: marks.map((status, at) => ({
      id: `t${at}`,
      status,
      due: spec(-marks.length + at, "10:00:00"),
    })),
    kept: marks.filter((one) => one === "done").length,
    owed: marks.length,
    dropped: 0,
    open: 0,
    skipped: 0,
    streak,
    longest: streak,
    measurable: true,
  }) as unknown as Series;

describe("the week ahead", () => {
  it("puts a timed thing under its day with its hour", () => {
    render(
      <Ahead
        coming={[coming("01A", "Médico", 2, "16:00:00")]}
        routines={[]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText("Médico")).toBeTruthy();
    expect(screen.getByText(/16/)).toBeTruthy();
  });

  it("keeps an empty day in sight without a word on it", () => {
    render(
      <Ahead
        coming={[coming("01A", "Médico", 1, "16:00:00")]}
        routines={[]}
        days={3}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("button")).toHaveLength(1);
    expect(screen.getByText(String(dayAway(2)))).toBeTruthy();
    expect(screen.getByText(String(dayAway(3)))).toBeTruthy();
  });

  it("says so when the whole window is empty", () => {
    render(<Ahead coming={[]} routines={[]} days={7} onOpen={vi.fn()} />);

    expect(screen.getByText(t("aheadNothing"))).toBeTruthy();
    expect(screen.queryAllByRole("button")).toHaveLength(0);
  });

  it("leaves out what falls past the window it was given", () => {
    render(
      <Ahead
        coming={[coming("01A", "Vuelo", 30, "07:15:00")]}
        routines={[]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.queryByText("Vuelo")).toBeNull();
  });

  it("opens what you click", async () => {
    const opened = vi.fn();
    render(<Ahead coming={[coming("01A", "Kermés", 2)]} routines={[]} days={7} onOpen={opened} />);

    await userEvent.click(screen.getByText("Kermés"));

    expect(opened).toHaveBeenCalledWith("01A");
  });

  it("sorts the timed ones first, by the clock", () => {
    render(
      <Ahead
        coming={[
          coming("01A", "Informe", 2),
          coming("01B", "Tarde", 2, "18:00:00"),
          coming("01C", "Mañana", 2, "09:00:00"),
        ]}
        routines={[]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    const shown = screen.getAllByRole("button").map((one) => one.textContent ?? "");

    expect(shown[0]).toContain("Mañana");
    expect(shown[1]).toContain("Tarde");
    expect(shown[2]).toContain("Informe");
  });

  it("marks the day that carries three things", () => {
    render(
      <Ahead
        coming={[
          coming("01A", "Kermés", 1, "11:00:00"),
          coming("01B", "Médico", 1, "16:00:00"),
          coming("01C", "Informe", 1),
        ]}
        routines={[]}
        days={1}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText(String(dayAway(1))).className).toContain("hue-amber");
  });

  it("leaves a lighter day alone", () => {
    render(
      <Ahead
        coming={[coming("01A", "Kermés", 1, "11:00:00"), coming("01B", "Informe", 1)]}
        routines={[]}
        days={1}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText(String(dayAway(1))).className).not.toContain("hue-amber");
  });

  it("shows what falls due, not only what is meant to be worked on", () => {
    render(
      <Ahead
        coming={[coming("01A", "Entregar informe", 2, "", true)]}
        routines={[]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    const shown = screen.getByText("Entregar informe").closest("button");

    expect(shown).toBeTruthy();
    expect(shown?.className).toContain("hue-amber");
  });

  it("names a routine once, above the days, and crowds none of them", () => {
    render(
      <Ahead
        coming={[coming("01A", "Kermés", 2, "11:00:00")]}
        routines={[routine("01B", "Tomar píldoras", "10:00:00")]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getAllByText("Tomar píldoras")).toHaveLength(1);
    expect(screen.getByText("Kermés")).toBeTruthy();
  });

  it("opens the routine it names", async () => {
    const opened = vi.fn();
    render(
      <Ahead
        coming={[]}
        routines={[routine("01B", "Tomar píldoras", "10:00:00")]}
        days={7}
        onOpen={opened}
      />,
    );

    await userEvent.click(screen.getByText("Tomar píldoras"));

    expect(opened).toHaveBeenCalledWith("01B");
  });

  it("does not call the week empty when a routine runs through it", () => {
    render(
      <Ahead
        coming={[]}
        routines={[routine("01B", "Tomar píldoras", "10:00:00")]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.queryByText(t("aheadNothing"))).toBeNull();
  });

  it("carries how many turns it has kept in a row", () => {
    const one = routine("01B", "Tomar píldoras", "10:00:00");
    render(
      <Ahead
        coming={[]}
        routines={[{ ...one, series: told(4, ["done", "open", "done", "done"]) }]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText("4")).toBeTruthy();
  });

  it("names the day of a routine that falls only once this week", () => {
    const one = routine("01B", "Revisión dental", "09:00:00");
    render(<Ahead coming={[]} routines={[{ ...one, on: stamp(2) }]} days={7} onOpen={vi.fn()} />);

    const line = screen.getByText("Revisión dental").closest("button");

    expect(line?.textContent).toContain(String(dayAway(2)));
  });

  it("names no day for a routine that falls every day", () => {
    render(
      <Ahead
        coming={[]}
        routines={[routine("01B", "Tomar píldoras", "10:00:00")]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText("Tomar píldoras").closest("button")?.textContent).not.toMatch(/\d/);
  });

  it("says nothing of a streak for a routine with no series behind it", () => {
    render(
      <Ahead
        coming={[]}
        routines={[routine("01B", "Tomar píldoras", "10:00:00")]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText("Tomar píldoras").closest("button")?.textContent).not.toMatch(/\d/);
  });
});
