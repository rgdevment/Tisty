import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { settles } from "../ui/Editor";

let waiting: (() => void)[] = [];

beforeEach(() => {
  waiting = [];
  vi.stubGlobal("requestAnimationFrame", (fn: () => void) => {
    waiting.push(fn);
    return waiting.length;
  });
  vi.stubGlobal("cancelAnimationFrame", () => {});
});

afterEach(() => vi.unstubAllGlobals());

const frames = (many: number) => {
  for (let n = 0; n < many; n += 1) {
    const due = waiting;
    waiting = [];
    for (const one of due) one();
  }
};

const scroller = (room: () => number) => {
  const at = document.createElement("div");
  let put = 0;
  Object.defineProperty(at, "scrollTop", {
    get: () => put,
    set: (asked: number) => {
      put = Math.min(asked, room());
    },
  });
  return at;
};

describe("being put back where the sheet was left", () => {
  it("keeps asking while the editor is still laying the page out", () => {
    let room = 0;
    const at = scroller(() => room);
    const done = vi.fn();
    settles(() => at, 900, done);

    frames(3);
    expect(at.scrollTop).toBe(0);
    expect(done).not.toHaveBeenCalled();

    room = 2000;
    frames(2);
    expect(at.scrollTop).toBe(900);
    expect(done).toHaveBeenCalled();
  });

  it("settles at once when the room is already there", () => {
    const at = scroller(() => 2000);
    const done = vi.fn();
    settles(() => at, 900, done);

    frames(2);
    expect(at.scrollTop).toBe(900);
    expect(done).toHaveBeenCalledTimes(1);
  });

  it("keeps quiet for one more frame, so its own last scroll is not saved as yours", () => {
    const at = scroller(() => 2000);
    const done = vi.fn();
    settles(() => at, 900, done);

    frames(1);
    expect(at.scrollTop).toBe(900);
    expect(done).not.toHaveBeenCalled();

    frames(1);
    expect(done).toHaveBeenCalled();
  });

  it("gives way the moment the reader scrolls, instead of dragging them back", () => {
    let room = 0;
    const at = scroller(() => room);
    const done = vi.fn();
    settles(() => at, 900, done);

    frames(1);
    room = 2000;
    at.scrollTop = 120;
    frames(1);

    expect(at.scrollTop).toBe(120);
    expect(done).toHaveBeenCalled();

    frames(3);
    expect(at.scrollTop).toBe(120);
  });

  it("gives up rather than asking for ever for a spot that never comes", () => {
    const at = scroller(() => 10);
    const done = vi.fn();
    settles(() => at, 900, done);

    frames(200);
    expect(done).toHaveBeenCalled();
    expect(at.scrollTop).toBe(10);
  });

  it("stops when the sheet it was given goes away", () => {
    const done = vi.fn();
    settles(() => null, 900, done);

    frames(1);
    expect(done).toHaveBeenCalled();
  });
});
