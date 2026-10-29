import { describe, expect, it, vi } from "vitest";
import { heads } from "../src/heads";

const page = (html: string): Document =>
  new DOMParser().parseFromString(`<article class="page">${html}</article>`, "text/html");

describe("the headings of a page", () => {
  it("lists each heading with its level and words, leaving the title out", () => {
    const found = heads(
      page(
        '<h1 data-title>Plan</h1><h2 id="a">Uno</h2><p>x</p><h3 id="b">Dos</h3><div><h2>Dentro</h2></div>',
      ),
    );
    expect(found.map(({ key, level, text }) => ({ key, level, text }))).toEqual([
      { key: "0:a", level: 2, text: "Uno" },
      { key: "1:b", level: 3, text: "Dos" },
    ]);
  });

  it("opens with the properties when the page starts with them, and goes to each", () => {
    const drawn = page('<div class="front">p</div><h2>Uno</h2>');
    const front = drawn.querySelector<HTMLElement>(".front") as HTMLElement;
    const heading = drawn.querySelector<HTMLElement>("h2") as HTMLElement;
    const clicked = vi.fn();
    front.addEventListener("click", clicked);
    heading.scrollIntoView = vi.fn();
    const [opening, first] = heads(drawn);
    expect(opening).toMatchObject({ key: "front", level: 0, text: "" });
    opening.go();
    expect(clicked).toHaveBeenCalledOnce();
    first.go();
    expect(heading.scrollIntoView).toHaveBeenCalledWith({ block: "start" });
  });

  it("has no properties to open when they are not the first thing", () => {
    expect(heads(page('<p>x</p><div class="front">p</div>'))).toEqual([]);
  });
});
