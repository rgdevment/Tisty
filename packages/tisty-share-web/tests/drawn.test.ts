import { afterEach, describe, expect, it, vi } from "vitest";
import { linkedOut, pictured, titleOf, unasked } from "../src/drawn";

const pageOf = (html: string): Document => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = `<article class="page">${html}</article>`;
  return page;
};

const png = { bytes: new Uint8Array([137, 80, 78, 71]), media: "image/png" };

afterEach(() => vi.unstubAllGlobals());

describe("a picture the text names", () => {
  it("is set aside before the frame reads it, so the browser never asks for it", () => {
    const html = '<p><img src="tisty:picture/0" alt="Uno"><img src="https://x.test/a.png"></p>';
    expect(unasked(html)).toBe(
      '<p><img data-picture="0" alt="Uno"><img src="https://x.test/a.png"></p>',
    );
  });

  it("shows its own bytes, from the envelope, by its place", () => {
    const made: Blob[] = [];
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: (blob: Blob) => {
        made.push(blob);
        return `blob:${made.length}`;
      },
    });
    const page = pageOf('<img data-picture="1"><img data-picture="0">');
    const urls = pictured(page, [png, { ...png, media: "image/webp" }]);
    const shown = [...page.querySelectorAll("img")].map((img) => img.getAttribute("src"));
    expect(shown).toEqual(["blob:1", "blob:2"]);
    expect(urls).toEqual(["blob:1", "blob:2"]);
    expect(made.map((blob) => blob.type)).toEqual(["image/webp", "image/png"]);
  });

  it("that the envelope does not carry stays empty", () => {
    const page = pageOf('<img data-picture="3"><img data-picture="x">');
    expect(pictured(page, [png])).toEqual([]);
    expect(page.querySelector("img[src]")).toBeNull();
  });
});

describe("a link in the document", () => {
  it("out of it opens apart and takes nothing of the page", () => {
    const page = pageOf(
      '<a href="https://x.test">x</a><a href="mailto:a@x.test">m</a><a href="#arriba">a</a><a class="card" data-doc="d1">d</a>',
    );
    linkedOut(page);
    const said = [...page.querySelectorAll("a")].map((a) => [a.target, a.rel]);
    expect(said).toEqual([
      ["_blank", "noopener noreferrer"],
      ["_blank", "noopener noreferrer"],
      ["", ""],
      ["", ""],
    ]);
  });
});

describe("the document's title", () => {
  it("is its first line, as the page draws it", () => {
    expect(titleOf(pageOf("<h1>Docker en WSL2</h1><p>Texto</p>"))).toBe("Docker en WSL2");
    expect(titleOf(pageOf("<p>Una nota corta</p><h2>Después</h2>"))).toBe("Una nota corta");
  });

  it("comes after the front matter when the text has one", () => {
    const page = pageOf('<dl class="front"><div><dt>tags</dt></div></dl><h1>Guía</h1>');
    expect(titleOf(page)).toBe("Guía");
  });

  it("is empty when the first block is no line of words", () => {
    expect(titleOf(pageOf("<figure class='code'><pre><code>x</code></pre></figure>"))).toBe("");
  });
});
