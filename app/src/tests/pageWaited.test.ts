import { afterEach, describe, expect, it } from "vitest";
import { landedAfter, reachIn } from "../ui/pageEdits";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nUno dos tres 👍🏽\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-30"><span data-src="8-29">Uno dos tres 👍🏽</span></p>';

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
  return () => text;
};

const asked = (inputType: string, data: string | null = null) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

const words = () => document.querySelector('[data-src="8-29"]')?.firstChild as Text;

const chosen = (said: string) => {
  const at = words().data.indexOf(said);
  const range = document.createRange();
  range.setStart(words(), at);
  range.setEnd(words(), at + said.length);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
};

const caretAfter = (said: string) => {
  const at = words().data.indexOf(said) + said.length;
  document.getSelection()?.collapse(words(), at);
};

const whole = (text: string) =>
  !/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/.test(text);

describe("how far a word's delete reaches in the text", () => {
  it("takes one word of Chinese, not the whole run", () => {
    expect(reachIn("我们今天去公园", 7, "deleteWordBackward")).toEqual({ from: 5, to: 7 });
  });

  it("takes the word with the spaces after it, and stops at a comma", () => {
    expect(reachIn("uno dos  ", 9, "deleteWordBackward")).toEqual({ from: 4, to: 9 });
    expect(reachIn("uno, dos", 8, "deleteWordBackward")).toEqual({ from: 5, to: 8 });
    expect(reachIn("uno dos", 3, "deleteWordForward")).toEqual({ from: 3, to: 7 });
  });

  it("takes an emoji, a link or marks whole, never half their writing nor the word before", () => {
    expect(reachIn("tres 👍🏽", 9, "deleteWordBackward")).toEqual({ from: 5, to: 9 });
    expect(reachIn("ver [docs](https://x.com/a)", 27, "deleteWordBackward")).toEqual({
      from: 4,
      to: 27,
    });
    expect(reachIn("**fuerte** resto", 0, "deleteWordForward")).toEqual({ from: 0, to: 10 });
  });
});

describe("the caret after an edit widened to whole characters", () => {
  it("moves back with the words taken past the end", () => {
    const wanted = { from: 5, to: 6, insert: "" };
    expect(landedAfter(wanted, { ...wanted, to: 7 }, 9)).toBe(8);
    expect(landedAfter(wanted, { ...wanted, to: 7 }, 5)).toBe(5);
    expect(landedAfter(wanted, { ...wanted, from: 4 }, 5)).toBe(4);
  });
});

describe("a key whose browser names a place other than the caret", () => {
  it("writes at the caret shown, not at a stale place before it", () => {
    const text = opened();
    caretAfter("tres");
    const stale = document.createRange();
    stale.setStart(document.querySelector("h1") as Element, 0);
    const event = new InputEvent("beforeinput", {
      inputType: "insertText",
      data: "X",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(event, "getTargetRanges", { value: () => [stale] });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(text()).toBe("# Hola\n\nUno dos tresX 👍🏽\n");
  });
});

describe("keys that come before the page shows the last one", () => {
  it("take a character drawn from several back whole, never half of it", () => {
    const text = opened();
    caretAfter("👍🏽");
    asked("insertText", "a");
    asked("deleteContentBackward");
    asked("deleteContentBackward");
    expect(whole(text())).toBe(true);
    expect(text()).toBe("# Hola\n\nUno dos tres \n");
  });

  it("take the words back to the line's start for a line's delete, not the letter after", () => {
    const text = opened();
    caretAfter("dos");
    asked("insertText", "a");
    asked("deleteSoftLineBackward");
    expect(text()).toBe("# Hola\n\n tres 👍🏽\n");
  });

  it("take a word back for a word's delete", () => {
    const text = opened();
    caretAfter("tres");
    asked("insertText", "x");
    asked("deleteWordBackward");
    expect(text()).toBe("# Hola\n\nUno dos  👍🏽\n");
  });

  it("write where the last key left the caret, not over words chosen before it", () => {
    const text = opened();
    chosen("dos");
    asked("insertText", "X");
    asked("insertText", "Y");
    expect(text()).toBe("# Hola\n\nUno XY tres 👍🏽\n");
  });
});
