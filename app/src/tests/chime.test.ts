import { beforeEach, describe, expect, it, vi } from "vitest";

const started: number[] = [];
const stopped: number[] = [];
const pitches: number[] = [];
const connected: unknown[] = [];

class FakeGain {
  gain = {
    setValueAtTime: vi.fn(),
    linearRampToValueAtTime: vi.fn(),
    exponentialRampToValueAtTime: vi.fn(),
  };
  connect = vi.fn((to: unknown) => {
    connected.push(to);
    return to;
  });
}

class FakeOscillator {
  type = "";
  frequency = {
    set value(hz: number) {
      pitches.push(hz);
    },
    get value() {
      return 0;
    },
  };
  connect = vi.fn((to: unknown) => to);
  start = vi.fn((at: number) => started.push(at));
  stop = vi.fn((at: number) => stopped.push(at));
}

class FakeAudio {
  static made = 0;
  state = "running";
  currentTime = 0;
  destination = "speakers";
  resume = vi.fn();
  constructor() {
    FakeAudio.made += 1;
  }
  createGain() {
    return new FakeGain();
  }
  createOscillator() {
    return new FakeOscillator();
  }
}

describe("the chime", () => {
  beforeEach(() => {
    started.length = 0;
    stopped.length = 0;
    pitches.length = 0;
    connected.length = 0;
    FakeAudio.made = 0;
    vi.stubGlobal("AudioContext", FakeAudio);
    vi.resetModules();
  });

  /// A single beep reads as a system error. Filing plays a major triad that
  /// climbs, which is what makes a sound lift rather than merely ping.
  it("plays a rising figure when a task is filed", async () => {
    const { play } = await import("../chime");

    play("filed");

    expect(started).toHaveLength(3);
    expect(started[0]).toBeLessThan(started[1]);
    expect(started[1]).toBeLessThan(started[2]);
    expect(pitches).toEqual([...pitches].sort((a, b) => a - b));
  });

  /// The notes overlap instead of following one another — that is the whole
  /// difference between a chord opening out and three separate beeps.
  it("lets each note still be ringing when the next arrives", async () => {
    const { play } = await import("../chime");

    play("filed");

    const apart = started[1] - started[0];
    expect(apart).toBeGreaterThan(0);
    expect(apart).toBeLessThan(0.2);
    expect(stopped[0]).toBeGreaterThan(started[2]);
  });

  /// A reminder has to carry further than the tick of a capture.
  it("plays a longer figure for a reminder", async () => {
    const { play } = await import("../chime");

    play("due");

    expect(started).toHaveLength(3);
    expect(started[0]).toBeLessThan(started[2]);
  });

  it("reuses one audio context instead of leaking one per sound", async () => {
    const { play } = await import("../chime");

    play("filed");
    play("filed");
    play("due");

    expect(FakeAudio.made).toBe(1);
  });

  /// Chrome suspends the context until a gesture; without this the first
  /// reminder of the session is silent.
  it("wakes a suspended context", async () => {
    class Asleep extends FakeAudio {
      state = "suspended";
    }
    vi.stubGlobal("AudioContext", Asleep);
    vi.resetModules();
    const { play } = await import("../chime");

    play("due");

    expect(started).toHaveLength(3);
  });

  it("stays quiet where there is no audio at all", async () => {
    vi.stubGlobal("AudioContext", undefined);
    vi.resetModules();
    const { play } = await import("../chime");

    expect(() => play("filed")).not.toThrow();
  });

  it("ignores a tone it does not know", async () => {
    const { heard } = await import("../chime");

    expect(heard("filed")).toBe(true);
    expect(heard("due")).toBe(true);
    expect(heard("explosion")).toBe(false);
    expect(heard(undefined)).toBe(false);
  });

  /// Told apart across a room: filing opens out, a reminder ticks.
  it("keeps the two shapes distinct", async () => {
    const { play } = await import("../chime");

    play("due");
    const ticks = stopped[0] - started[0];
    started.length = 0;
    stopped.length = 0;
    play("filed");
    const rings = stopped[0] - started[0];

    expect(rings).toBeGreaterThan(ticks);
  });
});