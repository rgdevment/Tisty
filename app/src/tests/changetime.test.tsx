import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import When from "../ui/When";

function put(over: { value?: string; clock?: string }) {
  const onPick = vi.fn();
  render(<When {...over} onPick={onPick} onClear={vi.fn()} onClose={vi.fn()} />);
  return onPick;
}

describe("changing only the hour", () => {
  it("can be applied, because an hour asks for a button", () => {
    const onPick = put({ value: "2026-08-20", clock: "10:00" });

    fireEvent.change(screen.getByLabelText(/time/i), { target: { value: "09:00" } });
    fireEvent.click(screen.getByText("Apply"));

    expect(onPick).toHaveBeenCalledWith("2026-08-20T09:00:00");
  });

  it("asks for one as soon as an hour is typed on an all-day date", () => {
    const onPick = put({ value: "2026-08-20" });

    expect(screen.queryByText("Apply")).toBeNull();
    fireEvent.change(screen.getByLabelText(/time/i), { target: { value: "07:30" } });
    fireEvent.click(screen.getByText("Apply"));

    expect(onPick).toHaveBeenCalledWith("2026-08-20T07:30:00");
  });

  it("still applies a bare day without one", () => {
    const onPick = put({ value: "2026-08-20" });

    fireEvent.click(screen.getAllByText("15")[0]);

    expect(onPick).toHaveBeenCalledTimes(1);
    expect(onPick.mock.calls[0][0]).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });
});
