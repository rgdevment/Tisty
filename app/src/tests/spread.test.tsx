import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { Task, Volume } from "../core";
import { t } from "../locales";
import Spread, { heftOf } from "../ui/Spread";

beforeAll(() => {
  if (!Element.prototype.setPointerCapture) {
    Element.prototype.setPointerCapture = vi.fn();
    Element.prototype.releasePointerCapture = vi.fn();
  }
});

const iso = (at: Date): string =>
  `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, "0")}-${String(at.getDate()).padStart(2, "0")}`;

const weekDay = (n: number): string => {
  const now = new Date();
  const first = new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate() - ((now.getDay() + 6) % 7),
  );
  return iso(new Date(first.getFullYear(), first.getMonth(), first.getDate() + n));
};

const made = (id: string, title: string, on?: string, clock = ""): Task =>
  ({
    id,
    title,
    status: "open",
    priority: "unset",
    order: "a0",
    steps: [],
    log: [],
    volume: {},
    ...(on === undefined
      ? {}
      : {
          date: {
            at: `${on}T${clock || "00:00:00"}`,
            tz: "America/Santiago",
            floating: true,
            has_time: clock !== "",
          },
        }),
  }) as unknown as Task;

const routine = (id: string, title: string, on: string): Task =>
  ({
    ...made(id, title, on),
    repeat: { from: "due", each: { every: 1, unit: "day" } },
  }) as unknown as Task;

const dragged = (from: HTMLElement, onto: Element | null) => {
  const was = document.elementFromPoint;
  document.elementFromPoint = () => onto as Element;
  fireEvent.pointerDown(from, { button: 0, clientX: 0, clientY: 0 });
  fireEvent.pointerMove(from, { clientX: 80, clientY: 80 });
  fireEvent.pointerUp(from, { clientX: 80, clientY: 80 });
  document.elementFromPoint = was;
};

const dayAt = (n: number) => document.querySelector(`[data-day="${weekDay(n)}"]`);

describe("spreading the week", () => {
  it("lays out the whole week, monday through sunday", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(document.querySelectorAll("[data-day]")).toHaveLength(7);
    expect(dayAt(0)).toBeTruthy();
    expect(dayAt(6)).toBeTruthy();
  });

  it("puts each task under the day it carries", () => {
    render(
      <Spread
        tasks={[made("01A", "Kermés", weekDay(4), "11:00:00")]}
        onPlace={vi.fn()}
        onOpen={vi.fn()}
      />,
    );

    expect(dayAt(4)?.textContent).toContain("Kermés");
  });

  it("calls a day with nothing on it free", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(dayAt(3)?.textContent).toContain(t("spreadFree"));
  });

  it("keeps what has no day at all in the tray, with its heft", () => {
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    const card = screen.getByText("Sin fecha").closest("button");

    expect(card?.textContent).toContain(t("spreadSmall"));
    expect(dayAt(0)?.textContent).not.toContain("Sin fecha");
  });

  it("leaves routines out of it, since a cadence deals their day", () => {
    render(
      <Spread
        tasks={[routine("01A", "Tomar píldoras", weekDay(2))]}
        onPlace={vi.fn()}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.queryByText("Tomar píldoras")).toBeNull();
  });

  it("deals a task a day when you drop it on one", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    dragged(screen.getByText("Sin fecha"), dayAt(3));

    expect(placed).toHaveBeenCalledWith("01A", weekDay(3));
  });

  it("says nothing when the day it lands on is the one it already had", () => {
    const placed = vi.fn();
    render(
      <Spread tasks={[made("01A", "Kermés", weekDay(4))]} onPlace={placed} onOpen={vi.fn()} />,
    );

    dragged(screen.getByText("Kermés"), dayAt(4));

    expect(placed).not.toHaveBeenCalled();
  });

  it("opens what you press without dragging", () => {
    const opened = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={vi.fn()} onOpen={opened} />);

    const card = screen.getByText("Sin fecha");
    fireEvent.pointerDown(card, { button: 0, clientX: 0, clientY: 0 });
    fireEvent.pointerUp(card, { clientX: 0, clientY: 0 });

    expect(opened).toHaveBeenCalledWith(expect.objectContaining({ id: "01A" }));
  });

  it("asks when the day you dropped it on is turning heavy, and offers a free one", () => {
    const placed = vi.fn();
    render(
      <Spread
        tasks={[
          made("01A", "Uno", weekDay(4)),
          made("01B", "Dos", weekDay(4)),
          made("01C", "Sin fecha"),
        ]}
        onPlace={placed}
        onOpen={vi.fn()}
      />,
    );

    dragged(screen.getByText("Sin fecha"), dayAt(4));

    expect(placed).toHaveBeenCalledWith("01C", weekDay(4));
    expect(screen.getByText(t("spreadLeaveIt"))).toBeTruthy();
  });

  it("moves it along when you take the free day it offered", () => {
    const placed = vi.fn();
    render(
      <Spread
        tasks={[
          made("01A", "Uno", weekDay(4)),
          made("01B", "Dos", weekDay(4)),
          made("01C", "Sin fecha"),
        ]}
        onPlace={placed}
        onOpen={vi.fn()}
      />,
    );

    dragged(screen.getByText("Sin fecha"), dayAt(4));
    fireEvent.click(
      screen.getByRole("button", { name: new RegExp(t("spreadMoveIt").split("{")[0].trim()) }),
    );

    expect(placed).toHaveBeenLastCalledWith("01C", weekDay(5));
  });

  it("stays quiet when the day it lands on has room", () => {
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    dragged(screen.getByText("Sin fecha"), dayAt(3));

    expect(screen.queryByText(t("spreadLeaveIt"))).toBeNull();
  });

  it("does nothing when a task is let go outside every day", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    dragged(screen.getByText("Sin fecha"), document.body);

    expect(placed).not.toHaveBeenCalled();
  });

  it("takes the day away when a dated task is dropped back on the tray", () => {
    const placed = vi.fn();
    render(
      <Spread tasks={[made("01A", "Kermés", weekDay(4))]} onPlace={placed} onOpen={vi.fn()} />,
    );

    dragged(
      dayAt(4)?.querySelector("button") as HTMLElement,
      document.querySelector("[data-tray]"),
    );

    expect(placed).toHaveBeenCalledWith("01A", null);
  });

  it("says nothing when what comes back to the tray never had a day", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    dragged(screen.getByText("Sin fecha"), document.querySelector("[data-tray]"));

    expect(placed).not.toHaveBeenCalled();
  });

  it("moves a task that already had a day onto a different one", () => {
    const placed = vi.fn();
    render(
      <Spread tasks={[made("01A", "Kermés", weekDay(1))]} onPlace={placed} onOpen={vi.fn()} />,
    );

    dragged(screen.getByText("Kermés"), dayAt(5));

    expect(placed).toHaveBeenCalledWith("01A", weekDay(5));
  });

  it("offers no day to move to when the crowded day is the last of the week", () => {
    const placed = vi.fn();
    render(
      <Spread
        tasks={[
          made("01A", "Uno", weekDay(6)),
          made("01B", "Dos", weekDay(6)),
          made("01C", "Sin fecha"),
        ]}
        onPlace={placed}
        onOpen={vi.fn()}
      />,
    );

    dragged(screen.getByText("Sin fecha"), dayAt(6));

    expect(placed).toHaveBeenCalledWith("01C", weekDay(6));
    expect(screen.getByText(t("spreadLeaveIt"))).toBeTruthy();
    expect(
      screen.queryByRole("button", {
        name: new RegExp(t("spreadMoveIt").split("{")[0].trim()),
      }),
    ).toBeNull();
  });
});

