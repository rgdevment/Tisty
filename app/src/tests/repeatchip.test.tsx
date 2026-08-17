import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Edits, Parsed } from "../core";
import Chips from "../ui/Chips";
import Field from "../ui/Field";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const seen = (repeat?: Parsed["repeat"]): Parsed => ({
  title: "tomar la pastilla",
  date: {
    at: "2026-08-11T10:00:00",
    tz: "America/Santiago",
    floating: true,
    has_time: true,
  },
  tags: [],
  spans: [],
  offers: [],
  repeat,
});

const show = (repeat?: Parsed["repeat"], edits: Edits = {}, onEdit = () => {}) =>
  render(<Chips seen={seen(repeat)} edits={edits} onEdit={onEdit} empty={<span />} />);

const daily = { from: "done", each: { every: 1, unit: "day" } } as const;

describe("the repeat chip", () => {
  it("says the cadence next to the date", () => {
    show(daily);

    expect(screen.getByText("every day")).toBeTruthy();
    expect(screen.getByText(/10:00/)).toBeTruthy();
  });

  it("counts when there is more than one", () => {
    show({ from: "done", each: { every: 3, unit: "week" } });

    expect(screen.getByText("every 3 weeks")).toBeTruthy();
  });

  it("can be taken off without touching the rest", async () => {
    const edited = vi.fn();
    show(daily, {}, edited);

    await userEvent.click(screen.getByRole("button", { name: /every day/i }));

    expect(edited).toHaveBeenCalledWith(expect.objectContaining({ noRepeat: true }));
  });

  it("is gone once it was taken off", () => {
    show(daily, { noRepeat: true });

    expect(screen.queryByText("every day")).toBeNull();
  });

  it("says nothing when the phrase had no cadence", () => {
    show(undefined);

    expect(screen.queryByText(/every/)).toBeNull();
  });
});

describe("the repeat mark in the text", () => {
  it("does not borrow the tint of another mark", () => {
    const { container } = render(
      <Field
        icon="+"
        value="tomar la pastilla cada día"
        marks={[{ span: { from: 18, to: 26, mark: "repeat", certainty: "sure" }, offered: false }]}
        hint=""
        onChange={() => {}}
        onSubmit={() => {}}
      />,
    );

    const painted = container.querySelector(".bg-mark-repeat");
    expect(painted).toBeTruthy();
    expect(container.querySelector(".bg-mark-list")).toBeNull();
  });
});
