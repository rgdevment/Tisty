import { describe, expect, it } from "vitest";
import { aloneOnLine, pictureGone } from "../ui/pictureGone";

const pageOf = (html: string) => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
  return document;
};

const gone = (text: string, nth = 0) => {
  const picture = document.querySelectorAll("img[data-src]")[nth];
  const span = picture ? pictureGone(text, picture) : null;
  return span ? text.slice(0, span.from) + text.slice(span.to) : null;
};

describe("a picture taken away", () => {
  it("takes its line and the blank line after it when it stands alone", () => {
    pageOf(
      '<p data-at="0-3">Uno</p><p data-at="5-16"><img data-src="5-16"></p><p data-at="18-21">Dos</p>',
    );
    expect(gone("Uno\n\n![a](a.png)\n\nDos\n")).toBe("Uno\n\nDos\n");
  });

  it("leaves a single blank line behind when it was the last block", () => {
    pageOf('<p data-at="0-3">Uno</p><p data-at="5-17"><img data-src="5-16"></p>');
    expect(gone("Uno\n\n![a](a.png)\n")).toBe("Uno\n");
  });

  it("leaves one space between the words it stood among", () => {
    pageOf('<p data-at="0-20">Dos <img data-src="4-15"> tres</p>');
    expect(gone("Dos ![b](b.png) tres\n")).toBe("Dos tres\n");
  });

  it("counts its place in bytes, as the core does, when the words before it are wider", () => {
    pageOf('<p data-at="0-22">Año <img data-src="5-16"> tres</p>');
    expect(gone("Año ![b](b.png) tres\n")).toBe("Año tres\n");
  });

  it("is alone on its line only in a paragraph with no words and nothing else drawn", () => {
    pageOf(
      '<p data-at="0-11"><img data-src="0-11"></p><p data-at="13-30">x <img data-src="15-26"></p><p data-at="32-60"><img data-src="32-43"><code data-src="44-50"></code></p>',
    );
    const [alone, worded, twice] = Array.from(document.querySelectorAll("p"));
    expect([aloneOnLine(alone), aloneOnLine(worded), aloneOnLine(twice)]).toEqual([
      true,
      false,
      false,
    ]);
  });
});
