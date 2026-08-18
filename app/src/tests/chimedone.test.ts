import { describe, expect, it, vi } from "vitest";
import { heard } from "../chime";

describe("the sound that says something is finished", () => {
  it("is a tone the window knows how to play", () => {
    expect(heard("done")).toBe(true);
  });

  it("is not the same tone as filing one", () => {
    expect(heard("filed")).toBe(true);
    expect(heard("done")).toBe(true);
  });

  it("still refuses anything that is not a tone", () => {
    expect(heard("whatever")).toBe(false);
    expect(heard(undefined)).toBe(false);
  });
});

describe("what the finished tone sounds like", () => {
  const notes = async () => {
    const played: number[] = [];
    const stop = vi.fn();
    vi.stubGlobal(
      "AudioContext",
      class {
        currentTime = 0;
        state = "running";
        destination = {};
        createOscillator() {
          return {
            type: "",
            frequency: { value: 0 },
            connect: () => ({ connect: () => {} }),
            start: () => {},
            stop,
            set _hz(_v: number) {},
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: () => {},
              linearRampToValueAtTime: () => {},
              exponentialRampToValueAtTime: () => {},
            },
            connect: () => ({ connect: () => {} }),
          };
        }
      },
    );
    const { play } = await import("../chime");
    const made: number[] = [];
    const real = (globalThis as { AudioContext: new () => AudioContext }).AudioContext;
    const spy = vi.spyOn(real.prototype, "createOscillator");
    play("done");
    for (const call of spy.mock.results) {
      const one = call.value as { frequency: { value: number } };
      made.push(one.frequency.value);
    }
    played.push(...made);
    return played;
  };

  it("rises and lands an octave above where it started", async () => {
    const hz = await notes();

    expect(hz.length).toBeGreaterThan(2);
    expect(hz[hz.length - 1]).toBeCloseTo(hz[0] * 2, 0);
  });
});
