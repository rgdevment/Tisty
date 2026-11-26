import { afterEach, describe, expect, it } from "vitest";
import { fencedOut } from "../ui/pageLists";
import { balanced } from "../ui/pageMarks";
import { fenceAt, mended } from "../ui/pageMends";
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

const composed = (data: string) => {
  const sheet = document.querySelector(".page");
  sheet?.dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true, data: "" }));
  sheet?.dispatchEvent(new CompositionEvent("compositionend", { bubbles: true, data }));
};

const settled = async () => {
  for (let turn = 0; turn < 30; turn++) await Promise.resolve();
};

const chose = (from: [Node, number], to: [Node, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
  document.dispatchEvent(new Event("selectionchange"));
};

const ENDS_CODE = "# T\n\nBefore\n\n```js\nconst a = 1;\n```\n";
const ENDS_CODE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-12"><span data-src="5-11">Before</span></p>' +
  '<figure class="code" data-at="13-35"><pre><code class="language-js" data-src="19-32">const a = 1;\n</code></pre></figure>';

describe("a choice to a code block's end, let go of before the menu reads it", () => {
  it("is copied and cut with the closing fence, as it was while chosen", () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    const code = document.querySelector(".page code")?.firstChild as Text;
    chose([wordsOf("T"), 0], [code, code.length]);
    document.getSelection()?.removeAllRanges();
    expect(writing?.choiceHeld()?.words).toBe("T\n\nBefore\n\n```js\nconst a = 1;\n```");
    writing?.choiceHeld()?.cut?.();
    expect(text()).toBe("# \n");
  });

  it("is worn by a format on its words alone, the code left as it is", () => {
    const text = opened(ENDS_CODE, ENDS_CODE_PLACED);
    const code = document.querySelector(".page code")?.firstChild as Text;
    chose([wordsOf("T"), 0], [code, code.length]);
    document.getSelection()?.removeAllRanges();
    writing?.format("**", "**");
    expect(text()).toBe("# **T**\n\n**Before**\n\n```js\nconst a = 1;\n```\n");
  });
});

const SPANNED = "# T\n\nUse `a*b` here and *it* there\n";
const SPANNED_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-35"><span data-src="5-9">Use </span><code data-src="9-14" contenteditable="false">a*b</code>' +
  '<span data-src="14-24"> here and </span><em><span data-src="25-27">it</span></em>' +
  '<span data-src="28-34"> there</span></p>';

describe("words cut across inline code that holds a star", () => {
  it("take the italic they end inside whole, as the star in the code pairs with nothing", () => {
    const text = opened(SPANNED, SPANNED_PLACED);
    chose([wordsOf("Use "), 0], [wordsOf("it"), 1]);
    expect(writing?.choiceHeld()?.words).toBe("Use `a*b` here and *it*");
    writing?.choiceHeld()?.cut?.();
    expect(text()).toBe("# T\n\n there\n");
  });
});

const MID = "# T\n\nWords here\n\n```js\na * b\nc\n```\n\nAfter *it* end\n";
const MID_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-16"><span data-src="5-15">Words here</span></p>' +
  '<figure class="code" data-at="17-34"><pre><code class="language-js" data-src="23-31">' +
  "a * b\nc\n</code></pre></figure>" +
  '<p data-at="36-51"><span data-src="36-42">After </span><em><span data-src="43-45">it</span></em>' +
  '<span data-src="46-50"> end</span></p>';

const codeText = () => document.querySelector(".page code")?.firstChild as Text;

const intoCode = () => {
  const from: [Node, number] = [wordsOf("Words here"), 3];
  chose(from, [codeText(), 4]);
  return rangeOf(from, [codeText(), 4]);
};

const outOfCode = () => {
  const to: [Node, number] = [wordsOf("After "), 3];
  chose([codeText(), 4], to);
  return rangeOf([codeText(), 4], to);
};

const REOPENED = "# T\n\nWor\n\n```js\nb\nc\n```\n\nAfter *it* end\n";
const CLOSED = "# T\n\nWords here\n\n```js\na * \n```\n\ner *it* end\n";

