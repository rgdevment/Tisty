import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { quickly } from "../ui/quickly";

vi.mock("../glyphs", () => ({ markup: () => '<path d="M1 1"/>' }));

const onMenu = vi.fn();

beforeAll(() => {
  quickly(document, () => ({ onMenu }));
});

beforeEach(() => {
  onMenu.mockClear();
  document.body.innerHTML =
    '<article class="page"><p data-at="0-9"><span data-src="0-9">hola mundo</span></p>' +
    '<input id="field"><textarea id="area"></textarea><select id="pick"></select></article>';
  document.getSelection()?.removeAllRanges();
});

const asked = (target: Element | null, init: MouseEventInit = {}) => {
  const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true, ...init });
  target?.dispatchEvent(event);
  return event;
};

const keyed = (key: string, more: KeyboardEventInit = {}) =>
  document.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, ...more }));

const caretInWords = () => {
  const range = document.createRange();
  range.setStart(document.querySelector("span")?.firstChild as Node, 2);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
};

const boxOf = (box: Partial<DOMRect>) =>
  vi.spyOn(Range.prototype, "getBoundingClientRect").mockReturnValue({
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
    width: 0,
    height: 0,
    x: 0,
    y: 0,
    toJSON: () => ({}),
    ...box,
  });

describe("the menu of the page asked from fields and from the keyboard", () => {
  it("leaves the system's menu alone on an input, a textarea or a select", () => {
    for (const id of ["#field", "#area", "#pick"]) {
      const event = asked(document.querySelector(id));
      expect(event.defaultPrevented).toBe(false);
    }
    expect(onMenu).not.toHaveBeenCalled();
  });

  it("still takes over the menu on the page's own words", () => {
    const event = asked(document.querySelector("span"), { clientX: 3, clientY: 4 });
    expect(event.defaultPrevented).toBe(true);
    expect(onMenu).toHaveBeenCalledWith(expect.objectContaining({ x: 3, y: 4 }));
  });

  it("opens at the caret when the menu key was pressed just before", () => {
    const box = boxOf({ left: 40, bottom: 70, width: 0, height: 18 });
    caretInWords();
    keyed("ContextMenu");
    asked(document.querySelector("span"), { clientX: 1, clientY: 2 });
    expect(onMenu).toHaveBeenCalledWith(expect.objectContaining({ x: 40, y: 70 }));
    box.mockRestore();
  });

  it("opens at the caret for Shift+F10 too", () => {
    const box = boxOf({ left: 12, bottom: 34, width: 5, height: 0 });
    caretInWords();
    keyed("F10", { shiftKey: true });
    asked(document.querySelector("span"), { clientX: 1, clientY: 2 });
    expect(onMenu).toHaveBeenCalledWith(expect.objectContaining({ x: 12, y: 34 }));
    box.mockRestore();
  });

  it("opens where the pointer was when the box of the choice is empty", () => {
    const box = boxOf({});
    caretInWords();
    keyed("ContextMenu");
    asked(document.querySelector("span"), { clientX: 8, clientY: 9 });
    expect(onMenu).toHaveBeenCalledWith(expect.objectContaining({ x: 8, y: 9 }));
    box.mockRestore();
  });

  it("opens where the pointer was when another key came after the menu key", () => {
    const box = boxOf({ left: 40, bottom: 70, height: 18 });
    caretInWords();
    keyed("ContextMenu");
    keyed("a");
    asked(document.querySelector("span"), { clientX: 8, clientY: 9 });
    expect(onMenu).toHaveBeenCalledWith(expect.objectContaining({ x: 8, y: 9 }));
    box.mockRestore();
  });

  it("forgets the menu key once a menu was opened", () => {
    const box = boxOf({ left: 40, bottom: 70, height: 18 });
    caretInWords();
    keyed("ContextMenu");
    asked(document.querySelector("span"), { clientX: 1, clientY: 2 });
    asked(document.querySelector("span"), { clientX: 8, clientY: 9 });
    expect(onMenu).toHaveBeenLastCalledWith(expect.objectContaining({ x: 8, y: 9 }));
    box.mockRestore();
  });

  it("carries a word field, null when the point names none", () => {
    asked(document.querySelector("span"), { clientX: 1, clientY: 1 });
    expect(onMenu).toHaveBeenCalledWith(expect.objectContaining({ word: null }));
  });
});

describe("the menu of the page asked from the keyboard on a word", () => {
  it("spells the caret's word and offers the caret's block, wherever the event says it is", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-11"><span data-src="0-10">hola mundo</span></p></article>';
    caretInWords();
    const spy = boxOf({ left: 30, bottom: 40, height: 18 });
    keyed("ContextMenu");
    asked(document.querySelector("article"), { clientX: 0, clientY: 0 });
    spy.mockRestore();
    const [said] = onMenu.mock.calls[0];
    expect(said.word).toEqual({ said: "hola", at: [0, 4] });
    expect(said.block?.element).toBe(document.querySelector("p"));
    expect([said.x, said.y]).toEqual([30, 40]);
  });
});
