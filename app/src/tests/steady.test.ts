import { describe, expect, it } from "vitest";
import { steady } from "../App";

describe("keeping the same list when nothing about it changed", () => {
  it("hands back what it was given before, so nobody sees a new list", () => {
    const was = { folders: [], docs: [{ id: "01A", file: "a-1", title: "Uno" }] };
    const found = { folders: [], docs: [{ id: "01A", file: "a-1", title: "Uno" }] };

    expect(steady(was, found)).toBe(was);
  });

  it("hands back the new one as soon as a single word differs", () => {
    const was = { folders: [], docs: [{ id: "01A", file: "a-1", title: "Uno" }] };
    const found = { folders: [], docs: [{ id: "01A", file: "a-1", title: "Dos" }] };

    expect(steady(was, found)).toBe(found);
  });

  it("notices what was added and what was taken away", () => {
    const one = { folders: [], docs: [{ id: "01A", file: "a-1", title: "Uno" }] };
    const two = { folders: [], docs: [] };

    expect(steady(one, two)).toBe(two);
    expect(steady(two, one)).toBe(one);
  });
});
