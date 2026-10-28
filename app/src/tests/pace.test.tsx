import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import type { Pace, Paced } from "../deadlines";
import PaceBar from "../ui/PaceBar";
import TaskList from "../ui/TaskList";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const a3 = {
  id: "A3",
  title: "A3 · Migración de Sodimac Uruguay",
  status: "open",
  priority: "unset",
  order: "a0",
  tags: [],
  reminders: [],
} as unknown as Task;

const slow: Paced = { pace: "slow", time: 36, done: 0 };

// One pace a line: how it is said and the colour that fills the bar.
const TONES = `
on_track|On track|bg-accent
slow|Running slow|bg-hue-amber
behind|Won't make it|bg-urgent
`;

describe("the pace of a task with parts", () => {
  it("shows A3 running slow under its title: time gone marked, nothing filled", () => {
    render(
      <TaskList tasks={[a3]} lists={[]} paces={{ A3: slow }} title="Hoy" onSelect={() => {}} />,
    );

    const bar = screen.getByRole("img", {
      name: "Running slow · 36% of the time gone · 0% of the parts closed",
    });
    const [filled, tick] = Array.from(bar.children) as HTMLElement[];
    expect(filled?.style.width).toBe("0%");
    expect(tick?.style.left).toBe("36%");
  });

  it("says the pace in the row's own name, so moving by keyboard hears it", () => {
    render(
      <TaskList tasks={[a3]} lists={[]} paces={{ A3: slow }} title="Hoy" onSelect={() => {}} />,
    );

    expect(screen.getByRole("listitem").getAttribute("aria-label")).toBe(
      "A3 · Migración de Sodimac Uruguay — Running slow · 36% of the time gone · 0% of the parts closed",
    );
  });

  it("draws no bar for a task without a pace", () => {
    render(<TaskList tasks={[a3]} lists={[]} title="Hoy" onSelect={() => {}} />);

    expect(screen.queryByRole("img")).toBeNull();
  });

  it("says each pace in its own colour", () => {
    for (const line of TONES.trim().split("\n")) {
      const [pace = "", said = "", tone = ""] = line.split("|");
      const { unmount } = render(<PaceBar paced={{ pace: pace as Pace, time: 50, done: 50 }} />);

      const bar = screen.getByRole("img", { name: new RegExp(`^${said}`) });
      expect(bar.firstElementChild?.className, line).toContain(tone);
      unmount();
    }
  });
});
