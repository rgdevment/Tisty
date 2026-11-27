import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const A = '<p data-at="0-2"><span data-src="0-1">A</span></p>';
const BLANK =
  '<p class="blank" data-at="3-10"><span data-src="3-9" contenteditable="false">&nbsp;</span></p>';
const B = '<p data-at="11-13"><span data-src="11-12">B</span></p>';
const KEPT = "A\n\n&nbsp;\n\nB\n";

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
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  const redrawn = (html: string) => {
    document.body.innerHTML = `<article class="page">${html}</article>`;
    writing?.drawn();
  };
  return { text: () => text, redrawn };
};

const wordsOf = (said: string) => {
  const walk = document.createTreeWalker(
    document.querySelector(".page") as Node,
    NodeFilter.SHOW_TEXT,
  );
  for (let node = walk.nextNode(); node; node = walk.nextNode())
    if ((node as Text).data.includes(said)) return node as Text;
  throw new Error(`no ${said} on the page`);
};

const asked = (inputType: string, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  document.querySelector(".page")?.dispatchEvent(event);
  return event.defaultPrevented;
};

const pressed = (key: string) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  document.querySelector(".page")?.dispatchEvent(event);
  return event.defaultPrevented;
};

describe("a blank line kept on the page", () => {
  it("is written by a second Enter on a blank line, and the caret goes on a new blank line below", () => {
    const { text, redrawn } = opened("A", '<p data-at="0-1"><span data-src="0-1">A</span></p>');
    document.getSelection()?.collapse(wordsOf("A"), 1);
    asked("insertParagraph");
    expect(text()).toBe("A\n\n");
    writing?.placeAt(3);
    asked("insertParagraph");
    expect(text()).toBe("A\n\n&nbsp;\n\n");
    redrawn(`${A}${BLANK}`);
    asked("insertText", "x");
    expect(text()).toBe("A\n\n&nbsp;\n\nx");
  });

  it("is replaced by the first key typed in it", () => {
    const { text } = opened(KEPT, `${A}${BLANK}${B}`);
    document.getSelection()?.collapse(wordsOf(" "), 0);
    asked("insertText", "x");
    expect(text()).toBe("A\n\nx\n\nB\n");
  });

  it("goes whole with Backspace at the start of the line below, and comes back with undo", () => {
    const { text } = opened(KEPT, `${A}${BLANK}${B}`);
    document.getSelection()?.collapse(wordsOf("B"), 0);
    asked("deleteContentBackward");
    expect(text()).toBe("A\n\nB\n");
    asked("historyUndo");
    expect(text()).toBe(KEPT);
  });

  it("goes whole with Delete at the end of the line above", () => {
    const { text } = opened(KEPT, `${A}${BLANK}${B}`);
    document.getSelection()?.collapse(wordsOf("A"), 1);
    asked("deleteContentForward");
    expect(text()).toBe("A\n\nB\n");
  });

  it("is dropped from the end of the page when the caret leaves for the start of the line above", () => {
    const { text } = opened("A\n\n&nbsp;\n\n", `${A}${BLANK}`);
    document.getSelection()?.collapse(wordsOf("\u00a0"), 0);
    pressed("ArrowUp");
    document.getSelection()?.collapse(wordsOf("A"), 0);
    document.dispatchEvent(new Event("selectionchange"));
    expect(text()).toBe("A\n");
  });

  it("is dropped from the end of the page when the caret leaves for the end of the line above", () => {
    const { text } = opened("A\n\n&nbsp;\n\n", `${A}${BLANK}`);
    document.getSelection()?.collapse(wordsOf(" "), 0);
    pressed("ArrowUp");
    document.getSelection()?.collapse(wordsOf("A"), 1);
    document.dispatchEvent(new Event("selectionchange"));
    expect(text()).toBe("A\n");
  });

  it.each([
    ["at the top of the page", '<p data-at="3-3"></p>', "A\n\n&nbsp;\n\n"],
    ["inside a quote", '<blockquote data-at="3-3"><p data-at="3-3"></p></blockquote>', "A\n\n"],
  ])("is written by Enter on a blank line %s only where it is not nested", (_, line, written) => {
    const { text } = opened("A\n\n", `${A}${line}`);
    writing?.placeAt(3);
    asked("insertParagraph");
    expect(text()).toBe(written);
  });
});
