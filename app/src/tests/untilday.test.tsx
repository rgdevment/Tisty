import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Repeat, Task } from "../core";
import { cadence } from "../format";
import Fields from "../ui/Fields";

const daily = (until?: string): Repeat => ({
  from: "due",
  each: { every: 1, unit: "day" },
  ...(until ? { until } : {}),
});

function put(repeat: Repeat) {
  const onPatch = vi.fn();
  const one = {
    id: "t1",
    title: "tomar la pastilla",
    status: "open",
    priority: "unset",
    order: "a0",
    repeat,
  } as Task;
  render(<Fields task={one} lists={[]} known={[]} onPatch={onPatch} />);
  return onPatch;
}

describe("a series with a last day", () => {
  it("says so on the chip, so it is not a promise for ever", () => {
    expect(cadence(daily("2026-09-30"))).toMatch(/until Sep 30/);
    expect(cadence(daily())).not.toMatch(/until/);
  });

  it("can be taken off again", () => {
    const onPatch = put(daily("2026-09-30"));

    fireEvent.click(screen.getByText(/↻ every day/));
    fireEvent.click(screen.getByText("No end"));

    expect(onPatch).toHaveBeenCalledWith({ repeat: { ...daily("2026-09-30"), until: null } });
  });

  it("offers no way to take one off when there is none", () => {
    put(daily());

    fireEvent.click(screen.getByText(/↻ every day/));

    expect(screen.getByText("Ends on…")).toBeTruthy();
    expect(screen.queryByText("No end")).toBeNull();
  });
});
