import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { BACKING_OFF, backoff, beat } from "../pacing";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
});

describe("beat", () => {
  it("fires once after the calm it was given", () => {
    const fire = vi.fn();
    const pace = beat(fire, 700, 2000);

    pace.soon();
    vi.advanceTimersByTime(699);
    expect(fire).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(fire).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(5000);
    expect(fire).toHaveBeenCalledTimes(1);
  });

  it("waits longer while keys keep coming closer than the calm", () => {
    const fire = vi.fn();
    const pace = beat(fire, 700, 2000);

    pace.soon();
    vi.advanceTimersByTime(500);
    pace.soon();
    vi.advanceTimersByTime(500);

    expect(fire).not.toHaveBeenCalled();
  });

  it("fires at most every `most` ms under continuous keys", () => {
    const at: number[] = [];
    const start = Date.now();
    const pace = beat(() => at.push(Date.now() - start), 700, 2000);

    for (let spent = 0; spent < 12_000; spent += 35) {
      pace.soon();
      vi.advanceTimersByTime(35);
    }

    expect(at.length).toBeGreaterThanOrEqual(5);
    expect(at.length).toBeLessThanOrEqual(6);
    expect(at[0]).toBeLessThanOrEqual(2000);
    for (let i = 1; i < at.length; i++) expect(at[i] - at[i - 1]).toBeLessThanOrEqual(2000 + 35);
  });

  it("starts a new window after a fire, counted from the next key", () => {
    const fire = vi.fn();
    const pace = beat(fire, 700, 2000);

    pace.soon();
    vi.advanceTimersByTime(700);
    vi.advanceTimersByTime(10_000);
    pace.soon();
    vi.advanceTimersByTime(699);

    expect(fire).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(1);
    expect(fire).toHaveBeenCalledTimes(2);
  });

  it("fires at once when the window is already spent", () => {
    const fire = vi.fn();
    const pace = beat(fire, 700, 0);

    pace.soon();
    vi.advanceTimersByTime(0);

    expect(fire).toHaveBeenCalledTimes(1);
  });

  it("stop cancels the pending fire and resets the window", () => {
    const fire = vi.fn();
    const pace = beat(fire, 700, 2000);

    pace.soon();
    vi.advanceTimersByTime(600);
    pace.stop();
    vi.advanceTimersByTime(5000);
    expect(fire).not.toHaveBeenCalled();

    pace.soon();
    vi.advanceTimersByTime(699);
    expect(fire).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(fire).toHaveBeenCalledTimes(1);
  });

  it("stop with nothing pending does nothing", () => {
    const pace = beat(vi.fn(), 700, 2000);

    expect(() => pace.stop()).not.toThrow();
  });
});

describe("backoff", () => {
  it("waits 1s, 2s, 4s, 8s, then 15s every time", () => {
    const fire = vi.fn();
    const again = backoff(fire);
    const gaps: number[] = [];

    for (let round = 0; round < 7; round++) {
      again.later();
      const before = Date.now();
      while (fire.mock.calls.length === round) vi.advanceTimersByTime(1);
      gaps.push(Date.now() - before);
    }

    expect(gaps).toEqual([1000, 2000, 4000, 8000, 15000, 15000, 15000]);
    expect(BACKING_OFF).toEqual([1000, 2000, 4000, 8000, 15000]);
  });

  it("does nothing when asked again while one is pending", () => {
    const fire = vi.fn();
    const again = backoff(fire);

    again.later();
    again.later();
    again.later();
    vi.advanceTimersByTime(1000);
    expect(fire).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(60_000);
    expect(fire).toHaveBeenCalledTimes(1);
    again.later();
    vi.advanceTimersByTime(1999);
    expect(fire).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(1);
    expect(fire).toHaveBeenCalledTimes(2);
  });

  it("reset cancels the pending retry and starts over from the first wait", () => {
    const fire = vi.fn();
    const again = backoff(fire);
    again.later();
    vi.advanceTimersByTime(1000);
    again.later();
    vi.advanceTimersByTime(1000);

    again.reset();
    vi.advanceTimersByTime(60_000);
    expect(fire).toHaveBeenCalledTimes(1);

    again.later();
    vi.advanceTimersByTime(999);
    expect(fire).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(1);
    expect(fire).toHaveBeenCalledTimes(2);
  });

  it("reset with nothing pending does nothing", () => {
    expect(() => backoff(vi.fn()).reset()).not.toThrow();
  });

  it("takes its own waits", () => {
    const fire = vi.fn();
    const again = backoff(fire, [10, 20]);

    again.later();
    vi.advanceTimersByTime(10);
    again.later();
    vi.advanceTimersByTime(20);
    again.later();
    vi.advanceTimersByTime(20);

    expect(fire).toHaveBeenCalledTimes(3);
  });
});
