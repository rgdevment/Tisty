import { afterEach, describe, expect, it, vi } from "vitest";
import { gathering } from "../refreshing";

const NOTHING = { tasks: false, papers: false, fresh: false, seen: false };

afterEach(() => {
  vi.clearAllTimers();
});

describe("gathering what a round tells in one breath", () => {
  it("refreshes once, after the window, for several asks inside it", () => {
    const refresh = vi.fn();
    const { ask } = gathering(refresh, 200);

    ask({ tasks: true });
    vi.advanceTimersByTime(100);
    ask({ papers: true });
    vi.advanceTimersByTime(99);
    expect(refresh).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);

    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it("joins what each ask wanted and leaves out what none did", () => {
    const refresh = vi.fn();
    const { ask } = gathering(refresh, 200);

    ask({ tasks: true });
    ask({ fresh: true });
    ask({ tasks: false });
    vi.advanceTimersByTime(200);

    expect(refresh).toHaveBeenCalledWith({ ...NOTHING, tasks: true, fresh: true });
  });

  it("waits the window from the first ask, not from the last", () => {
    const refresh = vi.fn();
    const { ask } = gathering(refresh, 200);

    ask({ seen: true });
    vi.advanceTimersByTime(150);
    ask({ seen: true });
    vi.advanceTimersByTime(50);

    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it("starts afresh for an ask that comes after a refresh", () => {
    const refresh = vi.fn();
    const { ask } = gathering(refresh, 200);

    ask({ tasks: true, papers: true });
    vi.advanceTimersByTime(200);
    ask({ seen: true });
    vi.advanceTimersByTime(200);

    expect(refresh).toHaveBeenNthCalledWith(1, { ...NOTHING, tasks: true, papers: true });
    expect(refresh).toHaveBeenNthCalledWith(2, { ...NOTHING, seen: true });
  });

  it("refreshes nothing when it is stopped before the window ends", () => {
    const refresh = vi.fn();
    const { ask, stop } = gathering(refresh, 200);

    ask({ tasks: true });
    stop();
    vi.advanceTimersByTime(1000);

    expect(refresh).not.toHaveBeenCalled();
  });

  it("forgets what was asked before it stopped", () => {
    const refresh = vi.fn();
    const { ask, stop } = gathering(refresh, 200);

    ask({ tasks: true });
    stop();
    ask({ papers: true });
    vi.advanceTimersByTime(200);

    expect(refresh).toHaveBeenCalledTimes(1);
    expect(refresh).toHaveBeenCalledWith({ ...NOTHING, papers: true });
  });

  it("waits two hundred milliseconds when no window is given", () => {
    const refresh = vi.fn();
    const { ask } = gathering(refresh);

    ask({ tasks: true });
    vi.advanceTimersByTime(199);
    expect(refresh).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);

    expect(refresh).toHaveBeenCalledTimes(1);
  });
});
