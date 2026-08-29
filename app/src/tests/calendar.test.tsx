import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Calendar from "../ui/Calendar";

const show = (value?: string) => {
  const onPick = vi.fn();
  render(<Calendar inline value={value} onPick={onPick} onClear={vi.fn()} onClose={vi.fn()} />);
  return onPick;
};

const iso = (day?: number) => {
  const on = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${on.getFullYear()}-${pad(on.getMonth() + 1)}-${pad(day ?? on.getDate())}`;
};

// Another day of the month this calendar is already showing. Reaching into the next month
// would scroll today out of view, which is a different question from the one being asked.
const anotherDay = () => {
  const today = new Date().getDate();
  return today > 14 ? today - 3 : today + 3;
};

const marked = () => screen.getByRole("button", { current: "date" as never });

describe("knowing which day is today", () => {
  it("marks today, so you can see where you are standing", () => {
    show();

    expect(marked().textContent).toBe(String(new Date().getDate()));
  });

  it("dresses it in one colour, not two that fight over which wins", () => {
    show();
    const classes = marked().className.split(/\s+/);

    expect(classes.filter((one) => /^text-(ink|faint|accent|bg)$/.test(one))).toEqual([
      "text-accent",
    ]);
  });

  it("still marks today when another day is the chosen one", () => {
    show(iso(anotherDay()));

    expect(marked().textContent).toBe(String(new Date().getDate()));
  });

  it("lets the chosen day keep its own look when it is today", () => {
    show(iso());
    const chosen = marked();

    expect(chosen.className).toContain("bg-accent");
    expect(chosen.className.split(/\s+/)).toContain("text-bg");
  });
});
