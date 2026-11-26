import { afterEach, describe, expect, it } from "vitest";
import { balanced } from "../ui/pageMarks";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
  Reflect.deleteProperty(document, "caretRangeFromPoint");
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

const asked = (inputType: string, reach: Range | null, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => (reach ? [reach] : []) });
  document.querySelector(".page")?.dispatchEvent(event);
};

const pasted = (reach: Range, plain: string) => {
  const event = new InputEvent("beforeinput", {
    inputType: "insertFromPaste",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
  Object.defineProperty(event, "dataTransfer", {
    value: { getData: (kind: string) => (kind === "text/plain" ? plain : "") },
  });
  document.querySelector(".page")?.dispatchEvent(event);
};

const pressed = (key: string, kind: string, reach: Range) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  document.querySelector(".page")?.dispatchEvent(event);
  if (!event.defaultPrevented) asked(kind, reach);
};

const settled = async () => {
  for (let turn = 0; turn < 30; turn++) await Promise.resolve();
};

const wordsOf = (said: string) => {
  const walk = document.createTreeWalker(
    document.querySelector(".page") as Node,
    NodeFilter.SHOW_TEXT,
  );
  for (let node = walk.nextNode(); node; node = walk.nextNode())
    if ((node as Text).data === said) return node as Text;
  throw new Error(`no "${said}" on the page`);
};

const rangeOf = (from: [Node, number], to: [Node, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  return range;
};

const chose = (from: [Node, number], to: [Node, number]) => {
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(rangeOf(from, to));
};

const sheetAt = (offset: number): [Node, number] => [
  document.querySelector(".page") as Node,
  offset,
];

const TABLE =
  '<table><colgroup><col><col></colgroup><thead><tr><th scope="col">a</th><th scope="col">b</th></tr></thead>' +
  "<tbody><tr><td>1</td><td>2</td></tr></tbody></table>";

const OPENS_TABLE = "| a | b |\n|---|---|\n| 1 | 2 |\n\nAfter and more\n";
const OPENS_TABLE_PLACED =
  `<div class="table" data-at="0-30">${TABLE}</div>` +
  '<p data-at="31-46"><span data-src="31-45">After and more</span></p>';

const OPENS_TWO =
  "| a | b |\n|---|---|\n| 1 | 2 |\n\n```mermaid\nflowchart LR\n  A --> B\n```\n\nAfter\n";
const OPENS_TWO_PLACED =
  `<div class="table" data-at="0-30">${TABLE}</div>` +
  '<figure class="diagram" data-at="31-68"><pre><code class="language-mermaid">flowchart LR\n  A --&gt; B\n</code></pre></figure>' +
  '<p data-at="70-76"><span data-src="70-75">After</span></p>';

const FRONTED = "---\ntitle: x\n---\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nAfter\n";
const FRONTED_PLACED =
  '<dl class="front" data-at="0-17"><div><dt>title</dt><dd>x</dd></div></dl>' +
  `<div class="table" data-at="18-48">${TABLE}</div>` +
  '<p data-at="49-55"><span data-src="49-54">After</span></p>';

const allOfOpening = (words: string) => {
  const last = wordsOf(words);
  const sheet = document.querySelector(".page") as Element;
  const first = sheet.querySelector(".table") as Element;
  chose(sheetAt(Array.prototype.indexOf.call(sheet.childNodes, first)), [last, last.length]);
  return rangeOf([last, last.length], [last, last.length]);
};

describe("a choice from before a table that opens the page", () => {
  it("is typed over whole, though the engine names a caret at its end", () => {
    const text = opened(OPENS_TABLE, OPENS_TABLE_PLACED);
    asked("insertText", allOfOpening("After and more"), "Z");
    expect(text()).toBe("Z\n");
  });

  it("is pasted over whole, though the engine names a caret at its end", async () => {
    const text = opened(OPENS_TABLE, OPENS_TABLE_PLACED);
    pasted(allOfOpening("After and more"), "Z");
    await settled();
    expect(text()).toBe("Z\n");
  });

  it("takes every block drawn whole above the words, a diagram after the table too", () => {
    const text = opened(OPENS_TWO, OPENS_TWO_PLACED);
    asked("insertText", allOfOpening("After"), "Z");
    expect(text()).toBe("Z\n");
  });

  it("past the page's properties, takes the table and leaves the properties as they were", () => {
    const text = opened(FRONTED, FRONTED_PLACED);
    asked("insertText", allOfOpening("After"), "Z");
    expect(text()).toBe("---\ntitle: x\n---\n\nZ\n");
  });
});

describe("a choice from before a table that opens the page to the start of the words under it", () => {
  const toWordsStart = () => {
    const words = wordsOf("After and more");
    chose(sheetAt(0), [words, 0]);
    return rangeOf(sheetAt(0), [words, 0]);
  };

  it("is typed over up to the words, as Backspace and a cut take it", () => {
    const typed = opened(OPENS_TABLE, OPENS_TABLE_PLACED);
    asked("insertText", toWordsStart(), "Z");
    expect(typed()).toBe("ZAfter and more\n");
    writing?.stop();
    const backed = opened(OPENS_TABLE, OPENS_TABLE_PLACED);
    pressed("Backspace", "deleteContentBackward", toWordsStart());
    expect(backed()).toBe("After and more\n");
    writing?.stop();
    const cut = opened(OPENS_TABLE, OPENS_TABLE_PLACED);
    toWordsStart();
    writing?.cut();
    expect(cut()).toBe("After and more\n");
  });
});

const ENDS_CODE = "# T\n\nBefore\n\n```js\nconst a = 1;\n```\n";
const ENDS_CODE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-12"><span data-src="5-11">Before</span></p>' +
  '<figure class="code" data-at="13-35"><pre><code class="language-js" data-src="19-32">const a = 1;\n</code></pre></figure>';

const codeText = () => document.querySelector(".page code")?.firstChild as Text;

const allOfEnding = () => {
  const code = codeText();
  chose([wordsOf("T"), 0], [code, code.length]);
  return rangeOf([wordsOf("T"), 0], [code, code.length - 1]);
};

describe("a choice that runs to the end of a code block that ends the page", () => {
  it("is cut with the closing fence, leaving no backtick behind", () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    allOfEnding();
    expect(writing?.chosen()).toBe("T\n\nBefore\n\n```js\nconst a = 1;\n```");
    writing?.cut();
    expect(text()).toBe("# \n");
  });

  it("is typed over with the closing fence, so no fence is left open", () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    asked("insertText", allOfEnding(), "Z");
    expect(text()).toBe("# Z\n");
  });

  it("is pasted over with the closing fence", async () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    pasted(allOfEnding(), "Z");
    await settled();
    expect(text()).toBe("# Z\n");
  });

  it.each([
    ["Backspace", "deleteContentBackward"],
    ["Delete", "deleteContentForward"],
  ])("is taken by %s with the closing fence, so the page is left empty", (key, kind) => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    pressed(key, kind, allOfEnding());
    expect(text()).toBe("");
  });

  it("ending before the code's own last line feed, still takes the closing fence", () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    const code = codeText();
    chose([wordsOf("T"), 0], [code, code.length - 1]);
    writing?.cut();
    expect(text()).toBe("# \n");
  });
});

