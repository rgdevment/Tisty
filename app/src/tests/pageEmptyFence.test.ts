import { afterEach, describe, expect, it } from "vitest";
import { laidOver } from "../ui/pageLaidOver";
import { fenceBreakAt } from "../ui/pageLists";
import { inCode } from "../ui/pageNodes";
import { sourceAt, spotOf } from "../ui/pageSpots";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;
let menus: boolean[] = [];

afterEach(() => {
  writing?.stop();
  writing = null;
  menus = [];
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
    blocks: (_at, slash) => menus.push(slash),
    acts: () => {},
  });
  writing.drawn();
  return () => text;
};

const typed = (data: string) =>
  document.querySelector(".page")?.dispatchEvent(
    new InputEvent("beforeinput", {
      inputType: "insertText",
      data,
      bubbles: true,
      cancelable: true,
    }),
  );

const caretOn = (selector: string, offset = 0) => {
  document.getSelection()?.collapse(document.querySelector(selector) as Node, offset);
  document.dispatchEvent(new Event("selectionchange"));
};

describe("typing into an empty code block", () => {
  it.each(["code", "pre"])("keeps the closing fence after the words, caret on %s", (on) => {
    const text = opened(
      "```ps1\n```",
      '<figure class="code" data-at="0-10"><pre><code class="language-ps1"></code></pre></figure>',
    );
    caretOn(`.page ${on}`);
    typed("exit 0");
    expect(text()).toBe("```ps1\nexit 0\n```");
  });

  it("keeps the closing fence in the middle of a page", () => {
    const text = opened(
      "Uno\n\n```ps1\n```\n\nDos\n",
      '<p data-at="0-4"><span data-src="0-3">Uno</span></p><figure class="code" data-at="5-15"><pre><code class="language-ps1"></code></pre></figure><p data-at="17-21"><span data-src="17-20">Dos</span></p>',
    );
    caretOn(".page code");
    typed("a");
    expect(text()).toBe("Uno\n\n```ps1\na\n```\n\nDos\n");
  });
});

const ps1 =
  '<figure class="code" data-at="0-10"><pre><code class="language-ps1"></code></pre></figure>';
const bare = '<figure class="code" data-at="0-7"><pre><code></code></pre></figure>';
const middle =
  '<p data-at="0-4"><span data-src="0-3">Uno</span></p><figure class="code" data-at="5-15"><pre><code class="language-ps1"></code></pre></figure><p data-at="17-21"><span data-src="17-20">Dos</span></p>';

describe("typing into an empty code block, other shapes", () => {
  it.each(["code", "pre"])("works without a language, caret on %s", (on) => {
    const text = opened("```\n```", bare);
    caretOn(`.page ${on}`);
    typed("a");
    expect(text()).toBe("```\na\n```");
  });

  it.each(["code", "pre"])("keeps a CRLF body whole, caret on %s", (on) => {
    const text = opened(
      "```ps1\r\n```\r\n",
      '<figure class="code" data-at="0-11"><pre><code class="language-ps1"></code></pre></figure>',
    );
    caretOn(`.page ${on}`);
    typed("a");
    expect(text()).toBe("```ps1\r\na\r\n```\r\n");
  });

  it("keeps the fence in the middle of a page with the caret on pre", () => {
    const text = opened("Uno\n\n```ps1\n```\n\nDos\n", middle);
    caretOn(".page pre");
    typed("a");
    expect(text()).toBe("Uno\n\n```ps1\na\n```\n\nDos\n");
  });

  it("lets a second letter follow the first", () => {
    const text = opened("```ps1\n```", ps1);
    caretOn(".page code");
    typed("a");
    expect(text()).toBe("```ps1\na\n```");
  });

  it("writes over the words chosen in a block of one line", () => {
    const text = opened(
      "```js\nlet a\n```",
      '<figure class="code" data-at="0-15"><pre><code class="language-js" data-src="6-12">let a\n</code></pre></figure>',
    );
    const node = document.querySelector(".page code")?.firstChild as Node;
    const range = document.createRange();
    range.setStart(node, 0);
    range.setEnd(node, 5);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    document.dispatchEvent(new Event("selectionchange"));
    typed("b");
    expect(text()).toBe("```js\nb\n```");
  });

  it("types at the end of the last line of a block with words", () => {
    const text = opened(
      "```js\nlet a\n```",
      '<figure class="code" data-at="0-15"><pre><code class="language-js" data-src="6-12">let a\n</code></pre></figure>',
    );
    const node = document.querySelector(".page code")?.firstChild as Node;
    caretOn(".page code", 1);
    document.getSelection()?.collapse(node, 6);
    document.dispatchEvent(new Event("selectionchange"));
    typed("b");
    expect(text()).toBe("```js\nlet a\nb\n```");
  });

  it("types on the blank line of a block with only that line", () => {
    const text = opened(
      "```\n\n```",
      '<figure class="code" data-at="0-8"><pre><code data-src="4-5">\n</code></pre></figure>',
    );
    const node = document.querySelector(".page code")?.firstChild as Node;
    document.getSelection()?.collapse(node, 0);
    document.dispatchEvent(new Event("selectionchange"));
    typed("a");
    expect(text()).toBe("```\na\n```");
  });
});