describe("a week that crosses into another month and year", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 0, 1, 9, 0, 0));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("still lays out seven days and lands a drop on the right date", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    expect(document.querySelectorAll("[data-day]")).toHaveLength(7);
    expect(dayAt(0)).toBeTruthy();
    expect(document.querySelector('[data-day="2025-12-29"]')).toBeTruthy();
    expect(document.querySelector('[data-day="2026-01-04"]')).toBeTruthy();

    dragged(screen.getByText("Sin fecha"), document.querySelector('[data-day="2025-12-31"]'));

    expect(placed).toHaveBeenCalledWith("01A", "2025-12-31");
  });
});

describe("walking the weeks", () => {
  const shown = (): string[] =>
    [...document.querySelectorAll("[data-day]")].map(
      (one) => one.getAttribute("data-day") as string,
    );

  it("starts on the week you are living", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(shown()[0]).toBe(weekDay(0));
  });

  it("goes on a week, and comes back one", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: t("spreadOn") }));
    expect(shown()[0]).toBe(weekDay(7));

    fireEvent.click(screen.getByRole("button", { name: t("spreadBack") }));
    expect(shown()[0]).toBe(weekDay(0));
  });

  it("offers the way home only once you have left", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(screen.queryByText(t("spreadNow"))).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: t("spreadBack") }));
    fireEvent.click(screen.getByRole("button", { name: t("spreadBack") }));
    expect(shown()[0]).toBe(weekDay(-14));

    fireEvent.click(screen.getByText(t("spreadNow")));
    expect(shown()[0]).toBe(weekDay(0));
  });

  it("deals a day on the week you walked to, not the one you came from", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: t("spreadOn") }));
    dragged(screen.getByText("Sin fecha"), document.querySelector(`[data-day="${weekDay(10)}"]`));

    expect(placed).toHaveBeenCalledWith("01A", weekDay(10));
  });
});

describe("a monday read from a zone behind UTC", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 8, 7, 9, 0, 0));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("lays out the week that monday opens, not the one it closed", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(document.querySelector('[data-day="2026-09-07"]')).toBeTruthy();
    expect(document.querySelector('[data-day="2026-09-13"]')).toBeTruthy();
    expect(document.querySelector('[data-day="2026-08-31"]')).toBeNull();
  });
});

describe("heftOf mirrors the weight the backend computes", () => {
  const weigh = (volume: Volume): number => {
    const plan = (volume.steps ?? 0) <= 2 ? 0 : (volume.steps ?? 0) <= 7 ? 1 : 2;
    const refs = (volume.refs ?? 0) === 0 ? 0 : (volume.refs ?? 0) <= 2 ? 1 : 2;
    return (volume.prose ?? 0) + plan + refs;
  };

  it("matches Volume::weight() from crates/tisty-core/src/model/task.rs for a spread of volumes", () => {
    const cases: Volume[] = [
      {},
      { steps: 2 },
      { steps: 3 },
      { steps: 7 },
      { steps: 8 },
      { steps: 40 },
      { refs: 1 },
      { refs: 2 },
      { refs: 3 },
      { prose: 5 },
      { prose: 8, steps: 8, refs: 3 },
      { steps: 0, refs: 0, prose: 0 },
    ];

    for (const volume of cases) {
      expect(heftOf(volume)).toBe(weigh(volume));
    }
  });

  it("treats a missing volume the same as an empty one", () => {
    expect(heftOf(undefined)).toBe(heftOf({}));
    expect(heftOf(undefined)).toBe(0);
  });
});
