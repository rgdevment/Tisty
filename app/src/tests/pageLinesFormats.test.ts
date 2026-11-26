import { afterEach, describe, expect, it } from "vitest";
import { applied, type Span } from "../ui/pageEdits";
import { balanced, keptGone, replaced, wornOver } from "../ui/pageMarks";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (body: string, placed: string, drawings: Record<string, string>) => {
  let text = body;
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: (now) => {
      const drawn = drawings[now];
      const sheet = document.querySelector(".page");
      if (drawn === undefined || !sheet) return;
      sheet.innerHTML = drawn;
      writing?.drawn();
    },
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

const settled = async () => {
  for (let turn = 0; turn < 30; turn++) await Promise.resolve();
};

const pressed = (text: string, one: Span, open: string, close = open) => {
  const worn = wornOver(text, one, open, close);
  return worn ? { text: applied(text, worn.change), words: worn.words ?? worn.one } : null;
};

const wornIn = (text: string, from: number, to: number, open = "**") =>
  pressed(text, { from, to }, open)?.text ?? text;

const all = (text: string) => ({ from: 0, to: text.trimEnd().length });

describe("a second press of a format over several lines", () => {
  it.each([["**"], ["*"], ["~~"], ["`"]])(
    "takes %s off every line the first press put it on",
    (open) => {
      const text = "Uno\n\nDos tres\n";
      const first = pressed(text, all(text), open);
      expect(first?.text).toBe(`${open}Uno${open}\n\n${open}Dos tres${open}\n`);
      const second = first && pressed(first.text, first.words, open);
      expect(second?.text).toBe(text);
    },
  );

  it.each([
    ["italic over bold lines", "**Uno**\n\n**Dos tres**\n", "*", "***Uno***\n\n***Dos tres***\n"],
    ["bold over italic lines", "*Uno*\n\n*Dos tres*\n", "**", "***Uno***\n\n***Dos tres***\n"],
    ["italic over a list of stars", "* uno\n* dos\n", "*", "* *uno*\n* *dos*\n"],
    [
      "bold with Windows line ends",
      "Uno\r\n\r\nDos tres\r\n",
      "**",
      "**Uno**\r\n\r\n**Dos tres**\r\n",
    ],
    [
      "bold over a page with code",
      "Words here\n\n```js\na * b\n```\n\nAfter *it* end\n",
      "**",
      "**Words here**\n\n```js\na * b\n```\n\n**After *it* end**\n",
    ],
  ])("takes %s off again", (_, text, open, once) => {
    const from = text.search(/[a-zA-Z]/);
    const first = pressed(text, { from, to: text.trimEnd().length }, open);
    expect(first?.text).toBe(once);
    expect(first && pressed(first.text, first.words, open)?.text).toBe(text);
  });

  it("takes them off the items of a list, past their bullets", () => {
    const text = "- uno\n- dos\n";
    const first = pressed(text, { from: 2, to: 11 }, "**");
    expect(first?.text).toBe("- **uno**\n- **dos**\n");
    expect(first && pressed(first.text, first.words, "**")?.text).toBe(text);
  });

  it("takes them off a choice that starts and ends inside the marks", () => {
    const text = "**Uno**\n\n**Dos**\n";
    expect(wornIn(text, 2, text.indexOf("Dos") + 3)).toBe("Uno\n\nDos\n");
  });

  it("puts them on every line once when only some lines wear them", () => {
    const text = "**Uno**\n\nDos\n";
    expect(wornIn(text, 0, text.trimEnd().length)).toBe("**Uno**\n\n**Dos**\n");
  });

  it.each([
    ["*a* and *b*\n*c* and *d*", "*"],
    ["**a** b **c**\n**d** e **f**", "**"],
    ["`x` maps to `y`\n`z` maps to `w`", "`"],
    ["~~a~~ b ~~c~~\n~~d~~", "~~"],
  ])("never takes off the marks of two formats at a line's edges, in %j", (text, open) => {
    const after = wornIn(text, 0, text.length, open);
    for (const line of text.split("\n")) expect(after).toContain(line);
    expect(after).not.toBe(text);
  });

  it("puts the format on a line whose edges are two of its spans, beside a line without it", () => {
    const text = "x\n\n*a* and *b*\n";
    expect(wornIn(text, 0, text.trimEnd().length, "*")).toBe("*x*\n\n**a* and *b**\n");
  });

  it("reads a line's marks past the code on it, so marks in the code never count", () => {
    const text = "**x `a**b` y**\n\nz\n";
    expect(wornIn(text, 0, text.trimEnd().length)).toBe("**x `a**b` y**\n\n**z**\n");
  });
});

const PLAIN = "# T\n\nUno\n\nDos\n";
const PLAIN_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-9"><span data-src="5-8">Uno</span></p>' +
  '<p data-at="10-14"><span data-src="10-13">Dos</span></p>';
const BOLD = "# T\n\n**Uno**\n\n**Dos**\n";
const BOLD_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-13"><strong><span data-src="7-10">Uno</span></strong></p>' +
  '<p data-at="14-22"><strong><span data-src="16-19">Dos</span></strong></p>';

describe("the format key pressed twice over two paragraphs", () => {
  it("leaves the page as it was, with the same words chosen", async () => {
    const text = opened(PLAIN, PLAIN_PLACED, { [PLAIN]: PLAIN_PLACED, [BOLD]: BOLD_PLACED });
    chose([wordsOf("Uno"), 0], [wordsOf("Dos"), 3]);
    writing?.format("**", "**");
    await settled();
    expect(text()).toBe(BOLD);
    writing?.format("**", "**");
    await settled();
    expect(text()).toBe(PLAIN);
    expect(writing?.choiceHeld()?.words).toBe("Uno\n\nDos");
  });
});

describe("a format over several lines leaves out the lines that hold no words", () => {
  it.each([
    ["an underline of equals", "Title\n=====\n\nWords\n", "**Title**\n=====\n\n**Words**\n"],
    ["an underline of dashes", "Title\n---\n\nWords\n", "**Title**\n---\n\n**Words**\n"],
    ["a rule", "Uno\n\n***\n\nDos\n", "**Uno**\n\n***\n\n**Dos**\n"],
    [
      "a table",
      "Words\n\n| a | b |\n|---|:-:|\n| 1 | 2 |\n\nAfter\n",
      "**Words**\n\n| a | b |\n|---|:-:|\n| 1 | 2 |\n\n**After**\n",
    ],
    [
      "a table written without its outer pipes",
      "Words\n\na | b\n--|--\n1 | 2\n\nAfter\n",
      "**Words**\n\na | b\n--|--\n1 | 2\n\n**After**\n",
    ],
    [
      "a table in a quote",
      "> Words\n>\n> | a | b |\n> |---|---|\n> | 1 | 2 |\n",
      "> **Words**\n>\n> | a | b |\n> |---|---|\n> | 1 | 2 |\n",
    ],
    [
      "a formula over lines",
      "Words\n\n$$\nx^2\n$$\n\nAfter\n",
      "**Words**\n\n$$\nx^2\n$$\n\n**After**\n",
    ],
    ["a formula among words", "a $$x\ny$$ b\n", "**a** $$x\ny$$ **b**\n"],
    [
      "an underline written with Windows line ends",
      "Title\r\n---\r\n\r\nWords\r\n",
      "**Title**\r\n---\r\n\r\n**Words**\r\n",
    ],
    [
      "a table written with Windows line ends",
      "Words\r\n\r\n| a |\r\n|---|\r\n| 1 |\r\n\r\nAfter\r\n",
      "**Words**\r\n\r\n| a |\r\n|---|\r\n| 1 |\r\n\r\n**After**\r\n",
    ],
    [
      "a formula written with Windows line ends",
      "Words\r\n\r\n$$\r\nx^2\r\n$$\r\n\r\nAfter\r\n",
      "**Words**\r\n\r\n$$\r\nx^2\r\n$$\r\n\r\n**After**\r\n",
    ],
  ])("as %s", (_, text, expected) => {
    expect(wornIn(text, 0, text.trimEnd().length)).toBe(expected);
  });

  it("wraps words with bars in them when no row of dashes makes them a table", () => {
    const text = "a | b\nc | d\n\ne\n";
    expect(wornIn(text, 0, text.trimEnd().length)).toBe("**a | b**\n**c | d**\n\n**e**\n");
  });
});

describe("keys over a choice that ends past inline code running over a line break", () => {
  const text = "Use `a\nb*c` and x* y\n";
  const gone = { from: 0, to: text.indexOf("and") };

  it("delete it whole, as no star in the code opens a format", () => {
    expect(applied(text, keptGone(text, gone, null))).toBe("and x* y\n");
  });

  it("type over it, as no star in the code opens a format", () => {
    expect(applied(text, replaced(text, gone, "Z").edit)).toBe("Zand x* y\n");
  });

  it("never pair the code's closing tick with a tick after it on its line", () => {
    const said = "Use `a\nb` and `*c` x* y\n";
    const to = said.indexOf("x*");
    expect(applied(said, keptGone(said, { from: 0, to }, null))).toBe("x* y\n");
  });

  it("read the items of a numbered list each on its own line", () => {
    const said = "1. a `x\n2. b *c* d\n3. e` f\n";
    const from = said.indexOf("*c");
    expect(applied(said, keptGone(said, { from, to: from + 2 }, null))).toBe(
      "1. a `x\n2. b  d\n3. e` f\n",
    );
  });

  it("pair a mark on a line before the code once, past a line with no code", () => {
    const said = "p *q* r\ns `t\nu` v\n";
    const gone = { from: said.indexOf("q"), to: said.indexOf("s") + 1 };
    expect(applied(said, keptGone(said, gone, null))).toBe("p  `t\nu` v\n");
    expect(applied(said, replaced(said, gone, "X").edit)).toBe("p *X* `t\nu` v\n");
  });

  it("read a numbered line under a bullet as words of the item, as it cannot open a list there", () => {
    const said = "- a `b\n  2. c` d\n";
    expect(wornIn(said, 2, said.trimEnd().length)).toBe("- **a** `b\n  2. c` **d**\n");
  });

  it("read a line inside that code as code", () => {
    const inner = "a `b\nc *d* e\nf` g\n";
    const from = inner.indexOf("*d");
    expect(applied(inner, keptGone(inner, { from, to: from + 2 }, null))).toBe(
      "a `b\nc * e\nf` g\n",
    );
  });
});

describe("the partner of an odd mark in a format or a cut", () => {
  const text = "a *b c\n\nlater *x* y\n";

  it("is looked for only in the paragraph the choice ends in", () => {
    expect(wornIn(text, 0, 6, "*")).toBe("*a b c*\n\nlater *x* y\n");
    expect(balanced(text, { from: 0, to: 6 })).toEqual({ from: 0, to: 6 });
  });

  it("is still found on the next line of the same paragraph", () => {
    expect(balanced("a *b\nc* d\n", { from: 0, to: 4 })).toEqual({ from: 0, to: 7 });
  });
});
