import { fireEvent, render, screen } from "@testing-library/react";
import { beforeAll, describe, expect, it, vi } from "vitest";
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
});
