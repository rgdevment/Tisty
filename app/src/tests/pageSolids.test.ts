import { afterEach, describe, expect, it, vi } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";
import { unnavigated } from "../unnavigated";

const BODY = "Hola #etiqueta mundo\n";
const PLACED =
  '<p data-at="0-21"><span data-src="0-5">Hola </span>' +
  '<span class="tag" data-src="5-14" contenteditable="false">#etiqueta</span>' +
  '<span data-src="14-20"> mundo</span></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.removeEventListener("keydown", unnavigated);
  document.body.innerHTML = "";
});

const opened = () => {
  document.body.innerHTML = `<article class="page">${PLACED}</article>`;
  const sheet = document.querySelector(".page") as HTMLElement;
  Object.defineProperty(sheet, "isContentEditable", { value: true });
  document.addEventListener("keydown", unnavigated);
  writing = writtenOn(document, {
    text: () => BODY,
    write: () => {},
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
};

const pressed = (key: string, init: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  document.querySelector(".page")?.dispatchEvent(event);
  return event;
};

const tagWords = () => document.querySelector(".tag")?.firstChild as Text;
const wordsAfter = () => document.querySelector('[data-src="14-20"]')?.firstChild as Text;

const chosen = () => {
  const held = document.getSelection()?.getRangeAt(0);
  return held && [held.startContainer, held.startOffset, held.endContainer, held.endOffset];
};

describe("a delete key on a page wired as the frame wires it, with a tag in the choice", () => {
  for (const [key, init] of [
    ["Backspace", {}],
    ["Delete", {}],
    ["Backspace", { altKey: true }],
  ] as const) {
    it(`chooses the whole tag on ${init.altKey ? "Alt+" : ""}${key} and lets the engine delete it`, () => {
      opened();
      document.getSelection()?.setBaseAndExtent(tagWords(), 1, tagWords(), 5);
      const event = pressed(key, init);
      const line = document.querySelector("p");
      expect(chosen()).toEqual([line, 1, line, 2]);
      expect(event.defaultPrevented).toBe(false);
    });
  }

  it("takes the tag whole when the choice opens inside it and reaches the words after, never words alone", () => {
    opened();
    document.getSelection()?.setBaseAndExtent(wordsAfter(), 1, wordsAfter(), 4);
    pressed("Backspace");
    expect(chosen()).toEqual([wordsAfter(), 1, wordsAfter(), 4]);
    document.getSelection()?.setBaseAndExtent(tagWords(), 4, wordsAfter(), 3);
    expect(pressed("Backspace").defaultPrevented).toBe(false);
    expect(chosen()).toEqual([document.querySelector("p"), 1, wordsAfter(), 3]);
  });

  it("keeps a choice made backward into the tag pointing the way it was made", () => {
    opened();
    document.getSelection()?.setBaseAndExtent(wordsAfter(), 3, tagWords(), 4);
    pressed("Backspace");
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([wordsAfter(), 3]);
    expect([held?.focusNode, held?.focusOffset]).toEqual([document.querySelector("p"), 1]);
  });

  it("never takes the tag into words chosen only up to its edge", () => {
    opened();
    const hola = document.querySelector('[data-src="0-5"]')?.firstChild as Text;
    const line = document.querySelector("p");
    document.getSelection()?.setBaseAndExtent(hola, 1, tagWords(), 0);
    pressed("Backspace");
    expect(chosen()).toEqual([hola, 1, line, 1]);
    document.getSelection()?.setBaseAndExtent(tagWords(), tagWords().length, wordsAfter(), 3);
    pressed("Backspace");
    expect(chosen()).toEqual([line, 2, wordsAfter(), 3]);
  });

  it("tells a backward choice by where its ends lie, not by the nodes the engine names them with", () => {
    opened();
    const line = document.querySelector("p") as Element;
    const held = document.getSelection() as Selection;
    held.setBaseAndExtent(line, 3, tagWords(), 4);
    const named = document.createRange();
    named.setStart(tagWords(), 4);
    named.setEnd(wordsAfter(), wordsAfter().length);
    const asked = vi.spyOn(held, "getRangeAt").mockReturnValue(named);
    pressed("Backspace");
    asked.mockRestore();
    expect([held.anchorNode, held.anchorOffset]).toEqual([wordsAfter(), wordsAfter().length]);
    expect([held.focusNode, held.focusOffset]).toEqual([line, 1]);
  });

  it("stands a caret left inside the tag beside it, on the side of its half, so Delete takes the letter after", () => {
    opened();
    document.getSelection()?.collapse(tagWords(), 7);
    expect(pressed("Delete").defaultPrevented).toBe(false);
    const line = document.querySelector("p");
    expect(chosen()).toEqual([line, 2, line, 2]);
  });

  it("moves the choice only for a key that deletes", () => {
    opened();
    document.getSelection()?.setBaseAndExtent(tagWords(), 1, tagWords(), 5);
    pressed("ArrowLeft");
    expect(chosen()).toEqual([tagWords(), 1, tagWords(), 5]);
    pressed("Backspace");
    const line = document.querySelector("p");
    expect(chosen()).toEqual([line, 1, line, 2]);
  });
});
