import { beforeEach, describe, expect, it, vi } from "vitest";
import { CLIMBS_AT_MOST, lend, notWidget, SHORTEST, TALLEST, tall } from "../ui/widgeting";

beforeEach(() => {
  document.documentElement.removeAttribute("data-theme");
});

describe("the height of a widget", () => {
  it("stays between a sliver and a screen", () => {
    expect(tall(1)).toBe(SHORTEST);
    expect(tall(99999)).toBe(TALLEST);
    expect(tall(120.2)).toBe(121);
  });
});

describe("a widget", () => {
  const drawnFrom = (source: string) => {
    const drawn = document.createElement("div");
    document.body.append(drawn);
    return { drawn, lending: lend(drawn, source) };
  };

  it("is drawn in a frame where nothing can run, and asks nobody for anything", () => {
    const { drawn, lending } = drawnFrom('<div class="stat"><b>4</b></div>');
    const frame = drawn.querySelector("iframe") as HTMLIFrameElement;

    expect(frame.getAttribute("sandbox")).toBe("allow-same-origin");
    expect(frame.srcdoc).toContain('<div class="stat"><b>4</b></div>');
    expect(frame.srcdoc).toContain("default-src 'none'");
    expect(frame.srcdoc).not.toContain("<script");
    lending.drop();
  });

  it("comes dark in a dark window and follows the theme without being drawn again", () => {
    document.documentElement.setAttribute("data-theme", "dark");
    const { drawn, lending } = drawnFrom("<p>a</p>");
    const frame = drawn.querySelector("iframe") as HTMLIFrameElement;
    expect(frame.srcdoc).toContain('<html class="dark">');
    expect(frame.style.colorScheme).toBe("dark");

    document.documentElement.removeAttribute("data-theme");
    lending.retheme();
    expect(frame.style.colorScheme).toBe("light");
    lending.drop();
  });

  it("is never drawn in part: a block with anything else says what, and no frame is made", () => {
    const { drawn } = drawnFrom(
      '<p>a</p><script>alert(1)</script><link rel="preconnect" href="https://a.b">',
    );

    expect(drawn.querySelector("iframe")).toBeNull();
    expect(drawn.querySelector(".lit-refused")?.textContent).toContain("<script>");
    expect(drawn.querySelector(".lit-refused")?.textContent).toContain("<link>");
  });

  it("is told apart from what the browser would really make of it", () => {
    expect(notWidget('<div class="stat"><b>4</b></div>')).toBeNull();
    expect(notWidget("<p onclick=x>a</p>")).toContain("<p onclick>");
    expect(notWidget('<a href="https://a.b">a</a>')).toContain("<a>");
    expect(notWidget("<svg><script>a</script></svg>")).toContain("<script>");
  });

  it("says when it is too big to draw", () => {
    expect(notWidget(`<p>${"x".repeat(600_000)}</p>`)).toContain("KB");
  });

  it("grows to what it holds, and stops when each answer climbs by the same step", () => {
    const told: (() => void)[] = [];
    vi.stubGlobal(
      "ResizeObserver",
      class {
        constructor(tell: () => void) {
          told.push(tell);
        }
        observe() {}
        disconnect() {}
      },
    );
    const { drawn, lending } = drawnFrom("<p>a</p>");
    const frame = drawn.querySelector("iframe") as HTMLIFrameElement;
    const inside = document.implementation.createHTMLDocument();
    inside.body.innerHTML = '<main class="w"></main>';
    Object.defineProperty(frame, "contentDocument", { value: inside });
    const box = inside.querySelector("main.w") as HTMLElement;
    Object.defineProperty(box, "scrollHeight", {
      get: () => Number.parseFloat(frame.style.height) + 50,
    });

    frame.dispatchEvent(new Event("load"));
    expect(frame.style.height).toBe("90px");

    const reached = () => Number.parseFloat(frame.style.height);
    for (let step = 0; step < CLIMBS_AT_MOST + 3; step += 1) told[0]();
    expect(reached()).toBeLessThan(90 + 50 * (CLIMBS_AT_MOST + 3));
    lending.drop();
    vi.unstubAllGlobals();
  });
});