describe("a choice from words into the middle of a code block", () => {
  it("is copied as words and the code it took, fenced, with no mark paired past it", () => {
    opened(MID, MID_PLACED);
    intoCode();
    expect(writing?.choiceHeld()?.words).toBe("ds here\n\n```js\na * \n```");
  });

  it("is cut, and the code left behind opens with its own fence", () => {
    const text = opened(MID, MID_PLACED);
    intoCode();
    writing?.choiceHeld()?.cut?.();
    expect(text()).toBe(REOPENED);
  });

  it("is typed over, and the code left behind opens with its own fence", () => {
    const text = opened(MID, MID_PLACED);
    asked("insertText", intoCode(), "Z");
    expect(text()).toBe(REOPENED.replace("Wor\n", "WorZ\n"));
  });

  it("is pasted over, and the code left behind opens with its own fence", async () => {
    const text = opened(MID, MID_PLACED);
    pasted(intoCode(), "Z");
    await settled();
    expect(text()).toBe(REOPENED.replace("Wor\n", "WorZ\n"));
  });

  it.each([
    ["Backspace", "deleteContentBackward"],
    ["Delete", "deleteContentForward"],
  ])("is taken by %s, and the code left behind opens with its own fence", (key, kind) => {
    const text = opened(MID, MID_PLACED);
    pressed(key, kind, intoCode());
    expect(text()).toBe(REOPENED);
  });

  it("is typed over by a composition, and the code left behind opens with its own fence", () => {
    const text = opened(MID, MID_PLACED);
    intoCode();
    composed("é");
    expect(text()).toBe(REOPENED.replace("Wor\n", "Woré\n"));
  });

  it("is split by Enter, and the code left behind opens with its own fence", () => {
    const text = opened(MID, MID_PLACED);
    asked("insertParagraph", intoCode());
    expect(text()).toBe(REOPENED);
  });
});

describe("a choice from the middle of a code block out into the words under it", () => {
  it("is copied as the code it took, fenced, and the words", () => {
    opened(MID, MID_PLACED);
    outOfCode();
    expect(writing?.choiceHeld()?.words).toBe("```js\nb\nc\n```\n\nAft");
  });

  it("is cut, and the code left above shuts with its own fence", () => {
    const text = opened(MID, MID_PLACED);
    outOfCode();
    writing?.choiceHeld()?.cut?.();
    expect(text()).toBe(CLOSED);
  });

  it("is typed over inside the code, which then shuts with its own fence", () => {
    const text = opened(MID, MID_PLACED);
    asked("insertText", outOfCode(), "Z");
    expect(text()).toBe(CLOSED.replace("a * \n", "a * Z\n"));
  });

  it("is taken by Backspace, and the code left above shuts with its own fence", () => {
    const text = opened(MID, MID_PLACED);
    pressed("Backspace", "deleteContentBackward", outOfCode());
    expect(text()).toBe(CLOSED);
  });
});

describe("marks paired for words that end on a fence's line", () => {
  it("leave the fence's backticks out of the pairing", () => {
    const text = "Words\n\n```js\ncode\n```\n";
    expect(balanced(text, { from: 0, to: 10 })).toEqual({ from: 0, to: 10 });
  });

  it("read a fence opened on a list item's line as code", () => {
    const text = "1. ```js\n   a * b\n   ```\n\nAfter *it* end\n";
    const to = text.indexOf("a * ") + 4;
    expect(balanced(text, { from: 0, to })).toEqual({ from: 0, to });
  });
});

describe("Shift+Enter over a choice from words into the middle of a code block", () => {
  it("leaves no backslash before the code's own fence", () => {
    const text = opened(MID, MID_PLACED);
    asked("insertLineBreak", intoCode());
    expect(text()).toBe(REOPENED);
  });
});

describe("a choice from words into the middle of a code block, carried by the pointer", () => {
  it("lands as blocks of its own in the middle of a line, its fences on lines of their own", () => {
    const text = opened(MID, MID_PLACED);
    intoCode();
    const spots: Record<number, [Node, number]> = {
      10: [codeText(), 2],
      60: [wordsOf("After "), 3],
    };
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
    expect(text()).toBe(
      "# T\n\nWor\n\n```js\nb\nc\n```\n\nAft\n\nds here\n\n```js\na * \n```\n\ner *it* end\n",
    );
  });
});

const LISTED = "# T\n\n1. ```js\n   a * b\n   ```\n\nAfter\n";
const LISTED_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<ol data-at="5-31"><li data-at="5-31"><figure class="code" data-at="8-29"><pre>' +
  '<code class="language-js" data-src="17-23">a * b\n</code></pre></figure></li></ol>' +
  '<p data-at="31-37"><span data-src="31-36">After</span></p>';

