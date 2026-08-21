import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Fields from "../ui/Fields";

function habit(): Task {
  const at = new Date(Date.now() + 86_400_000);
  const pad = (n: number) => String(n).padStart(2, "0");
  const day = `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}`;
  return {
    id: "t1",
    title: "tomar la paroxetina",
    status: "open",
    priority: "unset",
    order: "a0",
    date: { at: `${day}T09:00:00`, has_time: true, tz: "Europe/Madrid" },
    repeat: { from: "done", each: { every: 1, unit: "day" } },
  } as Task;
}

describe("adding a reminder to a repeating task", () => {
  it("does not blow up on the way", () => {
    const onPatch = vi.fn();
    render(<Fields task={habit()} lists={[]} known={[]} onPatch={onPatch} />);

    fireEvent.click(screen.getByText(/⏰ reminder/));
    fireEvent.click(screen.getByText("At the time"));

    expect(onPatch).toHaveBeenCalledTimes(1);
    expect(onPatch.mock.calls[0][0].remind).toMatch(/T09:00:00$/);
  });
});