const MID_CODE = "# T\n\nBefore and more\n\n```js\nconst a = 1;\n```\n\nAfter\n";
const MID_CODE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-21"><span data-src="5-20">Before and more</span></p>' +
  '<figure class="code" data-at="22-44"><pre><code class="language-js" data-src="28-41">const a = 1;\n</code></pre></figure>' +
  '<p data-at="46-52"><span data-src="46-51">After</span></p>';

const toCodeEnd = () => {
  const words = wordsOf("Before and more");
  const code = codeText();
  chose([words, 11], [code, code.length]);
  return rangeOf([words, 11], [code, code.length - 1]);
};

describe("a choice from words above a code block mid-page to the code's end", () => {
  it("is typed over with the closing fence, the words under it kept apart", () => {
    const text = opened(MID_CODE, MID_CODE_PLACED);
    asked("insertText", toCodeEnd(), "Z");
    expect(text()).toBe("# T\n\nBefore and Z\n\nAfter\n");
  });

  it("is copied and cut with the closing fence, the line feeds after it kept", () => {
    const text = opened(MID_CODE, MID_CODE_PLACED);
    toCodeEnd();
    expect(writing?.chosen()).toBe("more\n\n```js\nconst a = 1;\n```");
    writing?.cut();
    expect(text()).toBe("# T\n\nBefore and \n\nAfter\n");
  });

  it("is carried by the pointer with both fences", () => {
    const text = opened(MID_CODE, MID_CODE_PLACED);
    toCodeEnd();
    const after = wordsOf("After");
    const spots: Record<number, [Node, number]> = { 10: [codeText(), 2], 60: [after, 5] };
    Object.defineProperty(document, "caretRangeFromPoint", {
      configurable: true,
      value: (x: number) => rangeOf(spots[x], spots[x]),
    });
    const mouse = (kind: string, x: number, on: EventTarget) =>
      on.dispatchEvent(
        new MouseEvent(kind, { bubbles: true, cancelable: true, button: 0, clientX: x, detail: 1 }),
      );
    mouse("mousedown", 10, document.querySelector(".page code") as Element);
    mouse("mousemove", 60, document);
    mouse("mouseup", 60, document);
    expect(text()).toBe("# T\n\nBefore and \n\nAftermore\n\n```js\nconst a = 1;\n```\n");
  });
});