describe("pasting into an empty code block", () => {
  it("keeps the closing fence after the pasted words", async () => {
    const text = opened("```ps1\n```", ps1);
    caretOn(".page code");
    const event = new InputEvent("beforeinput", {
      inputType: "insertFromPaste",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(event, "dataTransfer", {
      value: { getData: (kind: string) => (kind === "text/plain" ? "exit 0" : "") },
    });
    document.querySelector(".page")?.dispatchEvent(event);
    for (let turn = 0; turn < 10; turn++) await Promise.resolve();
    expect(text()).toBe("```ps1\nexit 0\n```");
  });
});

describe("fenceBreakAt", () => {
  it.each([
    ["```ps1\n```", 7, "\n"],
    ["```ps1\n```", 0, ""],
    ["```js\nlet a\n```", 12, "\n"],
    ["````\n```\n````", 5, ""],
    ["````\n```\n````", 9, "\n"],
    ["~~~\na\n~~~", 6, "\n"],
    ["~~~\na\n~~~", 0, ""],
    ["```ps1\r\n```\r\n", 8, "\r\n"],
    ["```ps1\n```", 8, ""],
    ["```ps1\n```", 9, ""],
    ["hola\n```\n", 5, ""],
    ["```\nuno\n```\n\n```\n", 14, ""],
    ["```ps1\n  ```", 9, "\n  "],
    ["- ```\n  ```\n", 8, "\n  "],
    ["> ```\n> ```", 8, "\n> "],
  ])("%j at %i gives %j", (text, at, lineBreak) => {
    expect(fenceBreakAt(text, at)).toBe(lineBreak);
  });
});

describe("laidOver in a code block", () => {
  const span = (from: number, to: number) => ({ from, to, insert: "" });

  it("breaks the line before a closing fence", () => {
    const laid = laidOver("```ps1\n```", span(7, 7), "a", true);
    expect(laid.edit.insert).toBe("a\n");
    expect(laid.caret).toBe(8);
  });

  it("breaks the line with CRLF when the text uses it", () => {
    const laid = laidOver("```ps1\r\n```\r\n", span(8, 8), "a", true);
    expect(laid.edit.insert).toBe("a\r\n");
    expect(laid.caret).toBe(9);
  });

  it("adds nothing when the words end a line already", () => {
    expect(laidOver("```ps1\n```", span(7, 7), "a\n", true).edit.insert).toBe("a\n");
  });

  it("adds nothing for a delete", () => {
    expect(laidOver("```js\nlet a\n```", span(6, 12), "", true).edit.insert).toBe("");
  });

  it("adds nothing away from a closing fence", () => {
    expect(laidOver("```js\nlet a\n```", span(9, 9), "x", true).edit.insert).toBe("x");
  });

  it("replaces all the lines of a block", () => {
    const laid = laidOver("```js\nlet a\n```", span(6, 12), "b", true);
    expect(laid.edit.insert).toBe("b\n");
    expect(laid.caret).toBe(7);
  });
});

describe("sourceAt and spotOf on an empty block", () => {
  const drawn = (placed: string) => {
    document.body.innerHTML = `<article class="page">${placed}</article>`;
    return document;
  };

  it.each([
    ["```ps1\n```", ps1, "code", 7],
    ["```ps1\n```", ps1, "pre", 7],
    ["```\n```", bare, "code", 4],
    ["```\n```", bare, "pre", 4],
    ["Uno\n\n```ps1\n```\n\nDos\n", middle, "code", 12],
    ["Uno\n\n```ps1\n```\n\nDos\n", middle, "pre", 12],
    ["```ps1\r\n```\r\n", ps1, "code", 8],
  ])("%j puts the caret in %s at %i", (body, placed, on, at) => {
    const page = drawn(placed);
    const node = page.querySelector(`.page ${on}`) as Node;
    expect(sourceAt(body, node, 0)).toBe(at);
    const spot = spotOf(page, body, at);
    expect(spot?.[0]).toBe(page.querySelector(".page code"));
    expect(spot?.[1]).toBe(0);
  });
});

describe("typing into an empty code block inside other blocks", () => {
  it("keeps the lead of a list item", () => {
    const text = opened(
      "- ```\n  ```\n",
      '<ul data-at="0-12"><li data-at="0-12"><figure class="code" data-at="2-11"><pre><code></code></pre></figure></li></ul>',
    );
    caretOn(".page code");
    typed("a");
    expect(text()).toBe("- ```\n  a\n  ```\n");
  });

  it("keeps the mark of a quote", () => {
    const text = opened(
      "> ```\n> ```\n",
      '<blockquote data-at="0-12"><figure class="code" data-at="2-11"><pre><code></code></pre></figure></blockquote>',
    );
    caretOn(".page code");
    typed("a");
    expect(text()).toBe("> ```\n> a\n> ```\n");
  });

  it("keeps the indent of the closing fence", () => {
    const text = opened(
      "```ps1\n  ```",
      '<figure class="code" data-at="0-12"><pre><code class="language-ps1"></code></pre></figure>',
    );
    caretOn(".page code");
    typed("a");
    expect(text()).toBe("```ps1\n  a\n  ```");
  });

  it("writes with the caret on the figure itself", () => {
    const text = opened("```\n```", bare);
    caretOn(".page figure");
    typed("a");
    expect(text()).toBe("```\na\n```");
  });

  it("writes code literally with the caret on the figure of a block with words", () => {
    const text = opened(
      "```js\nlet a\n```",
      '<figure class="code" data-at="0-15"><pre><code class="language-js" data-src="6-12">let a\n</code></pre></figure>',
    );
    caretOn(".page figure", 0);
    typed("*");
    expect(text()).toBe("```js\n*let a\n```");
  });
});

const keyed = (name: string) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true }));

