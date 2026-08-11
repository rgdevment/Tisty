import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Notice from "../ui/Notice";
import type { Task } from "../core";

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

    await userEvent.click(screen.getByRole("button"));

    expect(gone).toHaveBeenCalled();
  });

  /// Clicking the body opens the task; only the ✕ dismisses it.
  it("opens the task when the body is clicked", async () => {
    const opened = vi.fn();
    const gone = vi.fn();
    render(<Notice task={task} lists={[]} onOpen={opened} onDismiss={gone} />);

    await userEvent.click(screen.getByText("tomar la pastilla"));

    expect(opened).toHaveBeenCalled();
  });

  /// Written from «upcoming» or «repeating», a task with no date lands on today
  /// and vanishes from the list the instant it is typed. The card said the
  /// title and nothing about where it went.
  it("says so when what was filed is not in the list behind it", () => {
    render(
      <Notice task={task} lists={[]} elsewhere onOpen={() => {}} onDismiss={() => {}} />,
    );

    expect(screen.getByText(/Not in this view/i)).toBeTruthy();
  });

  it("stays quiet when it is right there", () => {
    render(<Notice task={task} lists={[]} onOpen={() => {}} onDismiss={() => {}} />);

    expect(screen.queryByText(/Not in this view/i)).toBeNull();
  });
});
