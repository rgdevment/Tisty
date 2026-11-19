import { beforeEach, describe, expect, it } from "vitest";
import { tabbedInside } from "../ui/heldFocus";

const button = (said: string): HTMLButtonElement => {
  const one = document.createElement("button");
  one.textContent = said;
  return one;
};

describe("where Tab goes inside a box", () => {
  const box = document.createElement("div");
  const outside = button("behind");
  const [first, middle, last] = [button("first"), button("middle"), button("last")];

  beforeEach(() => {
    document.body.replaceChildren(outside, box);
    box.replaceChildren(first, middle, last);
  });

  it("brings the focus back in from outside the box", () => {
    outside.focus();

    expect(tabbedInside(box, false)).toBe(first);
    expect(tabbedInside(box, true)).toBe(last);
  });

  it("comes round from either end", () => {
    last.focus();
    expect(tabbedInside(box, false)).toBe(first);

    first.focus();
    expect(tabbedInside(box, true)).toBe(last);
  });

  it("leaves Tab to the window between the ends, and does nothing in an empty box", () => {
    middle.focus();
    expect(tabbedInside(box, false)).toBeNull();

    box.replaceChildren();
    expect(tabbedInside(box, false)).toBeNull();
  });
});
