import { beforeEach, describe, expect, it, vi } from "vitest";
import { settles } from "../ui/Editor";

const watchers: (() => void)[] = [];

class Watched {
  private told: (() => void) | null = null;
  constructor(private readonly said: () => void) {}
  observe() {
    this.told = this.said;
    watchers.push(() => this.told?.());
  }
  disconnect() {
    this.told = null;
  }
  unobserve() {}
}

describe("the view settling on where the document was left", () => {
  beforeEach(() => {
    watchers.length = 0;
    vi.stubGlobal("ResizeObserver", Watched);
    vi.stubGlobal("requestAnimationFrame", (go: () => void) => {
      go();
      return 1;
    });
    vi.stubGlobal("cancelAnimationFrame", () => {});
  });

  // A sheet only scrolls as far as what it holds, the same as a real one.
  const sheet = (tall: number) => {
    const room = document.createElement("div");
    Object.defineProperty(room, "scrollHeight", { value: tall, writable: true });
    Object.defineProperty(room, "clientHeight", { value: 500, writable: true });
    let top = 0;
    Object.defineProperty(room, "scrollTop", {
      get: () => top,
      set: (want: number) => {
        top = Math.max(0, Math.min(want, room.scrollHeight - room.clientHeight));
      },
      configurable: true,
    });
    return room;
  };

  const grow = (room: HTMLElement, tall: number) => {
    Object.defineProperty(room, "scrollHeight", { value: tall, writable: true });
    for (const one of watchers) one();
  };

  const reaching = (room: HTMLElement, dom: HTMLElement) =>
    settles({
      at: () => room,
      want: 3_000,
      done: () => {},
      put: () => {},
      watch: () => dom,
    });

  it("gives up the moment somebody types, however short the text still is", () => {
    const room = sheet(600);
    const dom = document.createElement("div");
    reaching(room, dom);

    // Not there yet: the body is still rendering, so it holds on and keeps watching.
    expect(room.scrollTop).toBe(100);

    dom.dispatchEvent(new Event("beforeinput", { bubbles: true }));
    // The code blocks finish highlighting and the sheet grows under the person writing.
    grow(room, 4_000);

    expect(room.scrollTop).toBe(100);
  });

  it("still reaches where the reader left off when nobody is typing", () => {
    const room = sheet(600);
    const dom = document.createElement("div");
    reaching(room, dom);
    expect(room.scrollTop).toBe(100);

    grow(room, 4_000);

    expect(room.scrollTop).toBe(3_000);
  });
});
