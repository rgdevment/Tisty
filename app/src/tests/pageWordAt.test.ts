import { afterEach, describe, expect, it } from "vitest";
import { wordAt } from "../ui/pageWordAt";

type Caret = { offsetNode: Node; offset: number };
const doc = document as unknown as {
  caretPositionFromPoint?: (x: number, y: number) => Caret | null;
  caretRangeFromPoint?: (x: number, y: number) => Range | null;
};

afterEach(() => {
  delete doc.caretPositionFromPoint;
  delete doc.caretRangeFromPoint;
  document.body.innerHTML = "";
});

const pageOf = (html: string) => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
};

const pointAt = (html: string, selector: string, offset: number) => {
  pageOf(html);
  const node = document.querySelector(selector)?.firstChild as Node;
  doc.caretPositionFromPoint = () => ({ offsetNode: node, offset });
  return wordAt(document, 1, 1);
};

describe("the word under a point of the page", () => {
  it("names a plain word and its bytes in the Markdown", () => {
    const html = '<p><span data-src="10-20">hola mundo</span></p>';
    expect(pointAt(html, "span", 7)).toEqual({ said: "mundo", at: [15, 20] });
    expect(pointAt(html, "span", 0)).toEqual({ said: "hola", at: [10, 14] });
  });

  it("counts UTF-8 bytes for a word after accented letters", () => {
    const html = '<p><span data-src="10-24">añoño prueva</span></p>';
    expect(pointAt(html, "span", 9)).toEqual({ said: "prueva", at: [18, 24] });
    expect(pointAt(html, "span", 2)).toEqual({ said: "añoño", at: [10, 17] });
  });

  it("trims apostrophes at the edges and keeps inner ones", () => {
    const quoted = "<p><span data-src=\"0-9\">'dont' it</span></p>";
    expect(pointAt(quoted, "span", 3)).toEqual({ said: "dont", at: [1, 5] });
    const inner = '<p><span data-src="0-7">don’t</span></p>';
    expect(pointAt(inner, "span", 2)).toEqual({ said: "don’t", at: [0, 7] });
  });

  it("falls back to caretRangeFromPoint when there is no caretPositionFromPoint", () => {
    pageOf('<p><span data-src="0-4">hola</span></p>');
    const node = document.querySelector("span")?.firstChild as Node;
    doc.caretRangeFromPoint = () => {
      const range = document.createRange();
      range.setStart(node, 2);
      return range;
    };
    expect(wordAt(document, 1, 1)).toEqual({ said: "hola", at: [0, 4] });
  });

  it("names nothing when the browser can say no place", () => {
    pageOf('<p><span data-src="0-4">hola</span></p>');
    expect(wordAt(document, 1, 1)).toBeNull();
    doc.caretPositionFromPoint = () => null;
    expect(wordAt(document, 1, 1)).toBeNull();
  });

  it("names nothing on a space or punctuation between words", () => {
    expect(pointAt('<p><span data-src="0-9">uno, . dos</span></p>', "span", 4)).toBeNull();
    expect(pointAt('<p><span data-src="0-4">a  b</span></p>', "span", 2)).toBeNull();
  });

  it("names nothing for a run of only apostrophes or digits", () => {
    expect(pointAt("<p><span data-src=\"0-3\">'''</span></p>", "span", 1)).toBeNull();
    expect(pointAt('<p><span data-src="0-3">123</span></p>', "span", 1)).toBeNull();
  });

  it("names nothing inside code, a link, a formula or a code block", () => {
    const inCode = '<p><code><span data-src="0-4">hola</span></code></p>';
    const inLink = '<p><a href="#"><span data-src="0-4">hola</span></a></p>';
    const inMath = '<p><span class="math"><span data-src="0-4">hola</span></span></p>';
    const inPre = '<pre><span data-src="0-4">hola</span></pre>';
    expect(pointAt(inCode, "span", 2)).toBeNull();
    expect(pointAt(inLink, "span", 2)).toBeNull();
    expect(pointAt(inMath, "span span", 2)).toBeNull();
    expect(pointAt(inPre, "span", 2)).toBeNull();
  });

  it("names nothing when the span's bytes are not its Markdown span", () => {
    expect(pointAt('<p><span data-src="0-7">hola</span></p>', "span", 2)).toBeNull();
    expect(pointAt('<p><span data-src="0-5">año</span></p>', "span", 2)).toBeNull();
  });

  it("names nothing for a text that shares its span or has no span with a place", () => {
    expect(pointAt('<p><span data-src="0-8">hola <b>uno</b></span></p>', "span", 2)).toBeNull();
    expect(pointAt("<p>hola</p>", "p", 2)).toBeNull();
  });

  it("names nothing outside the page", () => {
    document.body.innerHTML = '<div><span data-src="0-4">hola</span></div>';
    const node = document.querySelector("span")?.firstChild as Node;
    doc.caretPositionFromPoint = () => ({ offsetNode: node, offset: 1 });
    expect(wordAt(document, 1, 1)).toBeNull();
  });

  it("names nothing when the point is on an element rather than text", () => {
    pageOf('<p><span data-src="0-4">hola</span></p>');
    const element = document.querySelector("p") as Node;
    doc.caretPositionFromPoint = () => ({ offsetNode: element, offset: 0 });
    expect(wordAt(document, 1, 1)).toBeNull();
  });
});