const MARKED_CODE =
  "# T\n\nBefore and more\n\n```js\nconst s = '`' * 2;\n```\n\nAfter `x` and *y* end\n";
const MARKED_CODE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-21"><span data-src="5-20">Before and more</span></p>' +
  '<figure class="code" data-at="22-50"><pre><code class="language-js" data-src="28-47">' +
  "const s = '`' * 2;\n</code></pre></figure>" +
  '<p data-at="52-74"><span data-src="52-58">After </span><code data-src="58-61" contenteditable="false">x</code>' +
  '<span data-src="61-66"> and </span><em><span data-src="67-68">y</span></em><span data-src="69-73"> end</span></p>';

describe("a choice to the end of a code block that holds a lone star or backtick", () => {
  it("is cut with the fence and no further, as marks in code pair with nothing", () => {
    const text = opened(MARKED_CODE, MARKED_CODE_PLACED);
    toCodeEnd();
    expect(writing?.chosen()).toBe("more\n\n```js\nconst s = '`' * 2;\n```");
    writing?.cut();
    expect(text()).toBe("# T\n\nBefore and \n\nAfter `x` and *y* end\n");
  });
});

const OPENS_CODE = "```js\nconst a = 1;\n```\n\nAfter and more\n";
const OPENS_CODE_PLACED =
  '<figure class="code" data-at="0-22"><pre><code class="language-js" data-src="6-19">const a = 1;\n</code></pre></figure>' +
  '<p data-at="24-39"><span data-src="24-38">After and more</span></p>';

const allOfOpeningCode = () => {
  const words = wordsOf("After and more");
  chose([codeText(), 0], [words, words.length]);
  return rangeOf([codeText(), 0], [words, words.length]);
};

describe("a choice from the start of a code block that opens the page", () => {
  it("is typed over with the opening fence, so the words typed are no code", () => {
    const text = opened(OPENS_CODE, OPENS_CODE_PLACED);
    asked("insertText", allOfOpeningCode(), "Z");
    expect(text()).toBe("Z\n");
  });

  it("is cut with the opening fence", () => {
    const text = opened(OPENS_CODE, OPENS_CODE_PLACED);
    allOfOpeningCode();
    expect(writing?.chosen()).toBe(OPENS_CODE.slice(0, -1));
    writing?.cut();
    expect(text()).toBe("\n");
  });
});

describe("a choice of all the code inside one block", () => {
  it("is typed over between the fences, which stay", () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    const code = codeText();
    chose([code, 0], [code, code.length]);
    asked("insertText", rangeOf([code, 0], [code, code.length - 1]), "Z");
    expect(text()).toBe("# T\n\nBefore\n\n```js\nZ\n```\n");
  });

  it("is cut between the fences, which stay", () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    const code = codeText();
    chose([code, 0], [code, code.length - 1]);
    expect(writing?.chosen()).toBe("const a = 1;");
    writing?.cut();
    expect(text()).toBe("# T\n\nBefore\n\n```js\n\n```\n");
  });
});

