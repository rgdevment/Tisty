import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Owed from "../ui/Owed";

const days = (...back: number[]) =>
  back.map((n) => {
    const at = new Date();
    at.setDate(at.getDate() - n);
    return `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, "0")}-${String(at.getDate()).padStart(2, "0")}`;
  });

const shown = (list: string[]) => {
  const done = vi.fn();
  render(<Owed days={list} onConfirm={done} />);
  return done;
};

describe("claiming the days that went unmarked", () => {
  it("offers no way to confirm until a day is ticked, so it invents no history", () => {
    shown(days(1, 0));

    expect(screen.queryByRole("button", { name: /^add/i })).toBeNull();
  });

  it("names today and yesterday rather than dating them", () => {
    shown(days(1, 0));

    expect(screen.getByRole("button", { name: "today" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "yesterday" })).toBeTruthy();
  });

  it("hands back only the days that were ticked", async () => {
    const user = userEvent.setup();
    const [older, newer] = days(2, 1);
    const done = shown([older, newer]);

    await user.click(screen.getByRole("button", { name: "yesterday" }));
    await user.click(screen.getByRole("button", { name: /^add/i }));

    expect(done).toHaveBeenCalledWith([newer]);
  });

  it("says how many days it is about to add", async () => {
    const user = userEvent.setup();
    shown(days(2, 1));

    await user.click(screen.getByRole("button", { name: "yesterday" }));

    expect(screen.getByRole("button", { name: "Add 1" })).toBeTruthy();
  });

  it("lets a day be unticked again, which takes the confirmation away", async () => {
    const user = userEvent.setup();
    shown(days(1));
    const day = screen.getByRole("button", { name: "yesterday" });

    await user.click(day);
    expect(day.getAttribute("aria-pressed")).toBe("true");
    await user.click(day);

    expect(screen.queryByRole("button", { name: /^add/i })).toBeNull();
  });

  it("takes the focus, because the key that opens it moves on to the next row", () => {
    const [older] = days(2, 1);
    shown([older, ...days(1)]);

    expect(document.activeElement).toBe(screen.getAllByRole("button")[1]);
  });

  it("is a landmark a reader can be told about", () => {
    shown(days(1));

    expect(screen.getByRole("region", { name: /did you do it/i })).toBeTruthy();
  });

  it("marks the task and asks nothing more when Escape is pressed", async () => {
    const user = userEvent.setup();
    const done = shown(days(1));

    await user.keyboard("{Escape}");

    expect(done).toHaveBeenCalledWith([]);
  });

  it("closes the task with no days at all when the strip is dismissed", async () => {
    const user = userEvent.setup();
    const done = shown(days(1));

    await user.click(screen.getByRole("button", { name: /just mark it/i }));

    expect(done).toHaveBeenCalledWith([]);
  });
});
