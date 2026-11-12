import { describe, expect, it } from "vitest";
import { pinned } from "../src/ideas";

const drawn = (html: string) => {
  const page = document.implementation.createHTMLDocument();
  page.body.innerHTML = `<article class="page">${html}</article>`;
  pinned(page);
  return Array.from(page.querySelectorAll<HTMLElement>("blockquote"));
};

const NOTE =
  '<blockquote class="callout" data-callout="idea" data-when="2026-11-12 09:12">x</blockquote>';

describe("a sticky note held on the sheet", () => {
  it("is held the same way every time for the same date and time, by tape or a pin", () => {
    const [once] = drawn(NOTE);
    const [again] = drawn(NOTE);
    expect(["pin", "tape", "corner-left", "corner-right"]).toContain(once.dataset.hold);
    expect(again.dataset.hold).toBe(once.dataset.hold);
    expect(again.dataset.pin).toBe(once.dataset.pin);
    expect(again.style.getPropertyValue("--tilt")).toBe(once.style.getPropertyValue("--tilt"));
  });

  it("leans only a little, and leaves any other quote alone", () => {
    const notes = drawn(
      Array.from(
        { length: 40 },
        (_, n) =>
          `<blockquote class="callout" data-callout="idea" data-when="2026-11-${String((n % 28) + 1).padStart(2, "0")} 10:${String(n).padStart(2, "0")}">x</blockquote>`,
      ).join("") + '<blockquote class="callout" data-callout="note">y</blockquote>',
    );
    const note = notes.pop();
    for (const one of notes)
      expect(Math.abs(Number.parseFloat(one.style.getPropertyValue("--tilt")))).toBeLessThanOrEqual(
        1.75,
      );
    expect(new Set(notes.map((one) => one.dataset.hold)).size).toBe(4);
    expect(note?.dataset.hold).toBeUndefined();
  });
});
