import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { dropLater, fireAll, UNDO_FOR, undo, useGoing, useHiddenGoing } from "../dropLater";
import { settled, unsettled } from "../saving";

const going = (hides: string[], run: () => Promise<void> = () => Promise.resolve()) =>
  dropLater({ hides, said: "gone", run });

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(async () => {
  fireAll();
  await vi.runAllTimersAsync();
  vi.useRealTimers();
});

describe("a delete that waits", () => {
  it("hides what goes at once and runs nothing yet", () => {
    const run = vi.fn(() => Promise.resolve());
    const hidden = renderHook(() => useHiddenGoing());

    act(() => {
      going(["d1", "f1"], run);
    });

    expect(hidden.result.current.has("d1")).toBe(true);
    expect(hidden.result.current.has("f1")).toBe(true);
    expect(run).not.toHaveBeenCalled();
  });

  it("runs once the undo time is over, and shows nothing waiting after", async () => {
    const run = vi.fn(() => Promise.resolve());
    const hidden = renderHook(() => useHiddenGoing());
    const waiting = renderHook(() => useGoing());
    act(() => {
      going(["d1"], run);
    });

    await act(() => vi.advanceTimersByTimeAsync(UNDO_FOR - 1));
    expect(run).not.toHaveBeenCalled();
    await act(() => vi.advanceTimersByTimeAsync(1));

    expect(run).toHaveBeenCalledTimes(1);
    expect(hidden.result.current.has("d1")).toBe(false);
    expect(waiting.result.current).toEqual([]);
  });

  it("never runs what was undone, and shows it again", async () => {
    const run = vi.fn(() => Promise.resolve());
    const hidden = renderHook(() => useHiddenGoing());
    let key = "";
    act(() => {
      key = going(["d1"], run);
    });

    act(() => undo(key));
    await act(() => vi.advanceTimersByTimeAsync(UNDO_FOR * 2));

    expect(run).not.toHaveBeenCalled();
    expect(hidden.result.current.has("d1")).toBe(false);
  });

  it("undoes one delete and leaves the others waiting", async () => {
    const kept = vi.fn(() => Promise.resolve());
    const undone = vi.fn(() => Promise.resolve());
    let key = "";
    act(() => {
      going(["a"], kept);
      key = going(["b"], undone);
    });

    act(() => undo(key));
    await act(() => vi.advanceTimersByTimeAsync(UNDO_FOR));

    expect(kept).toHaveBeenCalledTimes(1);
    expect(undone).not.toHaveBeenCalled();
  });

  it("runs everything pending at once when asked, and not again when the time is over", async () => {
    const one = vi.fn(() => Promise.resolve());
    const two = vi.fn(() => Promise.resolve());
    act(() => {
      going(["a"], one);
      going(["b"], two);
    });

    await act(async () => {
      fireAll();
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(() => vi.advanceTimersByTimeAsync(UNDO_FOR * 2));

    expect(one).toHaveBeenCalledTimes(1);
    expect(two).toHaveBeenCalledTimes(1);
  });

  it("keeps the order of the deletes it runs", async () => {
    const order: string[] = [];
    act(() => {
      going(["a"], () => Promise.resolve().then(() => void order.push("a")));
      going(["b"], () => Promise.resolve().then(() => void order.push("b")));
    });

    await act(() => vi.advanceTimersByTimeAsync(UNDO_FOR));

    expect(order).toEqual(["a", "b"]);
  });

  it("goes on when one delete fails", async () => {
    const after = vi.fn(() => Promise.resolve());
    act(() => {
      going(["a"], () => Promise.reject(new Error("no")));
      going(["b"], after);
    });

    await act(() => vi.advanceTimersByTimeAsync(UNDO_FOR));

    expect(after).toHaveBeenCalledTimes(1);
  });

  it("is held by the window's close: it counts as unsaved, and settling runs it", async () => {
    const run = vi.fn(() => Promise.resolve());
    act(() => {
      going(["a"], run);
    });
    expect(unsettled()).toBe(true);

    await act(() => settled());

    expect(run).toHaveBeenCalledTimes(1);
    expect(unsettled()).toBe(false);
  });
});
