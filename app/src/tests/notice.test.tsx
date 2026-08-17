import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Notice from "../ui/Notice";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = {
  id: "01T",
  title: "tomar la pastilla",
  status: "open",
  priority: 4,
  order: "a0",
  tags: [],
  date: { at: "2026-08-11T10:00:00", tz: "America/Santiago", floating: true, has_time: true },
} as unknown as Task;

describe("the capture notice", () => {
  beforeEach(() => vi.useFakeTimers({ shouldAdvanceTime: true }));
  afterEach(() => vi.useRealTimers());

  it("goes away on its own", async () => {
    const gone = vi.fn();
    render(<Notice task={task} lists={[]} onOpen={() => {}} onDismiss={gone} />);

    expect(gone).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(7000);

    expect(gone).toHaveBeenCalled();
  });

  it("can be closed by hand", async () => {
    const gone = vi.fn();
    render(<Notice task={task} lists={[]} onOpen={() => {}} onDismiss={gone} />);

    await userEvent.click(screen.getByRole("button", { name: /close/i }));

    expect(gone).toHaveBeenCalled();
  });

  it("waits while the keyboard is on it", async () => {
    const gone = vi.fn();
    render(<Notice task={task} lists={[]} onOpen={() => {}} onDismiss={gone} />);

    screen.getByRole("button", { name: /open/i }).focus();
    await vi.advanceTimersByTimeAsync(9000);

    expect(gone).not.toHaveBeenCalled();
  });

  it("is a button, not a div that happens to answer clicks", () => {
    render(<Notice task={task} lists={[]} onOpen={() => {}} onDismiss={() => {}} />);

    const open = screen.getByRole("button", { name: /open/i });
    expect(open.textContent).toContain("tomar la pastilla");
  });

  it("opens the task when the body is clicked", async () => {
    const opened = vi.fn();
    const gone = vi.fn();
    render(<Notice task={task} lists={[]} onOpen={opened} onDismiss={gone} />);

    await userEvent.click(screen.getByText("tomar la pastilla"));

    expect(opened).toHaveBeenCalled();
  });

  it("says so when what was filed is not in the list behind it", () => {
    render(<Notice task={task} lists={[]} elsewhere onOpen={() => {}} onDismiss={() => {}} />);

    expect(screen.getByText(/Not in this view/i)).toBeTruthy();
  });

  it("stays quiet when it is right there", () => {
    render(<Notice task={task} lists={[]} onOpen={() => {}} onDismiss={() => {}} />);

    expect(screen.queryByText(/Not in this view/i)).toBeNull();
  });
});
