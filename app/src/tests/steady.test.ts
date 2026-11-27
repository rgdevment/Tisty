import { describe, expect, it } from "vitest";
import { steady } from "../steady";

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

describe("keeping the same answer when it is asked about again", () => {
  it("hands back the very same object it is given twice", () => {
    const one = { docs: [{ id: "01A" }] };

    expect(steady(one, one)).toBe(one);
  });

  it("compares plain values and nothing by what they spell", () => {
    expect(steady(null, null)).toBeNull();
    expect(steady("a", "a")).toBe("a");
    expect(steady(3, 4)).toBe(4);
    expect(steady<string | null>(null, "a")).toBe("a");
    expect(steady<string | null>("null", null)).toBeNull();
  });

  it("does not make a later, different answer equal because the old one was weighed before", () => {
    const was = { docs: [{ id: "01A", title: "Uno" }] };
    expect(steady(was, { docs: [{ id: "01A", title: "Uno" }] })).toBe(was);

    const later = { docs: [{ id: "01A", title: "Dos" }] };
    expect(steady(was, later)).toBe(later);
    expect(steady(later, { docs: [{ id: "01A", title: "Uno" }] })).not.toBe(later);
  });
});
