import { beforeEach, describe, expect, it } from "vitest";
import { tabbedInside } from "../ui/heldFocus";

const button = (said: string): HTMLButtonElement => {
  const one = document.createElement("button");
  one.textContent = said;
  return one;
};

describe("where Tab goes inside a box", () => {
  const box = document.createElement("div");
  const stop = document.createElement("figure");
  const outside = button("behind");
  const [first, middle, last] = [button("first"), button("middle"), button("last")];

  beforeEach(() => {
    document.body.replaceChildren(outside, box);
    stop.tabIndex = -1;
    box.replaceChildren(stop, first, middle, last);
  });

  it("brings the focus back in from outside the box", () => {
    outside.focus();

    expect(tabbedInside(box, false)).toBe(first);
    expect(tabbedInside(box, true)).toBe(last);
  });

  it("comes round from either end, through the stop when there is one", () => {
    last.focus();
    expect(tabbedInside(box, false)).toBe(first);
    expect(tabbedInside(box, false, stop)).toBe(stop);

    first.focus();
    expect(tabbedInside(box, true)).toBe(last);
    expect(tabbedInside(box, true, stop)).toBe(stop);

    stop.focus();
    expect(tabbedInside(box, false, stop)).toBe(first);
    expect(tabbedInside(box, true, stop)).toBe(last);
  });

  it("leaves Tab to the window between the ends, and does nothing in an empty box", () => {
    middle.focus();
    expect(tabbedInside(box, false, stop)).toBeNull();

    box.replaceChildren();
    expect(tabbedInside(box, false)).toBeNull();
  });
});
