import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY =
  "```js\nconst a = 1;\n```\n\nTras el código.\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nTras la tabla.\n";
const PLACED =
  '<figure class="code" data-at="0-22"><pre><code class="language-js" data-src="6-19">const a = 1;\n</code></pre></figure>' +
  '<p data-at="24-41"><span data-src="24-40">Tras el código.</span></p>' +
  '<div class="table" data-at="42-72"><table><thead><tr><th scope="col">a</th><th scope="col">b</th></tr></thead>' +
  "<tbody><tr><td>1</td><td>2</td></tr></tbody></table></div>" +
  '<p data-at="73-88"><span data-src="73-87">Tras la tabla.</span></p>';

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

const startOf = (src: string) => {
  const words = document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;
  document.getSelection()?.collapse(words, 0);
  return words;
};

const reachingBack = (inputType: string, from: Node, offset: number) => {
  const held = document.getSelection();
  const caret = held?.getRangeAt(0);
  const reach = document.createRange();
  reach.setStart(from, offset);
  if (caret) reach.setEnd(caret.startContainer, caret.startOffset);
  const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
  document.querySelector(".page")?.dispatchEvent(event);
};

describe("a word's or a line's delete at the start of a block after one drawn whole", () => {
  for (const kind of ["deleteWordBackward", "deleteSoftLineBackward", "deleteHardLineBackward"]) {
    it(`leaves a code block's fence as it was on ${kind}`, () => {
      const text = opened();
      startOf("24-40");
      const code = document.querySelector("code")?.firstChild as Text;
      reachingBack(kind, code, code.length - 3);
      expect(text()).toBe(BODY);
    });

    it(`leaves a table as it was on ${kind}`, () => {
      const text = opened();
      startOf("73-87");
      const cell = document.querySelectorAll("td")[1]?.firstChild as Text;
      reachingBack(kind, cell, 0);
      expect(text()).toBe(BODY);
    });
  }
});
