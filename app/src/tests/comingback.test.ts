import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { settles } from "../ui/Editor";

let waiting: (() => void)[] = [];
let watching: (() => void)[] = [];

class Watcher {
  constructor(private tell: () => void) {}
  observe() {
    watching.push(this.tell);
  }
  disconnect() {
    watching = watching.filter((one) => one !== this.tell);
  }
}

beforeEach(() => {
  waiting = [];
  watching = [];
  vi.stubGlobal("requestAnimationFrame", (fn: () => void) => {
    waiting.push(fn);
    return waiting.length;
  });
  vi.stubGlobal("cancelAnimationFrame", () => {});
  vi.stubGlobal("ResizeObserver", Watcher);
});

afterEach(() => vi.unstubAllGlobals());

const frames = (many: number) => {
  for (let n = 0; n < many; n += 1) {
    const due = waiting;
    waiting = [];
    for (const one of due) one();
  }
};

const grew = () => {
  for (const tell of [...watching]) tell();
};

const scroller = (room: () => number) => {
  const at = document.createElement("div");
  let put = 0;
  let wrote = 0;
  Object.defineProperty(at, "scrollTop", {
    get: () => put,
    set: (asked: number) => {
      wrote += 1;
      put = Math.max(0, Math.min(asked, room()));
      at.dispatchEvent(new Event("scroll"));
    },
  });
  return { at, wrote: () => wrote, moved: (to: number) => (put = to) };
};

const started = (at: HTMLElement, want: number, room = document.createElement("div")) => {
  const done = vi.fn();
  const stop = settles({ at: () => at, want, done, put: () => {}, watch: () => room });
  return { done, stop };
};

describe("being put back where the sheet was left", () => {
  it("goes straight there when the room is already made", () => {
    const sheet = scroller(() => 4000);
    const { done } = started(sheet.at, 800);

    frames(1);

    expect(sheet.at.scrollTop).toBe(800);
    expect(done).toHaveBeenCalled();
  });

  it("waits for the room to arrive, however late, without asking every frame", () => {
    let tall = 0;
    const sheet = scroller(() => tall);
    const { done } = started(sheet.at, 800);

    frames(1);
    expect(sheet.at.scrollTop).toBe(0);
    expect(done).not.toHaveBeenCalled();

    frames(600);
    expect(sheet.wrote()).toBeLessThan(4);

    tall = 4000;
    grew();

    expect(sheet.at.scrollTop).toBe(800);
    expect(done).toHaveBeenCalled();
  });

  it("takes the room as it comes, in pieces, however long that takes", () => {
    let tall = 0;
    const sheet = scroller(() => tall);
    const { done } = started(sheet.at, 800);

    frames(1);
    for (const step of [200, 400, 600]) {
      tall = step;
      grew();
      expect(done).not.toHaveBeenCalled();
    }

    tall = 900;
    grew();

    expect(sheet.at.scrollTop).toBe(800);
    expect(done).toHaveBeenCalled();
  });

  it("lets go the moment the reader scrolls somewhere else", () => {
    let tall = 0;
    const sheet = scroller(() => tall);
    const { done } = started(sheet.at, 800);

    frames(1);
    sheet.moved(150);
    sheet.at.dispatchEvent(new Event("scroll"));

    expect(done).toHaveBeenCalled();

    tall = 4000;
    grew();
    expect(sheet.at.scrollTop).toBe(150);
  });

  it("stops when the sheet it was given goes away", () => {
    const gone = { current: null as HTMLElement | null };
    const done = vi.fn();
    settles({ at: () => gone.current, want: 800, done, put: () => {} });

    frames(1);

    expect(done).toHaveBeenCalled();
  });

  it("stops watching once it is told to stop", () => {
    let tall = 0;
    const sheet = scroller(() => tall);
    const { done, stop } = started(sheet.at, 800);

    frames(1);
    stop();
    expect(done).toHaveBeenCalledTimes(1);

    tall = 4000;
    grew();
    expect(sheet.at.scrollTop).toBe(0);
  });
});
