import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { t } from "../locales";
import Spread from "../ui/Spread";

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

const owing = (id: string, title: string, by: string): Task =>
  ({
    ...made(id, title),
    deadline: {
      at: `${by}T00:00:00`,
      tz: "America/Santiago",
      floating: true,
      has_time: false,
    },
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
const days = () => document.querySelectorAll("[data-day]");

describe("the river of days", () => {
  it("starts on the monday of the week you are living and runs on for months", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(days()).toHaveLength(120);
    expect(days()[0].getAttribute("data-day")).toBe(weekDay(0));
    expect(dayAt(119)).toBeTruthy();
  });

  it("names the month once, where it turns", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    const said = [...document.querySelectorAll("p")]
      .map((one) => one.textContent ?? "")
      .filter((one) => /\d{4}$/.test(one));

    expect(said.length).toBeGreaterThanOrEqual(4);
    expect(new Set(said).size).toBe(said.length);
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
    expect(dayAt(4)?.textContent).toContain("11:00");
  });

  it("does not call a day free when work is owed on it", () => {
    render(
      <Spread
        tasks={[owing("01A", "Entregar el informe", weekDay(4))]}
        onPlace={vi.fn()}
        onOpen={vi.fn()}
      />,
    );

    expect(dayAt(4)?.textContent).toContain("Entregar el informe");
    expect(dayAt(4)?.textContent).not.toContain(t("spreadFree"));
    expect(document.querySelector("[data-tray]")?.textContent).not.toContain("Entregar el informe");
  });

  it("calls a day with nothing on it free", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(dayAt(3)?.textContent).toContain(t("spreadFree"));
  });

  it("keeps what has no day at all in the tray", () => {
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(document.querySelector("[data-tray]")?.textContent).toContain("Sin fecha");
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

  it("draws every task the same way, whatever it carries", () => {
    render(
      <Spread
        tasks={[
          made("01A", "Con hora", weekDay(1), "09:00:00"),
          made("01B", "Sin hora", weekDay(1)),
        ]}
        onPlace={vi.fn()}
        onOpen={vi.fn()}
      />,
    );

    const shapes = [...(dayAt(1)?.querySelectorAll("button") ?? [])].map((one) =>
      one.className.replace(/cursor-\w+/, ""),
    );

    expect(shapes).toHaveLength(2);
    expect(shapes[0]).toBe(shapes[1]);
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
    fireEvent.click(card);

    expect(opened).toHaveBeenCalledWith(expect.objectContaining({ id: "01A" }));
  });

  it("opens from the keyboard, like every other list in the app", async () => {
    const opened = vi.fn();
    const user = userEvent.setup();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={vi.fn()} onOpen={opened} />);

    const card = screen.getByText("Sin fecha").closest("button") as HTMLElement;
    card.focus();
    await user.keyboard("{Enter}");

    expect(opened).toHaveBeenCalledWith(expect.objectContaining({ id: "01A" }));
  });

  it("says when it is carrying, so the panel over the days can step aside", () => {
    const carrying = vi.fn();
    render(
      <Spread
        tasks={[made("01A", "Sin fecha")]}
        onPlace={vi.fn()}
        onOpen={vi.fn()}
        onCarrying={carrying}
      />,
    );

    const was = document.elementFromPoint;
    document.elementFromPoint = () => document.body;
    const card = screen.getByText("Sin fecha");
    fireEvent.pointerDown(card, { button: 0, clientX: 0, clientY: 0 });
    fireEvent.pointerMove(card, { clientX: 80, clientY: 80 });
    expect(carrying).toHaveBeenLastCalledWith(true);

    fireEvent.pointerUp(card, { clientX: 80, clientY: 80 });
    document.elementFromPoint = was;
    expect(carrying).toHaveBeenLastCalledWith(false);
  });

  it("does not open what you only dragged somewhere else", () => {
    const opened = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={vi.fn()} onOpen={opened} />);

    const card = screen.getByText("Sin fecha");
    dragged(card, dayAt(3));
    fireEvent.click(card);

    expect(opened).not.toHaveBeenCalled();
  });

  it("reaches a day that already went by, so slipped work can be dealt again", () => {
    const gone = iso(new Date(new Date().getFullYear(), new Date().getMonth(), 1));
    render(<Spread tasks={[made("01A", "Se me pasó", gone)]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(document.querySelector(`[data-day="${gone}"]`)).toBeTruthy();
    expect(document.querySelector(`[data-day="${gone}"]`)?.textContent).toContain("Se me pasó");
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

  it("does nothing when a task is let go outside every day", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    dragged(screen.getByText("Sin fecha"), document.body);

    expect(placed).not.toHaveBeenCalled();
  });

  it("asks when the day it landed on is turning heavy, and offers a free one", () => {
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

  it("keeps looking for a free day past the end of the week", () => {
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
    fireEvent.click(
      screen.getByRole("button", { name: new RegExp(t("spreadMoveIt").split("{")[0].trim()) }),
    );

    expect(placed).toHaveBeenLastCalledWith("01C", weekDay(7));
  });
});

describe("the month, for looking and no more", () => {
  const open = () => fireEvent.click(screen.getByRole("button", { name: t("spreadMoon") }));

  it("lays out six weeks of cells and hides the river while it shows", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    open();

    expect(days()).toHaveLength(42);
    expect(screen.queryByText(t("spreadFree"))).toBeNull();
  });

  it("never calls a cell free, since Tisty cannot promise a day is", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    open();

    expect(document.body.textContent).not.toContain(t("spreadFree"));
  });

  it("shows what a day carries, and says how many it could not fit", () => {
    render(
      <Spread
        tasks={[
          made("01A", "Uno", weekDay(3)),
          made("01B", "Dos", weekDay(3)),
          made("01C", "Tres", weekDay(3)),
          made("01D", "Cuatro", weekDay(3)),
        ]}
        onPlace={vi.fn()}
        onOpen={vi.fn()}
      />,
    );

    open();
    const cell = document.querySelector(`[data-day="${weekDay(3)}"]`);

    expect(cell?.textContent).toContain("Uno");
    expect(cell?.textContent).toContain("+1");
    expect(cell?.textContent).not.toContain("Cuatro");
  });

  it("takes no task anywhere: pressing a day sends you back to it in the river", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    open();
    fireEvent.click(document.querySelector(`[data-day="${weekDay(5)}"]`) as HTMLElement);

    expect(placed).not.toHaveBeenCalled();
    expect(days()).toHaveLength(120);
  });

  it("walks a month back and a month on", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);
    open();

    const now = new Date();
    const said = () => screen.getByRole("heading", { level: 2 }).textContent ?? "";
    const was = said();

    fireEvent.click(screen.getByRole("button", { name: t("spreadOn") }));
    expect(said()).not.toBe(was);

    fireEvent.click(screen.getByRole("button", { name: t("spreadBack") }));
    expect(said()).toBe(was);
    expect(said()).toContain(String(now.getFullYear()));
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

  it("opens the week that monday starts, not the one it closed", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    expect(days()[0].getAttribute("data-day")).toBe("2026-09-07");
    expect(document.querySelector('[data-day="2026-08-31"]')).toBeNull();
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

  it("still lands a drop on the right date", () => {
    const placed = vi.fn();
    render(<Spread tasks={[made("01A", "Sin fecha")]} onPlace={placed} onOpen={vi.fn()} />);

    expect(document.querySelector('[data-day="2025-12-29"]')).toBeTruthy();
    expect(document.querySelector('[data-day="2026-01-04"]')).toBeTruthy();

    dragged(screen.getByText("Sin fecha"), document.querySelector('[data-day="2025-12-31"]'));

    expect(placed).toHaveBeenCalledWith("01A", "2025-12-31");
  });
});
