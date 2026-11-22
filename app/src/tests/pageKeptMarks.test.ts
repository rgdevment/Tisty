import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BOLD = "# T\n\nAntes **negrita** después\n";
const BOLD_PLACED =
  '<h1 data-at="0-3"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-32"><span data-src="5-11">Antes </span><strong><span data-src="13-20">negrita</span></strong><span data-src="22-31"> después</span></p>';

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

const wordsIn = (src: string) => document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;

const rangeOf = (from: [Text, number], to: [Text, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  return range;
};

const chose = (range: Range) => {
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
};

type Asked = { data?: string; reach?: Range | null; pasted?: string };

const asked = (inputType: string, { data, reach, pasted }: Asked = {}) => {
  const event = new InputEvent("beforeinput", {
    inputType,
    data: data ?? null,
    bubbles: true,
    cancelable: true,
  });
  const reaches = reach === null ? [] : [reach ?? document.getSelection()?.getRangeAt(0)];
  Object.defineProperty(event, "getTargetRanges", { value: () => reaches });
  if (pasted !== undefined)
    Object.defineProperty(event, "dataTransfer", {
      value: { getData: (kind: string) => (kind === "text/plain" ? pasted : "") },
    });
  document.querySelector(".page")?.dispatchEvent(event);
};

const composed = (kind: string, data: string) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(new CompositionEvent(kind, { data, bubbles: true }));

const acrossTheEnd = () => rangeOf([wordsIn("13-20"), 3], [wordsIn("22-31"), 3]);

describe("words chosen across a format's edge", () => {
  it("keep the format's closing mark when they are deleted", () => {
    const text = opened(BOLD, BOLD_PLACED);
    chose(acrossTheEnd());
    asked("deleteContentBackward");
    expect(text()).toBe("# T\n\nAntes **neg**spués\n");
  });

  it("keep the format's opening mark when they are deleted from before it", () => {
    const text = opened(BOLD, BOLD_PLACED);
    chose(rangeOf([wordsIn("5-11"), 2], [wordsIn("13-20"), 3]));
    asked("deleteContentBackward");
    expect(text()).toBe("# T\n\nAn**rita** después\n");
  });

  it("take the format's marks with them when they held all its words", () => {
    const text = opened(BOLD, BOLD_PLACED);
    chose(rangeOf([wordsIn("13-20"), 0], [wordsIn("22-31"), 3]));
    asked("deleteContentBackward");
    expect(text()).toBe("# T\n\nAntes spués\n");
  });

  it("keep the closing mark after a letter typed over them, so the letter stays in the format", () => {
    const text = opened(BOLD, BOLD_PLACED);
    chose(acrossTheEnd());
    asked("insertText", { data: "X" });
    expect(text()).toBe("# T\n\nAntes **negX**spués\n");
  });

  it("keep the closing mark after words pasted over them", async () => {
    const text = opened(BOLD, BOLD_PLACED);
    chose(acrossTheEnd());
    asked("insertFromPaste", { pasted: "XY" });
    await Promise.resolve();
    await Promise.resolve();
    expect(text()).toBe("# T\n\nAntes **negXY**spués\n");
  });

  it("keep the closing mark when a composition begun over them is given up", () => {
    const text = opened(BOLD, BOLD_PLACED);
    chose(acrossTheEnd());
    composed("compositionstart", "");
    composed("compositionend", "");
    expect(text()).toBe("# T\n\nAntes **neg**spués\n");
  });
});

describe("a line's delete from inside a format", () => {
  it("takes the words before the caret and keeps the mark that opens the format", () => {
    const text = opened(BOLD, BOLD_PLACED);
    document.getSelection()?.collapse(wordsIn("13-20"), 3);
    asked("deleteSoftLineBackward", {
      reach: rangeOf([wordsIn("5-11"), 0], [wordsIn("13-20"), 3]),
    });
    expect(text()).toBe("# T\n\n**rita** después\n");
  });
});

describe("a word's delete across the marks WebKit reaches past", () => {
  it("takes a stop back alone and keeps the bold before it", () => {
    const text = opened(
      "# Hola\n\n我们**公园**。\n",
      '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
        '<p data-at="8-27"><span data-src="8-14">我们</span><strong><span data-src="16-22">公园</span></strong><span data-src="24-27">。</span></p>',
    );
    document.getSelection()?.collapse(wordsIn("24-27"), 1);
    asked("deleteWordBackward", { reach: rangeOf([wordsIn("16-22"), 0], [wordsIn("24-27"), 1]) });
    expect(text()).toBe("# Hola\n\n我们**公园**\n");
  });

  it("takes a stop forward alone and keeps the bold after it", () => {
    const text = opened(
      "# Hola\n\n。**公园**我们\n",
      '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
        '<p data-at="8-28"><span data-src="8-11">。</span><strong><span data-src="13-19">公园</span></strong><span data-src="21-27">我们</span></p>',
    );
    document.getSelection()?.collapse(wordsIn("8-11"), 0);
    asked("deleteWordForward", { reach: rangeOf([wordsIn("8-11"), 0], [wordsIn("13-19"), 2]) });
    expect(text()).toBe("# Hola\n\n**公园**我们\n");
  });
});

describe("a delete whose browser names no words to take", () => {
  const SHORT = "Uno **a** dos\n";
  const SHORT_PLACED =
    '<p data-at="0-14"><span data-src="0-4">Uno </span><strong><span data-src="6-7">a</span></strong><span data-src="9-13"> dos</span></p>';

  it("takes the letter after the format's closing mark, never the mark", () => {
    const text = opened(SHORT, SHORT_PLACED);
    document.getSelection()?.collapse(wordsIn("6-7"), 1);
    asked("deleteContentForward", { reach: rangeOf([wordsIn("6-7"), 1], [wordsIn("9-13"), 0]) });
    expect(text()).toBe("Uno **a**dos\n");
  });

  it("leaves the caret in the format after taking the letter past its closing mark", () => {
    const text = opened(SHORT, SHORT_PLACED);
    document.getSelection()?.collapse(wordsIn("6-7"), 1);
    asked("deleteContentForward", { reach: rangeOf([wordsIn("6-7"), 1], [wordsIn("9-13"), 1]) });
    asked("insertText", { data: "z" });
    expect(text()).toBe("Uno **az**dos\n");
  });

  it("takes the letter after the format's closing mark when the browser's reach holds no words", () => {
    const text = opened(SHORT, SHORT_PLACED);
    const inner = wordsIn("6-7");
    document.getSelection()?.collapse(inner, 1);
    const edge = document.createRange();
    edge.setStart(inner, 1);
    edge.setEnd(inner.parentNode as Node, 1);
    asked("deleteContentForward", { reach: edge });
    expect(text()).toBe("Uno **a**dos\n");
  });

  const ENDED = "Uno **a**\n\nDos\n";
  const ENDED_PLACED =
    '<p data-at="0-10"><span data-src="0-4">Uno </span><strong><span data-src="6-7">a</span></strong></p><p data-at="11-15"><span data-src="11-14">Dos</span></p>';

  it("joins the next block at a line's end when the browser's reach holds no words, keeping the format whole", () => {
    const text = opened(ENDED, ENDED_PLACED);
    const inner = wordsIn("6-7");
    document.getSelection()?.collapse(inner, 1);
    const edge = document.createRange();
    edge.setStart(inner, 1);
    edge.setEnd(inner.parentNode as Node, 1);
    asked("deleteContentForward", { reach: edge });
    expect(text()).toBe("Uno **a**Dos\n");
  });

  it("joins the next block at a line's end when the browser names no reach at all", () => {
    const text = opened(ENDED, ENDED_PLACED);
    document.getSelection()?.collapse(wordsIn("6-7"), 1);
    asked("deleteContentForward", { reach: null });
    expect(text()).toBe("Uno **a**Dos\n");
  });

  it("joins the next item past its mark at an item's end when the browser names no reach", () => {
    const text = opened(
      "- a\n- b\n",
      '<ul data-at="0-8"><li data-at="0-4"><span data-src="2-3">a</span></li><li data-at="4-8"><span data-src="6-7">b</span></li></ul>',
    );
    document.getSelection()?.collapse(wordsIn("2-3"), 1);
    asked("deleteContentForward", { reach: null });
    expect(text()).toBe("- ab\n");
  });

  it("takes an escaped mark whole when the caret stands before its backslash", () => {
    const text = opened(
      "我们公园\\~ fin\n",
      '<p data-at="0-11"><span data-src="0-4">我们公园</span><span data-src="5-10">~ fin</span></p>',
    );
    document.getSelection()?.collapse(wordsIn("0-4"), 4);
    asked("deleteContentForward", { reach: null });
    expect(text()).toBe("我们公园 fin\n");
  });

  it("takes the letter beside the caret when the browser names no reach at all", () => {
    const text = opened(SHORT, SHORT_PLACED);
    document.getSelection()?.collapse(wordsIn("6-7"), 1);
    asked("deleteContentForward", { reach: null });
    expect(text()).toBe("Uno **a**dos\n");
    asked("deleteContentBackward", { reach: null });
    expect(text()).toBe("Uno dos\n");
  });
});

describe("a delete in code", () => {
  it("takes only the letter, as the stars around it are code, not marks", () => {
    const text = opened(
      "```js\na*b*c\n```\n",
      '<figure class="code" data-at="0-16"><pre><code><span data-src="6-11">a*b*c</span></code></pre></figure>',
    );
    const code = wordsIn("6-11");
    document.getSelection()?.collapse(code, 3);
    asked("deleteContentBackward", { reach: rangeOf([code, 2], [code, 3]) });
    expect(text()).toBe("```js\na**c\n```\n");
  });
});

describe("words chosen from inside inline code into a bold", () => {
  it("keep the code's closing mark and the bold's opening mark", () => {
    const text = opened(
      "`code` y **bold**\n",
      '<p data-at="0-18"><code data-src="1-5">code</code><span data-src="6-9"> y </span><strong><span data-src="11-15">bold</span></strong></p>',
    );
    chose(rangeOf([wordsIn("1-5"), 2], [wordsIn("11-15"), 2]));
    asked("deleteContentBackward");
    expect(text()).toBe("`co`**ld**\n");
  });

  it("keep them when a composition begun over them is given up", () => {
    const text = opened(
      "`code` y **bold**\n",
      '<p data-at="0-18"><code data-src="1-5">code</code><span data-src="6-9"> y </span><strong><span data-src="11-15">bold</span></strong></p>',
    );
    chose(rangeOf([wordsIn("1-5"), 2], [wordsIn("11-15"), 2]));
    composed("compositionstart", "");
    composed("compositionend", "");
    expect(text()).toBe("`co`**ld**\n");
  });
});

describe("a delete in inline code", () => {
  it("takes the code's marks with its last letter, as the page draws nothing of them", () => {
    const text = opened(
      "x `a` y\n",
      '<p data-at="0-8"><span data-src="0-2">x </span><code data-src="3-4">a</code><span data-src="5-7"> y</span></p>',
    );
    const code = wordsIn("3-4");
    document.getSelection()?.collapse(code, 1);
    asked("deleteContentBackward", { reach: rangeOf([code, 0], [code, 1]) });
    expect(text()).toBe("x  y\n");
  });

  it("takes two codes whole when the words chosen run from one into the other", () => {
    const text = opened(
      "`a` y `b` fin\n",
      '<p data-at="0-14"><code data-src="1-2">a</code><span data-src="3-6"> y </span><code data-src="7-8">b</code><span data-src="9-13"> fin</span></p>',
    );
    chose(rangeOf([wordsIn("1-2"), 0], [wordsIn("7-8"), 1]));
    asked("deleteContentBackward");
    expect(text()).toBe(" fin\n");
  });
});

describe("words chosen into a format and typed over", () => {
  const MIXED = "x *a* **b** y\n";
  const MIXED_PLACED =
    '<p data-at="0-14"><span data-src="0-2">x </span><em><span data-src="3-4">a</span></em><span data-src="5-6"> </span><strong><span data-src="8-9">b</span></strong><span data-src="11-13"> y</span></p>';
  const SPACED = "a **b c** d\n";
  const SPACED_PLACED =
    '<p data-at="0-12"><span data-src="0-2">a </span><strong><span data-src="4-7">b c</span></strong><span data-src="9-11"> d</span></p>';

  it("take a format whose words were all chosen whole, as a delete does", () => {
    const text = opened(MIXED, MIXED_PLACED);
    chose(rangeOf([wordsIn("3-4"), 0], [wordsIn("8-9"), 1]));
    asked("insertText", { data: "Q" });
    expect(text()).toBe("x *Q* y\n");
  });

  it("join two formats the choice ran between, as a delete does", () => {
    const text = opened(
      "**ab** **cd**\n",
      '<p data-at="0-14"><strong><span data-src="2-4">ab</span></strong><span data-src="6-7"> </span><strong><span data-src="9-11">cd</span></strong></p>',
    );
    chose(rangeOf([wordsIn("2-4"), 1], [wordsIn("9-11"), 1]));
    asked("insertText", { data: "x" });
    expect(text()).toBe("**axd**\n");
  });

  it("open the format again past the spaces its words now start with", () => {
    const text = opened(SPACED, SPACED_PLACED);
    chose(rangeOf([wordsIn("0-2"), 0], [wordsIn("4-7"), 1]));
    asked("insertText", { data: "X" });
    expect(text()).toBe("X **c** d\n");
  });

  it("open the format again past those spaces when they are deleted", () => {
    const text = opened(SPACED, SPACED_PLACED);
    chose(rangeOf([wordsIn("0-2"), 0], [wordsIn("4-7"), 1]));
    asked("deleteContentBackward");
    expect(text()).toBe(" **c** d\n");
  });
});

describe("a key whose browser names no reach at the start of a line's words", () => {
  it("takes nothing of a heading's mark", () => {
    const text = opened(
      "Uno\n\n# Dos\n",
      '<p data-at="0-4"><span data-src="0-3">Uno</span></p><h1 data-at="5-11"><span data-src="7-10">Dos</span></h1>',
    );
    document.getSelection()?.collapse(wordsIn("7-10"), 0);
    asked("deleteContentBackward", { reach: null });
    expect(text()).toBe("Uno\n\n# Dos\n");
  });

  it("takes nothing of an item's mark on a line's delete", () => {
    const text = opened(
      "Uno\n\n- a\n",
      '<p data-at="0-4"><span data-src="0-3">Uno</span></p><ul data-at="5-9"><li data-at="5-9"><span data-src="7-8">a</span></li></ul>',
    );
    document.getSelection()?.collapse(wordsIn("7-8"), 0);
    asked("deleteSoftLineBackward", { reach: null });
    expect(text()).toBe("Uno\n\n- a\n");
  });
});

describe("a key whose browser names no reach at a line's end", () => {
  it("takes nothing past the line, where the page shows nothing to take", () => {
    const text = opened(
      "Uno\n\n[r]: https://x.com\n",
      '<p data-at="0-4"><span data-src="0-3">Uno</span></p>',
    );
    document.getSelection()?.collapse(wordsIn("0-3"), 3);
    asked("deleteContentForward", { reach: null });
    expect(text()).toBe("Uno\n\n[r]: https://x.com\n");
  });
});

describe("Backspace at a block's start joining it to the one above", () => {
  const joinedAt = (words: string, above: string) => {
    const start = wordsIn(words);
    const end = wordsIn(above);
    document.getSelection()?.collapse(start, 0);
    asked("deleteContentBackward", { reach: rangeOf([end, end.length], [start, 0]) });
  };

  it("keeps an equals sign at each side, as they are words and not a mark", () => {
    const text = opened(
      "Total =\n\n= 5\n",
      '<p data-at="0-8"><span data-src="0-7">Total =</span></p><p data-at="9-13"><span data-src="9-12">= 5</span></p>',
    );
    joinedAt("9-12", "0-7");
    expect(text()).toBe("Total == 5\n");
  });

  it("keeps the bold that ends a title written with closing hashes", () => {
    const text = opened(
      "## **T** ##\n\nUno\n",
      '<h2 data-at="0-12"><strong><span data-src="5-6">T</span></strong></h2><p data-at="13-17"><span data-src="13-16">Uno</span></p>',
    );
    joinedAt("13-16", "5-6");
    expect(text()).toBe("## **T**Uno\n");
  });

  it("makes formats nested the same way one, closers and openers met in reverse order", () => {
    const text = opened(
      "**~~a~~**\n\n**~~b~~**\n",
      '<p data-at="0-10"><strong><del><span data-src="4-5">a</span></del></strong></p>' +
        '<p data-at="11-21"><strong><del><span data-src="15-16">b</span></del></strong></p>',
    );
    joinedAt("15-16", "4-5");
    expect(text()).toBe("**~~ab~~**\n");
  });
});
