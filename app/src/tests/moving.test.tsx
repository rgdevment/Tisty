import { act, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { weigh } from "../format";
import { fill, t } from "../locales";
import Moving from "../Moving";

const bus = vi.hoisted(() => ({
  heard: new Map<string, ((said: { payload: unknown }) => void)[]>(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (named: string, fn: (said: { payload: unknown }) => void) => {
    bus.heard.set(named, [...(bus.heard.get(named) ?? []), fn]);
    return Promise.resolve(() => {});
  },
}));

const told = (payload: unknown) => {
  for (const fn of bus.heard.get("moving") ?? []) fn({ payload });
};

beforeEach(() => bus.heard.clear());

describe("the window that shows the store moving", () => {
  it("says it is getting ready before the first word arrives", () => {
    render(<Moving />);

    expect(screen.getByText(t("movingTitle"))).toBeTruthy();
    expect(screen.getByText(t("movingStarting"))).toBeTruthy();
  });

  it("says how much has been carried and fills the bar to match", () => {
    render(<Moving />);

    act(() => told({ done: 1024 * 1024, whole: 4 * 1024 * 1024 }));

    expect(
      screen.getByText(fill("movingAlong", weigh(1024 * 1024), weigh(4 * 1024 * 1024))),
    ).toBeTruthy();
    expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe("25");
  });

  it("never overfills the bar when the count runs past what was weighed", () => {
    render(<Moving />);

    act(() => told({ done: 10, whole: 5 }));

    expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe("100");
  });
});
