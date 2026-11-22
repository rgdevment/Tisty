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

const wordsOf = (src: string) => document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;

const backspacedAt = (words: string, above: string) => {
  const caret = wordsOf(words);
  document.getSelection()?.collapse(caret, 0);
  const end = wordsOf(above);
  const reach = document.createRange();
  reach.setStart(end, end.length);
  reach.setEnd(caret, 0);
  const event = new InputEvent("beforeinput", {
    inputType: "deleteContentBackward",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
  document.querySelector(".page")?.dispatchEvent(event);
};

describe("Backspace at the start of a block whose words open with a format", () => {
  it.each([
    [
      "bold to the paragraph above",
      "Uno.\n\n**negrita** dos\n",
      '<p data-at="0-5"><span data-src="0-4">Uno.</span></p><p data-at="6-22"><strong><span data-src="8-15">negrita</span></strong><span data-src="17-21"> dos</span></p>',
      "8-15",
      "0-4",
      "Uno.**negrita** dos\n",
    ],
    [
      "a link to the title above",
      "# Uno\n\n[enlace](https://example.com) dos\n",
      '<h1 data-at="0-6"><span data-src="2-5">Uno</span></h1><p data-at="7-41"><a href="https://example.com"><span data-src="8-14">enlace</span></a><span data-src="36-40"> dos</span></p>',
      "8-14",
      "2-5",
      "# Uno[enlace](https://example.com) dos\n",
    ],
    [
      "italic to a paragraph that ends in bold",
      "Uno **a**\n\n*b* dos\n",
      '<p data-at="0-10"><span data-src="0-4">Uno </span><strong><span data-src="6-7">a</span></strong></p><p data-at="11-19"><em><span data-src="12-13">b</span></em><span data-src="14-18"> dos</span></p>',
      "12-13",
      "6-7",
      "Uno **a***b* dos\n",
    ],
    [
      "bold to a paragraph that ends in bold, into one pair",
      "**uno**\n\n**dos** tres\n",
      '<p data-at="0-8"><strong><span data-src="2-5">uno</span></strong></p><p data-at="9-22"><strong><span data-src="11-14">dos</span></strong><span data-src="16-21"> tres</span></p>',
      "11-14",
      "2-5",
      "**unodos** tres\n",
    ],
    [
      "a star written as a word to the paragraph above",
      "Uno.\n\n\\*dos tres\n",
      '<p data-at="0-5"><span data-src="0-4">Uno.</span></p><p data-at="6-17"><span data-src="7-16">*dos tres</span></p>',
      "7-16",
      "0-4",
      "Uno.\\*dos tres\n",
    ],
    [
      "a bare address to the paragraph above",
      "Uno.\n\n<https://example.com> dos\n",
      '<p data-at="0-5"><span data-src="0-4">Uno.</span></p><p data-at="6-32"><a href="https://example.com"><span data-src="7-26">https://example.com</span></a><span data-src="27-31"> dos</span></p>',
      "7-26",
      "0-4",
      "Uno.<https://example.com> dos\n",
    ],
    [
      "a paragraph to a title written with closing hashes, which go",
      "## Título ##\n\nUno\n",
      '<h2 data-at="0-14"><span data-src="3-10">Título</span></h2><p data-at="15-19"><span data-src="15-18">Uno</span></p>',
      "15-18",
      "3-10",
      "## TítuloUno\n",
    ],
    [
      "a footnote's words to the paragraph above, its label going with the join",
      "Una nota[^a].\n\nUn párrafo más.\n\n[^a]: La primera nota.\n",
      '<p data-at="0-14"><span data-src="0-8">Una nota</span><sup class="fnref" data-src="8-12" contenteditable="false"><a href="#fn-a">a</a></sup><span data-src="12-13">.</span></p>' +
        '<p data-at="15-33"><span data-src="15-32">Un párrafo más.</span></p>' +
        '<div class="footnote" id="fn-a" data-at="34-57"><span class="fn-label">a</span><p data-at="40-57"><span data-src="40-56">La primera nota.</span></p></div>',
      "40-56",
      "15-32",
      "Una nota[^a].\n\nUn párrafo más.La primera nota.\n",
    ],
  ])("joins %s and keeps the marks that open it", (_, body, placed, words, above, joined) => {
    const text = opened(body, placed);
    backspacedAt(words, above);
    expect(text()).toBe(joined);
  });

  it.each([
    [
      "a bullet that opens with bold",
      "- uno\n- **negrita** dos\n",
      '<ul data-at="0-24"><li data-at="0-6"><span data-src="2-5">uno</span></li><li data-at="6-24"><strong><span data-src="10-17">negrita</span></strong><span data-src="19-23"> dos</span></li></ul>',
      "10-17",
      "2-5",
      "- uno\n\n**negrita** dos\n",
    ],
    [
      "a task that opens with a link",
      "- [ ] uno\n- [ ] [enlace](https://example.com) dos\n",
      '<ul data-at="0-50"><li class="task" data-at="0-10"><span class="check" data-done="false" contenteditable="false"></span><span data-src="6-9">uno</span></li>' +
        '<li class="task" data-at="10-50"><span class="check" data-done="false" contenteditable="false"></span><a href="https://example.com"><span data-src="17-23">enlace</span></a><span data-src="45-49"> dos</span></li></ul>',
      "17-23",
      "6-9",
      "- [ ] uno\n- [enlace](https://example.com) dos\n",
    ],
    [
      "a numbered task",
      "Antes.\n\n1. [ ] tarea\n",
      '<p data-at="0-7"><span data-src="0-6">Antes.</span></p><ol data-at="8-21"><li class="task" data-at="8-21"><span class="check" data-done="false" contenteditable="false"></span><span data-src="15-20">tarea</span></li></ol>',
      "15-20",
      "0-6",
      "Antes.\n\n1. tarea\n",
    ],
  ])(
    "takes the mark of %s away, as it does before plain words",
    (_, body, placed, words, above, freed) => {
      const text = opened(body, placed);
      backspacedAt(words, above);
      expect(text()).toBe(freed);
    },
  );

  it("takes words chosen from inside a format down to the next block, leaving one whole pair", () => {
    const text = opened(
      "**bold**\n\n**two** tres\n",
      '<p data-at="0-9"><strong><span data-src="2-6">bold</span></strong></p><p data-at="10-23"><strong><span data-src="12-15">two</span></strong><span data-src="17-22"> tres</span></p>',
    );
    const chosen = document.createRange();
    chosen.setStart(wordsOf("2-6"), 0);
    chosen.setEnd(wordsOf("12-15"), 0);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(chosen);
    document.querySelector(".page")?.dispatchEvent(
      new InputEvent("beforeinput", {
        inputType: "deleteContentBackward",
        bubbles: true,
        cancelable: true,
      }),
    );
    expect(text()).toBe("**two** tres\n");
  });
});
