import { afterEach, describe, expect, it, vi } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "Una imagen ![mini](mini.png) entre palabras.\n";
const PLACED =
  '<p data-at="0-45"><span data-src="0-11">Una imagen </span>' +
  '<img alt="mini" data-src="11-28" contenteditable="false" data-kept="mini.png" hidden>' +
  '<span class="picture-away">Algo falló<button type="button" class="media-again">Reintentar</button></span>' +
  '<span data-src="28-44"> entre palabras.</span></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (body = BODY, placed = PLACED) => {
  let text = body;
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  writing.drawn();
  return () => text;
};

const deleted = (inputType: string, reach: Range | null) => {
  const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => (reach ? [reach] : []) });
  document.querySelector(".page")?.dispatchEvent(event);
};

describe("a key beside the spaces a picture leaves unshown", () => {
  it("takes the space with Delete before it, though the browser reaches past it to the picture", () => {
    const text = opened();
    const words = document.querySelector('[data-src="0-11"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, 10);
    const reach = document.createRange();
    reach.setStart(words, 11);
    reach.setEnd(document.querySelector("p") as HTMLParagraphElement, 1);
    deleted("deleteContentForward", reach);
    expect(text()).toBe("Una imagen![mini](mini.png) entre palabras.\n");
  });

  it("takes the space with Delete before it, though the browser reaches over the space and the picture", () => {
    const text = opened();
    const words = document.querySelector('[data-src="0-11"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, 10);
    const reach = document.createRange();
    reach.setStart(words, 11);
    reach.setEnd(document.querySelector("p") as HTMLParagraphElement, 2);
    deleted("deleteContentForward", reach);
    expect(text()).toBe("Una imagen![mini](mini.png) entre palabras.\n");
  });

  it("takes the space with Delete before it when the browser names no reach at all", () => {
    const text = opened();
    const words = document.querySelector('[data-src="0-11"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, 10);
    deleted("deleteContentForward", null);
    expect(text()).toBe("Una imagen![mini](mini.png) entre palabras.\n");
  });

  it("takes the picture with Backspace after the space, where the browser reaches back to it", () => {
    const text = opened();
    const words = document.querySelector('[data-src="28-44"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, 1);
    const said = document.querySelector("button")?.firstChild as Text;
    const reach = document.createRange();
    reach.setStart(said, said.length);
    reach.setEnd(words.parentElement as HTMLElement, 0);
    deleted("deleteContentBackward", reach);
    expect(text()).toBe("Una imagen entre palabras.\n");
  });
});

const pictured = (many: number) => {
  const lines = Array.from({ length: many }, (_, at) => at * 23);
  const body = lines.map(() => "Foto ![p](p.png) fin.\n\n").join("");
  const placed = lines
    .map(
      (at) =>
        `<p data-at="${at}-${at + 22}"><span data-src="${at}-${at + 5}">Foto </span>` +
        `<img alt="p" src="p.png" data-src="${at + 5}-${at + 16}" contenteditable="false">` +
        `<span data-src="${at + 16}-${at + 21}"> fin.</span></p>`,
    )
    .join("");
  return { body, placed, last: (many - 1) * 23 };
};

const encodedOnBackspace = (many: number) => {
  const { body, placed, last } = pictured(many);
  const text = opened(body, placed);
  const words = document.querySelector(`[data-src="${last + 16}-${last + 21}"]`)
    ?.firstChild as Text;
  document.getSelection()?.collapse(words, 0);
  const reach = document.createRange();
  reach.setStart(words, 0);
  reach.setEnd(words, 0);
  const encoding = vi.spyOn(TextEncoder.prototype, "encode");
  deleted("deleteContentBackward", reach);
  const count = encoding.mock.calls.length;
  encoding.mockRestore();
  expect(text().endsWith("Foto fin.\n\n")).toBe(true);
  writing?.stop();
  writing = null;
  return count;
};

describe("Backspace after a picture on a page with many", () => {
  it("reads the page's bytes as often with forty pictures as with one", () => {
    expect(encodedOnBackspace(40)).toBe(encodedOnBackspace(1));
  });
});
