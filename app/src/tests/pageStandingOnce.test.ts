import { afterEach, describe, expect, it, vi } from "vitest";
import * as spots from "../ui/pageSpots";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

vi.mock("../ui/pageSpots", async (real) => {
  const own = await real<typeof import("../ui/pageSpots")>();
  return { ...own, spotOf: vi.fn(own.spotOf) };
});

const TWO = "# T\n\nHola mundo bonito\n\nSegundo párrafo\n";
const PLACED =
  '<h1 id="t" data-at="0-4"><span data-src="2-3">T</span></h1><p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p><p data-at="24-41"><span data-src="24-40">Segundo párrafo</span></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  vi.restoreAllMocks();
  document.body.innerHTML = "";
});

const typedAhead = () => {
  let text = TWO;
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
  const words = document.querySelector('[data-src="5-22"]')?.firstChild as Text;
  document.getSelection()?.collapse(words, words.length);
  pressed("X");
  asked("insertText", "X");
  return () => text;
};

const sheet = () => document.querySelector(".page") as Element;

const asked = (inputType: string, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => [] });
  sheet().dispatchEvent(event);
};

const pressed = (key: string) =>
  sheet().dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));

describe("a key that comes before the page shows the last one", () => {
  it("reads the page for its place once in each of its events", () => {
    const text = typedAhead();
    const read = vi.mocked(spots.spotOf);
    read.mockClear();
    pressed("Backspace");
    expect(read).toHaveBeenCalledTimes(1);
    read.mockClear();
    asked("deleteContentBackward");
    expect(read).toHaveBeenCalledTimes(1);
    expect(text()).toBe(TWO);
  });

  it("reads the page without leaving a live range behind for the page to keep up", () => {
    typedAhead();
    const made = vi.spyOn(document, "createRange");
    pressed("Backspace");
    expect(made).not.toHaveBeenCalled();
  });
});
