import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Spine from "../ui/Spine";

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

describe("the spine", () => {
  it("shows one knot per day and no titles", () => {
    render(<Spine tasks={[task("01A", "Kermés", dayFrom(1), true)]} days={3} onOpen={vi.fn()} />);

    expect(screen.getAllByRole("button")).toHaveLength(3);
    expect(screen.queryByText("Kermés")).toBeNull();
  });

  it("unfolds the week when a knot is pressed", async () => {
    render(<Spine tasks={[task("01A", "Kermés", dayFrom(1), true)]} days={3} onOpen={vi.fn()} />);

    await userEvent.click(screen.getAllByRole("button")[0]);

    expect(screen.getByText("Kermés")).toBeTruthy();
  });

  it("folds again on escape", async () => {
    render(<Spine tasks={[task("01A", "Kermés", dayFrom(1), true)]} days={3} onOpen={vi.fn()} />);

    await userEvent.click(screen.getAllByRole("button")[0]);
    await userEvent.keyboard("{Escape}");

    expect(screen.queryByText("Kermés")).toBeNull();
  });

  it("opens what you pick and folds itself", async () => {
    const opened = vi.fn();
    render(<Spine tasks={[task("01A", "Kermés", dayFrom(1), true)]} days={3} onOpen={opened} />);

    await userEvent.click(screen.getAllByRole("button")[0]);
    await userEvent.click(screen.getByText("Kermés"));

    expect(opened).toHaveBeenCalledWith("01A");
    expect(screen.queryByText("Kermés")).toBeNull();
  });

  it("marks the knot of a heavy day", () => {
    const at = dayFrom(1);
    render(
      <Spine
        tasks={[
          task("01A", "Kermés", at, true),
          task("01B", "Médico", at, true),
          task("01C", "Informe", at),
        ]}
        days={2}
        onOpen={vi.fn()}
      />,
    );

    const knots = screen.getAllByRole("button");

    expect(knots[0].className).toContain("hue-amber");
    expect(knots[1].className).not.toContain("hue-amber");
  });
});