const asked = (inputType: string) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true }));

describe("a caret on the pre of an empty code block", () => {
  const onPre = () => {
    const text = opened("```ps1\n```", ps1);
    caretOn(".page pre");
    return text;
  };

  it("writes a slash as code and opens no menu", () => {
    const text = onPre();
    typed("/");
    expect(text()).toBe("```ps1\n/\n```");
    expect(menus).toEqual([]);
  });

  it("breaks a line with Enter", () => {
    const text = onPre();
    asked("insertParagraph");
    expect(text()).toBe("```ps1\n\n```");
  });

  it("writes a backtick as it is", () => {
    const text = onPre();
    typed("`");
    expect(text()).toBe("```ps1\n`\n```");
  });

  it("indents with Tab and goes on writing after the indent", () => {
    const text = onPre();
    keyed("Tab");
    expect(text()).toBe("```ps1\n  ```");
    typed("a");
    expect(text()).toBe("```ps1\n  a\n  ```");
  });
});

describe("inCode", () => {
  const within = (html: string, selector: string) => {
    document.body.innerHTML = html;
    return inCode(document.querySelector(selector) as Node);
  };

  it("is true on the pre and on the figure of a code block", () => {
    const html = '<figure class="code"><pre><code></code></pre></figure>';
    expect(within(html, "pre")).toBe(true);
    expect(within(html, "figure")).toBe(true);
  });

  it("is true in inline code", () => {
    expect(within('<p><code contenteditable="false">a</code></p>', "code")).toBe(true);
  });

  it("is false in the front matter", () => {
    expect(within('<dl class="front agent-own"><pre>name: a</pre></dl>', "pre")).toBe(false);
  });
});
