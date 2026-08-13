import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { DateSpec } from "../core";
import Recall from "../ui/Recall";

function spec(at: string, has_time: boolean): DateSpec {
  return { at, has_time, tz: "Europe/Madrid" } as DateSpec;
}

const SOON = () => {
  const at = new Date(Date.now() + 6 * 3600_000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}T${pad(at.getHours())}:00:00`;
};

function put(on: DateSpec) {
  const onAdd = vi.fn();
  render(<Recall on={on} taken={[]} onAdd={onAdd} onClose={vi.fn()} />);
  return onAdd;
}

describe("Recall", () => {
  it("offers the event's own time, which is what a daily medicine needs", async () => {
    const at = SOON();
    const onAdd = put(spec(at, true));

    const row = screen.getByText("At the time");
    row.click();

    expect(onAdd).toHaveBeenCalledWith(at);
  });

  it("keeps offering the three that come before it", () => {
    put(spec(SOON(), true));

    expect(screen.getByText("An hour before")).toBeTruthy();
    expect(screen.getByText("30 minutes before")).toBeTruthy();
    expect(screen.getByText("15 minutes before")).toBeTruthy();
  });

  it("says nothing about a time an all-day task does not have", () => {
    const at = new Date(Date.now() + 3 * 86_400_000);
    const pad = (n: number) => String(n).padStart(2, "0");
    put(spec(`${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}`, false));

    expect(screen.queryByText("At the time")).toBeNull();
    expect(screen.getByText("An hour before")).toBeTruthy();
  });
});
