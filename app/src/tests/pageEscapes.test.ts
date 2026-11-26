import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "Uno \\*dos tres\n";
const PLACED =
  '<p data-at="0-15"><span data-src="0-4">Uno </span><span data-src="5-14">*dos tres</span></p>';

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
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  writing.drawn();
  return () => text;
};

const deletedOverStar = (inputType: string, caret: number) => {
  const words = document.querySelector('[data-src="5-14"]')?.firstChild as Text;
  document.getSelection()?.collapse(words, caret);
  const reach = document.createRange();
  reach.setStart(words, 0);
  reach.setEnd(words, 1);
  const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
  document.querySelector(".page")?.dispatchEvent(event);
};

describe("a mark written as a word, with the backslash that keeps it one", () => {
  it("goes whole with Backspace, so no backslash is left to show", () => {
    const text = opened();
    deletedOverStar("deleteContentBackward", 1);
    expect(text()).toBe("Uno dos tres\n");
  });

  it("goes whole with Delete as well", () => {
    const text = opened();
    deletedOverStar("deleteContentForward", 0);
    expect(text()).toBe("Uno dos tres\n");
  });
});
