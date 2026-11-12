import { describe, expect, it } from "vitest";
import { settles } from "../ui/seeking";
import { carried, firstInSight } from "../ui/sighted";

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

const blocks = (sizes: [number, number][]) =>
  sizes.map(([top, height]) => {
    const one = document.createElement("p");
    one.getBoundingClientRect = () => ({ top, bottom: top + height, height }) as DOMRect;
    return one;
  });

describe("holding a moved note in view", () => {
  it("reaches the place on every frame, whatever the page does, until the person takes the view", async () => {
    const { room, shape } = scroller();
    const inside = document.implementation.createHTMLDocument();
    let want = 900;
    let ended = false;
    settles({
      at: () => room,
      want: () => want,
      done: () => {
        ended = true;
      },
      put: () => {},
      keeps: { waits: 4000, inside: () => inside },
    });
    await nextFrame();
    expect(room.scrollTop).toBe(900);
    want = 700;
    await nextFrame();
    expect(room.scrollTop).toBe(700);
    shape.tall = 1100;
    want = 650;
    room.scrollTop = 600;
    room.dispatchEvent(new Event("scroll"));
    await nextFrame();
    expect(room.scrollTop).toBe(600);
    expect(ended).toBe(false);
    inside.dispatchEvent(new Event("wheel"));
    expect(ended).toBe(true);
    want = 100;
    await nextFrame();
    expect(room.scrollTop).toBe(600);
  });
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
