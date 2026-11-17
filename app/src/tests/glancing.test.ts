import { describe, expect, it, vi } from "vitest";
import type { Glance } from "../citing";
import { glancing } from "../ui/glancing";
import { went } from "../ui/paging";

const SAID: Glance = {
  title: "Viajes",
  heading: "Hoteles",
  html: "<p>El del mar.</p>",
  more: true,
};

const page = (html: string) => {
  const one = document.implementation.createHTMLDocument("");
  one.body.innerHTML = `<div class="page">${html}</div>`;
  return one;
};

const rest = async (on: Element) => {
  on.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
  await vi.advanceTimersByTimeAsync(500);
};

describe("a glance at the document a link leads to", () => {
  it("opens on a link to a heading after resting on it, and shuts when the pointer leaves", async () => {
    const at = page(`<p>Ver <a href="tisty:doc/dev_a-0002#hoteles">hoteles</a>.</p>`);
    const look = vi.fn(() => Promise.resolve(SAID));
    glancing(at, look);
    const link = at.querySelector("a") as HTMLAnchorElement;

    await rest(link);

    expect(look).toHaveBeenCalledWith("dev_a-0002", "hoteles");
    expect(at.querySelector(".glance-title")?.textContent).toBe("Viajes › Hoteles");
    expect(at.querySelector(".glance-body")?.innerHTML).toBe("<p>El del mar.</p>");
    expect(at.querySelector(".glance")?.hasAttribute("data-more")).toBe(true);

    link.dispatchEvent(new MouseEvent("mouseout", { bubbles: true, relatedTarget: at.body }));
    await vi.advanceTimersByTimeAsync(300);

    expect(at.querySelector(".glance")).toBeNull();
  });

  it("asks nothing of a card, a web link or a glance that never rested", async () => {
    const at = page(
      `<a class="card" href="tisty:doc/a">A</a><a href="https://example.com">web</a><a href="tisty:doc/b">B</a>`,
    );
    const look = vi.fn(() => Promise.resolve(SAID));
    glancing(at, look);

    await rest(at.querySelector(".card") as Element);
    await rest(at.querySelector("a[href^=https]") as Element);
    const passing = at.querySelector("a[href='tisty:doc/b']") as Element;
    passing.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
    passing.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    await vi.advanceTimersByTimeAsync(500);

    expect(look).not.toHaveBeenCalled();
    expect(at.querySelector(".glance")).toBeNull();
  });
});

describe("following a link to a heading of another document", () => {
  it("names the document and the heading apart", () => {
    const at = page(
      `<a href="tisty:doc/dev_a-0002#hoteles">h</a><a href="tisty:doc/dev_a-0003">d</a>`,
    );
    const [heading, plain] = Array.from(at.querySelectorAll("a"));

    expect(went(heading, false)).toEqual({ kind: "doc", id: "dev_a-0002", at: "hoteles" });
    expect(went(plain, false)).toEqual({ kind: "doc", id: "dev_a-0003" });
  });
});
