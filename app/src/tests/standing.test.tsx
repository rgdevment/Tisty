import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Standing from "../Standing";

const store = vi.hoisted(() => ({ broke: [] as unknown[][] }));

vi.mock("../broke", () => ({
  broke: (...said: unknown[]) => store.broke.push(said),
}));

const Falls = () => {
  throw new TypeError("Cannot read properties of undefined (reading 'replace')");
};

describe("a window that stops drawing", () => {
  beforeEach(() => {
    store.broke = [];
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  it("says so and offers to draw it again, instead of going black", () => {
    render(
      <Standing>
        <Falls />
      </Standing>,
    );

    expect(screen.getByText(t("windowFell"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("windowFellAgain") })).toBeTruthy();
  });

  it("writes down what broke, with what it said", () => {
    render(
      <Standing>
        <Falls />
      </Standing>,
    );

    expect(store.broke).toHaveLength(1);
    expect(store.broke[0][0]).toBe("TypeError");
    expect(store.broke[0][1]).toContain("reading 'replace'");
  });

  it("draws the window again when asked", async () => {
    const reload = vi.fn();
    Object.defineProperty(window, "location", {
      value: { ...window.location, reload },
      writable: true,
    });

    render(
      <Standing>
        <Falls />
      </Standing>,
    );
    await userEvent.click(screen.getByRole("button", { name: t("windowFellAgain") }));

    expect(reload).toHaveBeenCalled();
  });

  it("stays out of the way while nothing is wrong", () => {
    render(
      <Standing>
        <p>lo de siempre</p>
      </Standing>,
    );

    expect(screen.getByText("lo de siempre")).toBeTruthy();
    expect(store.broke).toHaveLength(0);
  });
});
