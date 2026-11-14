import { describe, expect, it } from "vitest";
import { pinMarkup, pinned } from "../src/ideas";

const drawn = (html: string) => {
  const page = document.implementation.createHTMLDocument();
  page.body.innerHTML = `<article class="page">${html}</article>`;
  pinned(page, "es");
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

  it("says its date and time above its words, and wears the board's own pin when pinned", () => {
    const notes = drawn(
      Array.from(
        { length: 12 },
        (_, n) =>
          `<blockquote class="callout" data-callout="idea" data-when="2026-11-12 09:${String(n).padStart(2, "0")}">x</blockquote>`,
      ).join(""),
    );
    expect(notes[0].dataset.said).toMatch(/^12 nov\.? · 09:00$/);
    const pinnedOnes = notes.filter((one) => one.dataset.hold === "pin");
    expect(pinnedOnes.length).toBeGreaterThan(0);
    for (const one of pinnedOnes)
      expect(one.querySelectorAll(":scope > svg.sticky-pin")).toHaveLength(1);
    for (const one of notes.filter((note) => note.dataset.hold !== "pin"))
      expect(one.querySelector("svg")).toBeNull();
  });

  it("leans only a little, and leaves any other quote alone", () => {
    const two = (n: number) => String(n).padStart(2, "0");
    const stuck = Array.from(
      { length: 40 },
      (_, n) =>
        `<blockquote class="callout" data-callout="idea" data-when="2026-11-${two((n % 28) + 1)} 10:${two(n)}">x</blockquote>`,
    ).join("");
    const notes = drawn(`${stuck}<blockquote class="callout" data-callout="note">y</blockquote>`);
    const note = notes.pop();
    const leans = notes.map((one) =>
      Math.abs(Number.parseFloat(one.style.getPropertyValue("--tilt"))),
    );
    expect(Math.max(...leans)).toBeLessThanOrEqual(1.75);
    expect(new Set(notes.map((one) => one.dataset.hold)).size).toBe(4);
    expect(note?.dataset.hold).toBeUndefined();
  });

  it("wears a pin whose names keep only what an id may hold, whatever it is handed", () => {
    const held = document.createElement("span");
    held.innerHTML = pinMarkup('pin-:r1:"><img src=x onerror=alert(1)>');

    expect(held.querySelector("img")).toBeNull();
    expect(held.querySelector("radialGradient")?.id).toBe("pin-r1imgsrcxonerroralert1-cap");
    expect(held.querySelector("ellipse[fill]")?.getAttribute("fill")).toBe(
      "url(#pin-r1imgsrcxonerroralert1-cap)",
    );
  });
});
