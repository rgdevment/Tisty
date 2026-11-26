import { afterEach, describe, expect, it } from "vitest";
import { balanced, wornOver } from "../ui/pageMarks";
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

const chose = (from: [Node, number], to: [Node, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
  document.dispatchEvent(new Event("selectionchange"));
};

const letGo = () => document.getSelection()?.removeAllRanges();

const codes = () =>
  Array.from(document.querySelectorAll(".page code")).map((one) => one.firstChild as Text);

const MID = "# T\n\nWords here\n\n```js\na * b\nc\n```\n\nAfter *it* end\n";
const MID_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-16"><span data-src="5-15">Words here</span></p>' +
  '<figure class="code" data-at="17-34"><pre><code class="language-js" data-src="23-31">' +
  "a * b\nc\n</code></pre></figure>" +
  '<p data-at="36-51"><span data-src="36-42">After </span><em><span data-src="43-45">it</span></em>' +
  '<span data-src="46-50"> end</span></p>';

const intoCode = () => chose([wordsOf("Words here"), 3], [codes()[0], 4]);
const outOfCode = () => chose([codes()[0], 4], [wordsOf("After "), 3]);

describe("a format over words on both sides of a code block", () => {
  it.each([
    ["**", "**"],
    ["*", "*"],
    ["~~", "~~"],
    ["`", "`"],
  ])("wraps the words in %s and leaves the code and its fences as they are", (open, close) => {
    const text = opened(MID, MID_PLACED);
    chose([wordsOf("Words here"), 3], [wordsOf("After "), 3]);
    writing?.format(open, close);
    expect(text()).toBe(
      `# T\n\nWor${open}ds here${close}\n\n\`\`\`js\na * b\nc\n\`\`\`\n\n${open}Aft${close}er *it* end\n`,
    );
  });
});

describe("a format over a choice into or out of the middle of a code block", () => {
  it("wraps the words before the code, while the choice is held", () => {
    const text = opened(MID, MID_PLACED);
    intoCode();
    writing?.format("**", "**");
    expect(text()).toBe(MID.replace("Words here", "Wor**ds here**"));
  });

  it("wraps the words after the code, while the choice is held", () => {
    const text = opened(MID, MID_PLACED);
    outOfCode();
    writing?.format("**", "**");
    expect(text()).toBe(MID.replace("After ", "**Aft**er "));
  });

  it("wraps the words before the code, once the choice is let go", () => {
    const text = opened(MID, MID_PLACED);
    intoCode();
    letGo();
    writing?.format("**", "**");
    expect(text()).toBe(MID.replace("Words here", "Wor**ds here**"));
  });

  it("wraps the words after the code, once the choice is let go", () => {
    const text = opened(MID, MID_PLACED);
    outOfCode();
    letGo();
    writing?.format("**", "**");
    expect(text()).toBe(MID.replace("After ", "**Aft**er "));
  });

  it("is offered by the menu", () => {
    opened(MID, MID_PLACED);
    intoCode();
    expect(writing?.formattable()).toBe(true);
  });
});

describe("a choice that stays inside code", () => {
  it("takes no format, nor is one offered, in one code block", () => {
    const text = opened(MID, MID_PLACED);
    chose([codes()[0], 0], [codes()[0], 7]);
    expect(writing?.formattable()).toBe(false);
    writing?.format("**", "**");
    expect(text()).toBe(MID);
  });

  it("takes no format once let go", () => {
    const text = opened(MID, MID_PLACED);
    chose([codes()[0], 0], [codes()[0], 7]);
    letGo();
    writing?.format("**", "**");
    expect(text()).toBe(MID);
  });
});

const INLINE = "# T\n\nSay `a*b` here\n\n```js\nc\n```\n";
const INLINE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-20"><span data-src="5-9">Say </span><code data-src="9-14" contenteditable="false">a*b</code>' +
  '<span data-src="14-19"> here</span></p>' +
  '<figure class="code" data-at="21-32"><pre><code class="language-js" data-src="27-29">c\n</code></pre></figure>';

describe("a choice from a code block's letters to the block's own edge", () => {
  it("takes no format, as it stays inside the block", () => {
    const text = opened(MID, MID_PLACED);
    chose([codes()[0], 0], [document.querySelector(".page figure.code") as Node, 1]);
    expect(writing?.formattable()).toBe(false);
    writing?.format("**", "**");
    expect(text()).toBe(MID);
  });
});

describe("a choice from inline code into a code block", () => {
  it("takes no format, as inline code is written as it stands", () => {
    const text = opened(INLINE, INLINE_PLACED);
    const [inline, block] = codes();
    chose([inline, 1], [block, 1]);
    expect(writing?.formattable()).toBe(false);
    writing?.format("**", "**");
    expect(text()).toBe(INLINE);
  });
});

const BETWEEN = "# T\n\n```js\na * b\n```\n\nMiddle *it* words\n\n```py\nx * y\n```\n";
const BETWEEN_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<figure class="code" data-at="5-20"><pre><code class="language-js" data-src="11-17">a * b\n</code></pre></figure>' +
  '<p data-at="22-40"><span data-src="22-29">Middle </span><em><span data-src="30-32">it</span></em>' +
  '<span data-src="33-39"> words</span></p>' +
  '<figure class="code" data-at="41-56"><pre><code class="language-py" data-src="47-53">x * y\n</code></pre></figure>';

describe("a choice from one code block into another", () => {
  it("wraps only the words between them", () => {
    const text = opened(BETWEEN, BETWEEN_PLACED);
    const [first, second] = codes();
    chose([first, 2], [second, 3]);
    writing?.format("**", "**");
    expect(text()).toBe(BETWEEN.replace("Middle *it* words", "**Middle *it* words**"));
  });
});

const UNSHUT = "# T\n\nWords here\n\n```js\na * b\nc\n";
const UNSHUT_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-16"><span data-src="5-15">Words here</span></p>' +
  '<figure class="code" data-at="17-31"><pre><code class="language-js" data-src="23-31">' +
  "a * b\nc\n</code></pre></figure>";

describe("a choice into code whose fence is never shut", () => {
  it("wraps the words before the code, as the page draws the rest as code", () => {
    const text = opened(UNSHUT, UNSHUT_PLACED);
    chose([wordsOf("Words here"), 3], [codes()[0], 4]);
    writing?.format("*", "*");
    expect(text()).toBe(UNSHUT.replace("Words here", "Wor*ds here*"));
  });
});

const wornIn = (text: string, from: number, to: number) => {
  const change = wornOver(text, { from, to }, "**", "**")?.change;
  return change ? text.slice(0, change.from) + change.insert + text.slice(change.to) : text;
};

describe("a format over lines across code", () => {
  it("leaves code written with Windows line ends as it is", () => {
    const text = "Words\r\n\r\n```js\r\na * b\r\n```\r\n\r\nAfter\r\n";
    expect(wornIn(text, 0, text.indexOf("After") + 5)).toBe(
      "**Words**\r\n\r\n```js\r\na * b\r\n```\r\n\r\n**After**\r\n",
    );
  });

  it("leaves code in a quote as it is, and wraps the quote's words past its mark", () => {
    const text = "> Words\n>\n> ```\n> a * b\n> ```\n>\n> After\n";
    expect(wornIn(text, 2, text.indexOf("After") + 5)).toBe(
      "> **Words**\n>\n> ```\n> a * b\n> ```\n>\n> **After**\n",
    );
  });

  it("leaves code opened on a list item's line as it is", () => {
    const text = "Words\n\n1. ```js\n   a * b\n   ```\n\nAfter\n";
    expect(wornIn(text, 0, text.indexOf("After") + 5)).toBe(
      "**Words**\n\n1. ```js\n   a * b\n   ```\n\n**After**\n",
    );
  });
});

describe("marks paired for words cut across inline code that runs over a line break", () => {
  it("read the code as code, so a star in it pairs with nothing", () => {
    const text = "Use `a\nb*c` here and *it* there\n";
    const to = text.indexOf("*it") + 2;
    expect(balanced(text, { from: 0, to })).toEqual({ from: 0, to: to + 2 });
  });

  it.each([
    ["a blank line", "x `a\n\n*it* b`\n"],
    ["a list item", "x `a\n- *it* b`\n"],
    ["a heading", "x `a\n# *it* b`\n"],
    ["a break written with Windows line ends", "x `a\r\n---\r\n*it* b`\r\n"],
  ])("end the code at %s, so the marks after it pair", (_, text) => {
    const from = text.indexOf("it") + 1;
    expect(balanced(text, { from, to: text.trimEnd().length }).from).toBe(from - 2);
  });
});

describe("a format over words around inline code that runs over a line break", () => {
  it("wraps the words on each side of the code, never the code", () => {
    const text = "a `foo\nbar` b\n";
    expect(wornIn(text, 0, text.length - 1)).toBe("**a** `foo\nbar` **b**\n");
  });

  it("wraps the words before the code when the choice ends in it", () => {
    const text = "a `foo\nbar` b\n";
    expect(wornIn(text, 0, text.indexOf("\n"))).toBe("**a** `foo\nbar` b\n");
  });
});

describe("marks paired for words before a quote", () => {
  it("end a paragraph's code at the quote, as a quote breaks the paragraph", () => {
    const text = "say `x and *more* end\n> y` z\n";
    const from = text.indexOf("ore");
    const to = text.indexOf(" end") + 3;
    expect(balanced(text, { from, to })).toEqual({ from: text.indexOf("*more"), to });
  });
});

describe("the words a format over code leaves chosen", () => {
  it("end at the last words, not in the code after them", () => {
    opened(MID, MID_PLACED);
    intoCode();
    writing?.format("`", "`");
    expect(writing?.choiceHeld()?.words).toBe("`ds here`");
  });

  it("start at the first words, not in the code before them", () => {
    opened(MID, MID_PLACED);
    outOfCode();
    writing?.format("*", "*");
    expect(writing?.choiceHeld()?.words).toBe("*Aft*");
  });
});

describe("inline code that runs over a line break inside one paragraph", () => {
  it("runs on over a numbered line that cannot open a list there", () => {
    const text = "a `b\n2. c` d\n";
    expect(wornIn(text, 0, text.length - 1)).toBe("**a** `b\n2. c` **d**\n");
  });

  it("runs on over a quote's lazy line to the quote's next line", () => {
    const text = "> a `b\nc\n> d` e\n";
    expect(wornIn(text, 2, text.length - 1)).toBe("> **a** `b\nc\n> d` **e**\n");
  });

  it("is left out of a format on one line that widens into bold, which then takes its marks off", () => {
    const text = "a **b** c `d\ne` f\n";
    const from = text.indexOf("b**");
    expect(wornIn(text, from, text.indexOf("\n"))).toBe("a **b c** `d\ne` f\n");
  });
});
