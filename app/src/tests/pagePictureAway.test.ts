import { afterEach, describe, expect, it } from "vitest";
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

const opened = () => {
  let text = BODY;
  document.body.innerHTML = `<article class="page">${PLACED}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  writing.drawn();
  return () => text;
};

const deleted = (inputType: string, reach: Range) => {
  const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
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