describe("a choice from the middle of a code block that opens on a list item's line", () => {
  const outOfListed = () => {
    const to: [Node, number] = [wordsOf("After"), 3];
    chose([codeText(), 4], to);
    return rangeOf([codeText(), 4], to);
  };

  it("shuts the code left in the item with its indented fence, not a new item", () => {
    const text = opened(LISTED, LISTED_PLACED);
    pressed("Backspace", "deleteContentBackward", outOfListed());
    expect(text()).toBe("# T\n\n1. ```js\n   a * \n   ```\n\ner\n");
  });

  it("is copied with the item's fence, and the rest of its line indented", () => {
    opened(LISTED, LISTED_PLACED);
    outOfListed();
    expect(writing?.choiceHeld()?.words).toBe("1. ```js\n   b\n   ```\n\nAft");
  });
});

const BOLD_AFTER = "# T\n\n```js\na * b\n```\n\n**bold** end\n";
const BOLD_AFTER_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<figure class="code" data-at="5-20"><pre><code class="language-js" data-src="11-17">a * b\n</code></pre></figure>' +
  '<p data-at="22-35"><strong><span data-src="24-28">bold</span></strong><span data-src="30-34"> end</span></p>';

describe("a choice from the middle of a code block into a bold word", () => {
  it("opens the bold again after the fence, not inside the code", () => {
    const text = opened(BOLD_AFTER, BOLD_AFTER_PLACED);
    const to: [Node, number] = [wordsOf("bold"), 2];
    chose([codeText(), 2], to);
    pressed("Backspace", "deleteContentBackward", rangeOf([codeText(), 2], to));
    expect(text()).toBe("# T\n\n```js\na \n```\n\n**ld** end\n");
  });
});

describe("the fence that shuts code left in a quote", () => {
  it("lands after the quote's mark when the choice starts at a line's words", () => {
    const text = "> ```\n> a\n> b\n> ```\n\nAfter\n";
    const from = text.indexOf("b\n");
    const to = text.indexOf("ter");
    const mend = { first: fenceAt(text, 0), last: null };
    const edit = mended(text, { from, to, insert: "" }, mend);
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(
      "> ```\n> a\n> ```\n\nter\n",
    );
  });
});

describe("marks paired for words near code", () => {
  it("skip a backtick written with a backslash", () => {
    const text = "Price \\` and *it* and `x`\n";
    const to = text.indexOf("*it") + 2;
    expect(balanced(text, { from: 0, to })).toEqual({ from: 0, to: to + 2 });
  });

  it("read a fence left open in a list item as shut once the item ends", () => {
    const text = "- ```\n  x\n\nPara *it* end\n";
    const from = text.indexOf("Para");
    const to = text.indexOf("*it") + 2;
    expect(balanced(text, { from, to })).toEqual({ from, to: to + 2 });
  });
});

describe("a key that splits a choice into code moves the caret to the code", () => {
  it.each([["insertParagraph"], ["insertLineBreak"]])(
    "after %s, at the code's first letter, past its fence",
    (kind) => {
      opened(MID, MID_PLACED);
      asked(kind, intoCode());
      expect(writing?.caretAt()).toBe(REOPENED.indexOf("b\nc"));
    },
  );
});

const INDENTED = "```py\n    return x\n```\n\nafter text\n";
const INDENTED_PLACED =
  '<figure class="code" data-at="0-22"><pre><code class="language-py" data-src="6-19">' +
  "    return x\n</code></pre></figure>" +
  '<p data-at="24-35"><span data-src="24-34">after text</span></p>';

describe("a choice from the words of an indented code line out of the code", () => {
  it("shuts the code on a line of its own, not behind the line's indent", () => {
    const text = opened(INDENTED, INDENTED_PLACED);
    const to: [Node, number] = [wordsOf("after text"), 2];
    chose([codeText(), 4], to);
    pressed("Backspace", "deleteContentBackward", rangeOf([codeText(), 4], to));
    expect(text()).toBe("```py\n    \n```\n\nter text\n");
  });
});

const TWO_CODES = "Words here\n\n```js\na * b\n```\n\n```py\nxy\n```\n";
const TWO_CODES_PLACED =
  '<p data-at="0-11"><span data-src="0-10">Words here</span></p>' +
  '<figure class="code" data-at="12-27"><pre><code class="language-js" data-src="18-24">a * b\n</code></pre></figure>' +
  '<figure class="code" data-at="29-41"><pre><code class="language-py" data-src="35-38">xy\n</code></pre></figure>';

