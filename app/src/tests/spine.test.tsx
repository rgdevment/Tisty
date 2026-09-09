import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Coming, Task } from "../core";
import Spine from "../ui/Spine";

const stamp = (away: number): string => {
  const at = new Date();
  at.setDate(at.getDate() + away);
  const month = String(at.getMonth() + 1).padStart(2, "0");
  const day = String(at.getDate()).padStart(2, "0");
  return `${at.getFullYear()}-${month}-${day}`;
};

const coming = (id: string, title: string, away: number, clock = ""): Coming => {
  const task = {
    id,
    title,
    status: "open",
    priority: "unset",
    order: "a0",
    steps: [],
    log: [],
    volume: {},
    date: {
      at: `${stamp(away)}T${clock || "00:00:00"}`,
      tz: "America/Santiago",
      floating: true,
      has_time: clock !== "",
    },
  } as unknown as Task;
  return { task, on: stamp(away), due: false };
};

describe("the spine", () => {
  it("shows one knot per day and no titles", () => {
    render(
      <Spine
        coming={[coming("01A", "Kermés", 1, "11:00:00")]}
        routines={[]}
        days={3}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("button")).toHaveLength(3);
    expect(screen.queryByText("Kermés")).toBeNull();
  });

  it("unfolds the week when a knot is pressed", async () => {
    render(
      <Spine
        coming={[coming("01A", "Kermés", 1, "11:00:00")]}
        routines={[]}
        days={3}
        onOpen={vi.fn()}
      />,
    );

    await userEvent.click(screen.getAllByRole("button")[0]);

    expect(screen.getByText("Kermés")).toBeTruthy();
  });

  it("folds again on escape, and hands the focus back", async () => {
    render(
      <Spine
        coming={[coming("01A", "Kermés", 1, "11:00:00")]}
        routines={[]}
        days={3}
        onOpen={vi.fn()}
      />,
    );

    const knot = screen.getAllByRole("button")[0];
    await userEvent.click(knot);
    await userEvent.keyboard("{Escape}");

    expect(screen.queryByText("Kermés")).toBeNull();
    expect(document.activeElement).toBe(knot);
  });

  it("opens what you pick and folds itself", async () => {
    const opened = vi.fn();
    render(
      <Spine
        coming={[coming("01A", "Kermés", 1, "11:00:00")]}
        routines={[]}
        days={3}
        onOpen={opened}
      />,
    );

    await userEvent.click(screen.getAllByRole("button")[0]);
    await userEvent.click(screen.getByText("Kermés"));

    expect(opened).toHaveBeenCalledWith("01A");
    expect(screen.queryByText("Kermés")).toBeNull();
  });

  it("marks the knot of a heavy day", () => {
    render(
      <Spine
        coming={[
          coming("01A", "Kermés", 1, "11:00:00"),
          coming("01B", "Médico", 1, "16:00:00"),
          coming("01C", "Informe", 1),
        ]}
        routines={[]}
        days={2}
        onOpen={vi.fn()}
      />,
    );

    const knots = screen.getAllByRole("button");

    expect(knots[0].className).toContain("hue-amber");
    expect(knots[1].className).not.toContain("hue-amber");
  });
});
