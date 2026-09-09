import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { t } from "../locales";
import Ahead from "../ui/Ahead";

const dayFrom = (away: number, clock = "16:00:00"): string => {
  const at = new Date();
  at.setDate(at.getDate() + away);
  const month = String(at.getMonth() + 1).padStart(2, "0");
  const day = String(at.getDate()).padStart(2, "0");
  return `${at.getFullYear()}-${month}-${day}T${clock}`;
};

const task = (id: string, title: string, at?: string, timed = false): Task =>
  ({
    id,
    title,
    status: "open",
    priority: "unset",
    order: "a0",
    steps: [],
    log: [],
    volume: {},
    ...(at ? { date: { at, tz: "America/Santiago", floating: true, has_time: timed } } : {}),
  }) as unknown as Task;

describe("the week ahead", () => {
  it("puts a timed thing under its day with its hour", () => {
    render(<Ahead tasks={[task("01A", "Médico", dayFrom(2), true)]} days={7} onOpen={vi.fn()} />);

    expect(screen.getByText("Médico")).toBeTruthy();
    expect(screen.getByText(/16/)).toBeTruthy();
  });

  it("calls a day with nothing on it free", () => {
    render(<Ahead tasks={[task("01A", "Médico", dayFrom(1), true)]} days={3} onOpen={vi.fn()} />);

    expect(screen.getAllByText(t("aheadFree"))).toHaveLength(2);
  });

  it("says so when the whole window is empty", () => {
    render(<Ahead tasks={[]} days={7} onOpen={vi.fn()} />);

    expect(screen.getByText(t("aheadNothing"))).toBeTruthy();
    expect(screen.queryByText(t("aheadFree"))).toBeNull();
  });

  it("leaves out what falls past the window", () => {
    render(<Ahead tasks={[task("01A", "Vuelo", dayFrom(30), true)]} days={7} onOpen={vi.fn()} />);

    expect(screen.queryByText("Vuelo")).toBeNull();
  });

  it("leaves out today, which the list already holds", () => {
    render(<Ahead tasks={[task("01A", "Banco", dayFrom(0), true)]} days={7} onOpen={vi.fn()} />);

    expect(screen.queryByText("Banco")).toBeNull();
  });

  it("opens what you click", async () => {
    const opened = vi.fn();
    render(<Ahead tasks={[task("01A", "Kermés", dayFrom(2))]} days={7} onOpen={opened} />);

    await userEvent.click(screen.getByText("Kermés"));

    expect(opened).toHaveBeenCalledWith("01A");
  });

  it("sorts the timed ones first, by the clock", () => {
    render(
      <Ahead
        tasks={[
          task("01A", "Informe", dayFrom(2)),
          task("01B", "Tarde", dayFrom(2, "18:00:00"), true),
          task("01C", "Mañana", dayFrom(2, "09:00:00"), true),
        ]}
        days={7}
        onOpen={vi.fn()}
      />,
    );

    const shown = screen.getAllByRole("button").map((one) => one.textContent ?? "");

    expect(shown[0]).toContain("Mañana");
    expect(shown[1]).toContain("Tarde");
    expect(shown[2]).toContain("Informe");
  });
});
