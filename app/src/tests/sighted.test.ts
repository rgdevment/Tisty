import { describe, expect, it } from "vitest";
import { carried, firstInSight, holding } from "../ui/sighted";

const scroller = () => {
  const room = document.createElement("div");
  let top = 0;
  const shape = { tall: 2000 };
  Object.defineProperty(room, "scrollTop", {
    get: () => top,
    set: (asked: number) => {
      top = Math.max(0, Math.min(asked, shape.tall - 500));
    },
  });
  Object.defineProperty(room, "scrollHeight", { get: () => shape.tall });
  return { room, shape };
};

const nextFrame = () => new Promise((done) => requestAnimationFrame(() => done(null)));

describe("holding a moved note in view", () => {
  it("reaches the place, reaches it again when the page shrank, and lets go when the person scrolls", async () => {
    const { room, shape } = scroller();
    let want = 900;
    const stop = holding(room, document.createElement("div"), () => want);
    await nextFrame();
    expect(room.scrollTop).toBe(900);
    shape.tall = 1200;
    room.scrollTop = 900;
    want = 650;
    room.dispatchEvent(new Event("scroll"));
    expect(room.scrollTop).toBe(650);
    room.scrollTop = 100;
    room.dispatchEvent(new Event("scroll"));
    want = 400;
    room.dispatchEvent(new Event("scroll"));
    expect(room.scrollTop).toBe(100);
    stop();
  });
});

const blocks = (sizes: [number, number][]) =>
  sizes.map(([top, height]) => {
    const one = document.createElement("p");
    one.getBoundingClientRect = () => ({ top, bottom: top + height, height }) as DOMRect;
    return one;
  });

describe("the block that keeps a page in place", () => {
  it("is the first one reaching below the top of the view, found by halves", () => {
    const page = blocks([
      [0, 40],
      [50, 40],
      [100, 40],
      [150, 40],
      [200, 40],
    ]);
    expect(firstInSight(page, 120)).toBe(page[2]);
    expect(firstInSight(page, -10)).toBe(page[0]);
    expect(firstInSight(page, 500)).toBeNull();
  });

  it("is never a block that takes no room", () => {
    const page = blocks([
      [0, 40],
      [45, 0],
      [50, 40],
    ]);
    expect(firstInSight(page, 44)).toBe(page[2]);
  });

  it("follows a place still waiting into words changed before or after it", () => {
    const before = "Uno.\n\nDos.\n\nTres.\n";
    const at = new TextEncoder().encode("Uno.\n\n").length;
    expect(carried(before, `Cero.\n\n${before}`, at)).toBe(at + "Cero.\n\n".length);
    expect(carried(before, `${before}\nCuatro.\n`, at)).toBe(at);
    expect(carried("ñu.\n\nDos.\n", "ñu y más.\n\nDos.\n", "ñu.\n\n".length + 1)).toBe(
      new TextEncoder().encode("ñu y más.\n\n").length,
    );
  });
});
