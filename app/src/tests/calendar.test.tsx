import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Calendar from "../ui/Calendar";

const show = (value?: string) => {
  const onPick = vi.fn();
  render(<Calendar inline value={value} onPick={onPick} onClear={vi.fn()} onClose={vi.fn()} />);
  return onPick;
};

const iso = (away = 0) => {
  const on = new Date();
  on.setDate(on.getDate() + away);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${on.getFullYear()}-${pad(on.getMonth() + 1)}-${pad(on.getDate())}`;
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
    show(iso(3));

    expect(marked().textContent).toBe(String(new Date().getDate()));
  });

  it("lets the chosen day keep its own look when it is today", () => {
    show(iso());
    const chosen = marked();

    expect(chosen.className).toContain("bg-accent");
    expect(chosen.className.split(/\s+/)).toContain("text-bg");
  });
});
