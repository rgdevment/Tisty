import { afterEach, describe, expect, it, vi } from "vitest";
import { sideways } from "../ui/sideways";

const page = (widths: [number, number][]) => {
  const made = document.implementation.createHTMLDocument("");
  made.body.innerHTML = `<div class="page">${widths
    .map((_, at) => `<div class="table" data-at="${at}-${at + 1}"><table></table></div>`)
    .join("")}</div>`;
  made.querySelectorAll<HTMLElement>(".table").forEach((box, at) => {
    Object.defineProperty(box, "scrollWidth", { get: () => widths[at][0], configurable: true });
    Object.defineProperty(box, "clientWidth", { get: () => widths[at][1] });
  });
  return made;
};

afterEach(() => vi.unstubAllGlobals());

describe("a table that scrolls sideways", () => {
  it("takes a keyboard stop, named as a table, and one that fits takes none", () => {
    const made = page([
      [900, 440],
      [440, 440],
    ]);
    expect(sideways(made)).toEqual([]);
    const [wide, fits] = made.querySelectorAll<HTMLElement>(".table");

    expect(wide.getAttribute("tabindex")).toBe("0");
    expect(wide.getAttribute("role")).toBe("region");
    expect(wide.getAttribute("aria-label")).toBe("Table");
    expect(fits.hasAttribute("tabindex")).toBe(false);
    expect(fits.hasAttribute("role")).toBe(false);
  });

  it("is looked at again when its box changes size, and let go when the page is undressed", () => {
    const seen: (() => void)[] = [];
    const disconnect = vi.fn();
    const made = page([[900, 440]]);
    Object.defineProperty(made, "defaultView", {
      value: {
        ResizeObserver: class {
          constructor(called: () => void) {
            seen.push(called);
          }
          observe() {}
          disconnect = disconnect;
        },
      },
    });
    const box = made.querySelector<HTMLElement>(".table") as HTMLElement;
    const [stop] = sideways(made);
    expect(box.getAttribute("tabindex")).toBe("0");

    Object.defineProperty(box, "scrollWidth", { get: () => 440 });
    seen[0]();
    expect(box.hasAttribute("tabindex")).toBe(false);
    expect(box.hasAttribute("aria-label")).toBe(false);

    stop();
    expect(disconnect).toHaveBeenCalled();
  });
});
