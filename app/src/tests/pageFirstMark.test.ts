import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (body: string, placed: string) => {
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
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  writing.drawn();
  return () => text;
};

const backspace = () => {
  const event = new KeyboardEvent("keydown", { key: "Backspace", bubbles: true, cancelable: true });
  document.querySelector(".page")?.dispatchEvent(event);
  return event;
};

describe("Backspace at the very start of a document that opens with a list", () => {
  it("takes the first item's mark away by itself, as WebKit sends no key there", () => {
    const text = opened(
      "- uno\n\nFin.\n",
      '<ul data-at="0-6"><li data-at="0-6"><span data-src="2-5">uno</span></li></ul><p data-at="7-12"><span data-src="7-11">Fin.</span></p>',
    );
    document
      .getSelection()
      ?.collapse(document.querySelector('[data-src="2-5"]')?.firstChild as Text, 0);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("uno\n\nFin.\n");
  });

  it("leaves Backspace to the browser anywhere past the first line", () => {
    const text = opened(
      "Antes\n\n- uno\n",
      '<p data-at="0-6"><span data-src="0-5">Antes</span></p><ul data-at="7-13"><li data-at="7-13"><span data-src="9-12">uno</span></li></ul>',
    );
    document
      .getSelection()
      ?.collapse(document.querySelector('[data-src="9-12"]')?.firstChild as Text, 0);
    expect(backspace().defaultPrevented).toBe(false);
    expect(text()).toBe("Antes\n\n- uno\n");
  });
});