const TWO_CODES = "```\nA1 A2\n```\n\npara\n\n```\nB1 B2\n```\n\nAfter\n";
const TWO_CODES_PLACED =
  '<figure class="code" data-at="0-13"><pre><code data-src="4-10">A1 A2\n</code></pre></figure>' +
  '<p data-at="15-20"><span data-src="15-19">para</span></p>' +
  '<figure class="code" data-at="21-34"><pre><code data-src="25-31">B1 B2\n</code></pre></figure>' +
  '<p data-at="36-42"><span data-src="36-41">After</span></p>';

describe("a choice from one code block into another", () => {
  it("from the middle of the first to the end of the second, shuts the first and takes the second", () => {
    const text = opened(TWO_CODES, TWO_CODES_PLACED);
    const first = wordsOf("A1 A2\n");
    const second = wordsOf("B1 B2\n");
    chose([first, 3], [second, second.length]);
    asked("insertText", rangeOf([first, 3], [second, second.length - 1]), "Z");
    expect(text()).toBe("```\nA1 Z\n```\n\nAfter\n");
  });

  it("from the start of the first to the middle of the second, takes the first and reopens the rest", () => {
    const text = opened(TWO_CODES, TWO_CODES_PLACED);
    const first = wordsOf("A1 A2\n");
    const second = wordsOf("B1 B2\n");
    chose([first, 0], [second, 2]);
    asked("insertText", rangeOf([first, 0], [second, 2]), "Z");
    expect(text()).toBe("Z\n\n```\n B2\n```\n\nAfter\n");
  });

  it("from the start of the first to the end of the second, takes both fences", () => {
    const text = opened(TWO_CODES, TWO_CODES_PLACED);
    const first = wordsOf("A1 A2\n");
    const second = wordsOf("B1 B2\n");
    chose([first, 0], [second, second.length]);
    asked("insertText", rangeOf([first, 0], [second, second.length - 1]), "Z");
    expect(text()).toBe("Z\n\nAfter\n");
  });
});

describe("a choice that starts on the sheet right before a code block mid-page", () => {
  const fromSheet = () => {
    const after = wordsOf("After");
    chose(sheetAt(2), [after, 3]);
    return rangeOf(sheetAt(2), [after, 3]);
  };

  it("is typed over from the opening fence, the words above kept apart", () => {
    const text = opened(MID_CODE, MID_CODE_PLACED);
    asked("insertText", fromSheet(), "Z");
    expect(text()).toBe("# T\n\nBefore and more\n\nZer\n");
  });

  it("is cut from the opening fence, the words above kept apart", () => {
    const text = opened(MID_CODE, MID_CODE_PLACED);
    fromSheet();
    expect(writing?.chosen()).toBe("```js\nconst a = 1;\n```\n\nAft");
    writing?.cut();
    expect(text()).toBe("# T\n\nBefore and more\n\ner\n");
  });
});

const LISTED_CODE = "# T\n\n1. ```js\n   x\n   ```\n\nPara **bold** end\n";
const LISTED_CODE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<ol data-at="5-27"><li data-at="5-27"><figure class="code" data-at="8-25"><pre>' +
  '<code class="language-js" data-src="17-19">x\n</code></pre></figure></li></ol>' +
  '<p data-at="27-45"><span data-src="27-32">Para </span><strong><span data-src="34-38">bold</span></strong>' +
  '<span data-src="40-44"> end</span></p>';

describe("words cut under a code block that opens on a list item's line", () => {
  it("take their format's marks along, as the fence above is no concern of theirs", () => {
    const text = opened(LISTED_CODE, LISTED_CODE_PLACED);
    const bold = wordsOf("bold");
    chose([bold, 0], [bold, 4]);
    expect(writing?.chosen()).toBe("**bold**");
    writing?.cut();
    expect(text()).toBe("# T\n\n1. ```js\n   x\n   ```\n\nPara  end\n");
  });
});

describe("marks paired for a cut past a fence in a quote", () => {
  it("count the words after the quote, where the fence ended with it", () => {
    const text = "> ```\n> x\nsome *bold* text\n";
    const end = text.indexOf("ld*");
    expect(balanced(text, { from: 0, to: end })).toEqual({ from: 0, to: end + 3 });
  });
});
