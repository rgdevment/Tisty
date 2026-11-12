import { describe, expect, it } from "vitest";
import { carried, firstInSight } from "../ui/sighted";

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