describe("words and fenced code carried by the pointer", () => {
  it("are not let go inside another code block, whose fences they would break", () => {
    const text = opened(TWO_CODES, TWO_CODES_PLACED);
    const [first, second] = Array.from(document.querySelectorAll(".page code")).map(
      (one) => one.firstChild as Text,
    );
    chose([wordsOf("Words here"), 3], [first, 4]);
    const spots: Record<number, [Node, number]> = { 10: [first, 2], 60: [second, 1] };
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
    expect(text()).toBe(TWO_CODES);
  });
});

describe("code shut in a quote, with words of the quote after it", () => {
  it("keeps those words in the quote", () => {
    const text = "> ```\n> code\n> ```\n> para text\n";
    const from = text.indexOf("ode");
    const to = text.indexOf("ra text");
    const edit = mended(text, { from, to, insert: "" }, { first: fenceAt(text, 0), last: null });
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(
      "> ```\n> c\n> ```\n>\n> ra text\n",
    );
  });
});

describe("fences read in a quote's list item, or behind a tab", () => {
  it("hold the item's code until the quote's words after it", () => {
    const text = "> - ```js\n>   a *b\n>   c* d\n>   ```\n> after\n";
    const blank = (line: string) => " ".repeat(line.length);
    expect(fencedOut(text)).toBe(
      [
        blank("> - ```js"),
        blank(">   a *b"),
        blank(">   c* d"),
        blank(">   ```"),
        "> after",
        "",
      ].join("\n"),
    );
  });

  it("hold a line indented by a tab inside the item", () => {
    const text = "- ```\n\tx *a\n  ```\n\nPara\n";
    expect(fencedOut(text).split("\n")[1]).toBe(" ".repeat("\tx *a".length));
  });
});

describe("a backslash written with a backslash", () => {
  it("leaves the star after it a mark to pair", () => {
    const text = "a \\\\*b* c\n";
    const from = text.indexOf("b");
    const to = text.indexOf("c");
    expect(balanced(text, { from, to })).toEqual({ from: from - 1, to });
  });
});

describe("Enter over a choice from the middle of a code block into a bold word", () => {
  it("opens the bold again after the fence, as Backspace does", () => {
    const text = opened(BOLD_AFTER, BOLD_AFTER_PLACED);
    const to: [Node, number] = [wordsOf("bold"), 2];
    chose([codeText(), 2], to);
    asked("insertParagraph", rangeOf([codeText(), 2], to));
    expect(text()).toBe("# T\n\n```js\na \n\n```\n\n**ld** end\n");
  });
});

const ITALIC_BEFORE = "# T\n\nSome *it* x\n\n```js\na * b\n```\n";
const ITALIC_BEFORE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-17"><span data-src="5-10">Some </span><em><span data-src="11-13">it</span></em>' +
  '<span data-src="14-16"> x</span></p>' +
  '<figure class="code" data-at="18-33"><pre><code class="language-js" data-src="24-30">a * b\n</code></pre></figure>';

describe("Shift+Enter over a choice from an italic word into the middle of a code block", () => {
  it("shuts the italic before the code's own fence", () => {
    const text = opened(ITALIC_BEFORE, ITALIC_BEFORE_PLACED);
    const from: [Node, number] = [wordsOf("it"), 1];
    chose(from, [codeText(), 4]);
    asked("insertLineBreak", rangeOf(from, [codeText(), 4]));
    expect(text()).toBe("# T\n\nSome *i*\n\n```js\nb\n```\n");
  });
});

describe("a fence left open on a list item's own lines", () => {
  it("is shut where the item ends, so the words after it pair their marks", () => {
    const text = "- item\n\n  ```\n  x\n\nPara *it* end\n";
    const from = text.indexOf("Para");
    const to = text.indexOf("*it") + 2;
    expect(balanced(text, { from, to })).toEqual({ from, to: to + 2 });
  });

  it("still shuts on its own fence when that fence is less indented than the opening one", () => {
    const text = "   ```\nx *a\n```\n\nPara *it* end\n";
    const from = text.indexOf("Para");
    const to = text.indexOf("*it") + 2;
    expect(balanced(text, { from, to })).toEqual({ from, to: to + 2 });
  });
});
