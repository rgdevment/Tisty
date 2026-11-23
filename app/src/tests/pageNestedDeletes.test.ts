import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const WORDS = '<span data-src="2-9">uno dos</span>';

const NESTED = {
  code: {
    body: "- uno dos\n  ```js\n  x\n  ```\n",
    placed: `<ul data-at="0-28"><li data-at="0-28">${WORDS}<figure class="code" data-at="12-27"><pre><code class="language-js" data-src="20-22">x\n</code></pre></figure></li></ul>`,
    inside: "x\n",
  },
  table: {
    body: "- uno dos\n  | a |\n  |---|\n  | 1 |\n",
    placed: `<ul data-at="0-34"><li data-at="0-34">${WORDS}<div class="table" data-at="12-34"><table><colgroup><col></colgroup><thead><tr><th scope="col">a</th></tr></thead><tbody><tr><td>1</td></tr></tbody></table></div></li></ul>`,
    inside: "a",
  },
  diagram: {
    body: "- uno dos\n  ```mermaid\n  flowchart LR\n  ```\n",
    placed: `<ul data-at="0-44"><li data-at="0-44">${WORDS}<figure class="diagram" data-at="12-43"><pre><code class="language-mermaid">flowchart LR\n</code></pre></figure></li></ul>`,
    inside: "flowchart LR\n",
  },
  formula: {
    body: "- uno dos\n  ```math\n  x\n  ```\n",
    placed: `<ul data-at="0-30"><li data-at="0-30">${WORDS}<div class="math" data-at="12-29"><math display="block"><mi>x</mi></math></div></li></ul>`,
    inside: "x",
  },
  callout: {
    body: "- uno dos\n  > [!NOTE]\n  > nota\n",
    placed: `<ul data-at="0-31"><li data-at="0-31">${WORDS}<blockquote class="callout" data-callout="note" data-at="12-31"><p data-at="26-31"><span data-src="26-30">nota</span></p></blockquote></li></ul>`,
    inside: "nota",
  },
  quotedCode: {
    body: "- uno dos\n  > ```\n  > x\n  > ```\n",
    placed: `<ul data-at="0-32"><li data-at="0-32">${WORDS}<blockquote data-at="12-32"><figure class="code" data-at="14-31"><pre><code data-src="22-24">x\n</code></pre></figure></blockquote></li></ul>`,
    inside: "x\n",
  },
};

type Shown = { readonly body: string; readonly placed: string };

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = ({ body, placed }: Shown) => {
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
  return () => text;
};

const words = (said: string) => {
  const walk = document.createTreeWalker(
    document.querySelector(".page") as Node,
    NodeFilter.SHOW_TEXT,
  );
  for (let node = walk.nextNode(); node; node = walk.nextNode())
    if ((node as Text).data === said) return node as Text;
  throw new Error(`no ${said} on the page`);
};

const between = (from: [Node, number], to: [Node, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  return range;
};

const deleted = (reach: Range, back = false) => {
  const sheet = document.querySelector(".page");
  const pressed = back ? "Backspace" : "Delete";
  const key = new KeyboardEvent("keydown", { key: pressed, bubbles: true, cancelable: true });
  sheet?.dispatchEvent(key);
  if (key.defaultPrevented) return;
  const inputType = back ? "deleteContentBackward" : "deleteContentForward";
  const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
  sheet?.dispatchEvent(event);
};

const SHAPES = Object.entries(NESTED);

describe("a Delete at the end of a list item's own line", () => {
  it.each(SHAPES)("leaves a nested %s whole", (_, shown) => {
    const text = opened(shown);
    const end: [Node, number] = [words("uno dos"), 7];
    document.getSelection()?.collapse(...end);
    deleted(between(end, [words(shown.inside), 0]));
    expect(text()).toBe(shown.body);
  });

  it.each(SHAPES)("leaves a nested %s whole from a choice of the break into it", (_, shown) => {
    const text = opened(shown);
    const chosen = between([words("uno dos"), 7], [words(shown.inside), 0]);
    document.getSelection()?.setBaseAndExtent(words("uno dos"), 7, words(shown.inside), 0);
    deleted(chosen);
    expect(text()).toBe(shown.body);
  });

  it.each(SHAPES)("leaves a nested %s whole from a Backspace on that choice", (_, shown) => {
    const text = opened(shown);
    const chosen = between([words("uno dos"), 7], [words(shown.inside), 0]);
    document.getSelection()?.setBaseAndExtent(words("uno dos"), 7, words(shown.inside), 0);
    deleted(chosen, true);
    expect(text()).toBe(shown.body);
  });
});

const QUOTED = {
  afterWords: {
    body: "- uno dos\n  > cita\n  > mas\n",
    placed: `<ul data-at="0-27"><li data-at="0-27">${WORDS}<blockquote data-at="12-27"><p data-at="14-27"><span data-src="14-18">cita</span><br data-src="18-19" contenteditable="false"><span data-src="23-26">mas</span></p></blockquote></li></ul>`,
  },
  afterTag: {
    body: "- uno #eti\n  > cita\n  > mas\n",
    placed:
      '<ul data-at="0-28"><li data-at="0-28"><span data-src="2-6">uno </span><span class="tag" data-tag="eti" data-src="6-10" contenteditable="false">#eti</span>' +
      '<blockquote data-at="13-28"><p data-at="15-28"><span data-src="15-19">cita</span><br data-src="19-20" contenteditable="false"><span data-src="24-27">mas</span></p></blockquote></li></ul>',
  },
};

describe("a Delete at the end of a list item's own line before a nested quote", () => {
  it("joins the quote's first line after a piece, as it does after words", () => {
    const words = opened(QUOTED.afterWords);
    const end: [Node, number] = [document.querySelector('[data-src="2-9"]')?.firstChild as Node, 7];
    document.getSelection()?.collapse(...end);
    deleted(between(end, [document.querySelector('[data-src="14-18"]')?.firstChild as Node, 0]));
    expect(words()).toBe("- uno doscita\n  > mas\n");
    writing?.stop();
    const tagged = opened(QUOTED.afterTag);
    const tag = document.querySelector(".tag") as Element;
    document.getSelection()?.collapse(tag.parentNode as Node, 2);
    expect(tag.nextSibling?.textContent).toBe("​");
    deleted(
      between(
        [tag.parentNode as Node, 2],
        [document.querySelector('[data-src="15-19"]')?.firstChild as Node, 0],
      ),
    );
    expect(tagged()).toBe("- uno #eticita\n  > mas\n");
  });
});
