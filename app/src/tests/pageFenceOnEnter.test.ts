import { afterEach, describe, expect, it } from "vitest";
import { fenceShutOnBreak, opening } from "../ui/pageLists";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const shutOn = (text: string, at: number) => {
  const moved = fenceShutOnBreak(text, at);
  if (!moved) return null;
  const { from, to, insert } = moved.edit;
  const next = text.slice(0, from) + insert + text.slice(to);
  return `${next.slice(0, moved.caret)}|${next.slice(moved.caret)}`;
};

describe("fenceShutOnBreak", () => {
  it.each([
    ["bare fence at the end", "```", 3, "```\n|\n```"],
    ["bare fence with its final newline", "```\n", 3, "```\n|\n```\n"],
    ["caret past the newline of the opener", "```\n", 4, "```\n|\n```\n"],
    ["a language on the opener", "# T\n\n```js", 10, "# T\n\n```js\n|\n```"],
    ["a longer tilde mark", "~~~~", 4, "~~~~\n|\n~~~~"],
    ["a fence in a quote", "> ```", 5, "> ```\n> |\n> ```"],
    ["a fence under a list item", "- ```", 5, "- ```\n  |\n  ```"],
    ["CRLF text", "a\r\n\r\n```\r\n", 8, "a\r\n\r\n```\r\n|\r\n```\r\n"],
    ["the caret on the blank line under the opener", "```\n\n", 4, "```\n|\n```\n"],
    ["blanks after the opener", "```  \n\n", 5, "```  \n|\n```\n"],
  ])("closes %s", (_name, text, at, expected) => {
    expect(shutOn(text, at)).toBe(expected);
  });

  it.each([
    ["a closing fence below", "```\n\n```\n", 3],
    ["a block with lines", "```\ncode\n", 9],
    ["the caret mid-line on the opener", "```js", 4],
    ["no fence", "texto\n", 5],
    ["a fence closed earlier and plain text after", "```\ncode\n```\n\ntexto\n", 19],
    ["words after the opener", "```\n\nmas\n", 4],
    ["words after the caret", "```\n\nmas\n", 3],
  ])("does nothing with %s", (_name, text, at) => {
    expect(fenceShutOnBreak(text, at)).toBeNull();
  });
});

describe("opening under an open fence", () => {
  it("closes a fence with lines before the new line", () => {
    const text = "# T\n\n```\ncode\n";
    expect(opening(text, text.length, "")).toBe("```\n\n");
  });

  it("breaks the line first when the body does not end in one", () => {
    const text = "# T\n\n```\ncode";
    expect(opening(text, text.length, "")).toBe("\n```\n\n");
  });

  it("keeps the lead of a quoted fence", () => {
    const text = "> ```\n> code";
    expect(opening(text, text.length, "")).toBe("\n> ```\n\n");
  });

  it("writes the closing lead with the line break of a CRLF text", () => {
    const text = "# T\r\n\r\n```\r\ncode\r\n";
    expect(opening(text, text.length, "")).toBe("```\r\n\r\n");
  });

  it("breaks a CRLF line first when the body does not end in one", () => {
    const text = "# T\r\n\r\n```\r\ncode";
    expect(opening(text, text.length, "")).toBe("\r\n```\r\n\r\n");
  });

  it("is unchanged for a closed fence", () => {
    const text = "# T\n\n```\ncode\n```\n";
    expect(opening(text, text.length, "")).toBe("\n");
  });

  it("is unchanged when text follows the place", () => {
    const text = "# T\n\n```\ncode\n";
    expect(opening(`${text}mas\n`, text.length, "")).toBe("\n");
  });
});

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
  writing.drawn();
  return () => text;
};

const input = (inputType: string, data?: string) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

const caretOn = (selector: string, offset = 0) => {
  document.getSelection()?.collapse(document.querySelector(selector) as Node, offset);
  document.dispatchEvent(new Event("selectionchange"));
};

describe("Enter on a fence that arrived whole", () => {
  const placed =
    '<p data-at="0-6"><span data-src="0-5">Antes</span></p><figure class="code" data-at="7-10"><pre><code></code></pre></figure>';

  it("writes the closing fence and leaves the caret on the blank line", () => {
    const text = opened("Antes\n\n```", placed);
    caretOn(".page code");
    input("insertParagraph");
    expect(text()).toBe("Antes\n\n```\n\n```");
  });

  it("is closed by Shift+Enter too, with the caret on the blank line", () => {
    const text = opened("Antes\n\n```", placed);
    caretOn(".page code");
    input("insertLineBreak");
    expect(text()).toBe("Antes\n\n```\n\n```");
    input("insertText", "dentro");
    expect(text()).toBe("Antes\n\n```\ndentro\n```");
  });

  it("lands the words typed next between the fences", () => {
    const text = opened("Antes\n\n```", placed);
    caretOn(".page code");
    input("insertParagraph");
    input("insertText", "dentro");
    expect(text()).toBe("Antes\n\n```\ndentro\n```");
  });
});

describe("a line begun under an old body that ends in an open fence", () => {
  it("closes the fence before the new words", () => {
    const body = "# T\n\n```\nviejo\n";
    const text = opened(
      body,
      '<h1 data-at="0-4"><span data-src="2-3">T</span></h1><figure class="code" data-at="5-15"><pre><code data-src="9-15">viejo\n</code></pre></figure>',
    );
    writing?.begin(body.length, "", document.querySelector("figure") as Element);
    input("insertText", "nuevo");
    expect(text()).toBe("# T\n\n```\nviejo\n```\n\nnuevo");
  });
});
