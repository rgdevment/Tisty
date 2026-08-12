import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { DateSpec } from "../core";
import Recall from "../ui/Recall";

/// A task with no date offers only «pick a day», because there is nothing to
/// count an offset back from. That is the path that blanked the window.
describe("picking a day for a reminder", () => {
  it("opens the calendar on a task that has no date", () => {
    render(<Recall on={undefined} taken={[]} onAdd={vi.fn()} onClose={vi.fn()} />);

    expect(() => fireEvent.click(screen.getByText("Pick a day and time…"))).not.toThrow();
  });

  it("opens the calendar on a task that has one", () => {
    const at = new Date(Date.now() + 86_400_000);
    const pad = (n: number) => String(n).padStart(2, "0");
    const on = {
      at: `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}T09:00:00`,
      has_time: true,
      tz: "Europe/Madrid",
    } as DateSpec;

    render(<Recall on={on} taken={[]} onAdd={vi.fn()} onClose={vi.fn()} />);

    expect(() => fireEvent.click(screen.getByText("Pick a day and time…"))).not.toThrow();
  });
});
