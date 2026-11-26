import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { checkOf, type PlacedNear, type Sent } from "../placing";
import { matched } from "../ui/pagePatching";
import { placeOf, placeSaid } from "../ui/pagePlaces";
import { type PageWriting, writtenOn } from "../ui/pageWriting";
import { near as nearAnswer, type Span } from "./nearing";

const BYTES = new TextEncoder();
const SWITCH = "tisty.nativeTyping";

// A toy core: blocks apart by a blank line, every one drawn as a paragraph of one plain span.
const blocksOf = (text: string) => text.replace(/\n$/, "").split("\n\n");

const spansOf = (blocks: readonly string[]): Span[] => {
  let at = 0;
  return blocks.map((one) => {
    const from = at;
    at += BYTES.encode(one).length + 2;
    return [from, from + BYTES.encode(one).length];
  });
};

const drawnP = (said: string, [from, to]: Span) =>
  `<p data-at="${from}-${to}"><span data-src="${from}-${to}">${said}</span></p>`;

const drawnAs = (text: string, first: number, last: number) => {
  const blocks = blocksOf(text);
  const spans = spansOf(blocks);
  return nearAnswer({
    spans,
    from: first,
    to: last,
    html: blocks
      .slice(first, last)
      .map((one, at) => drawnP(one, spans[first + at]))
      .join(""),
  });
};

const PARAGRAPHS = ["Uno ñandú 漢字 fin", "Tercero"];
const BODY = `${PARAGRAPHS.join("\n\n")}\n`;
const PLACED = spansOf(PARAGRAPHS)
  .map((span, at) => drawnP(PARAGRAPHS[at], span))
  .join("");

type Core = (sent: Sent, from: number, to: number) => Promise<PlacedNear>;

let writing: PageWriting | null = null;
let text = BODY;
let redraws: string[] = [];
let shownAt: string[] = [];
let wrote: string[] = [];
let asked: Sent[] = [];
let core: Core = (sent) => Promise.resolve(blockwise(sent));

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
  document.documentElement.classList.remove("untitled");
  localStorage.removeItem(SWITCH);
  vi.useRealTimers();
});

const opened = (html = PLACED, body = BODY, said: string | null = "on") => {
  writing?.stop();
  text = body;
  redraws = [];
  shownAt = [];
  wrote = [];
  asked = [];
  core = (sent) => Promise.resolve(blockwise(sent));
  if (said === null) localStorage.removeItem(SWITCH);
  else localStorage.setItem(SWITCH, said);
  document.body.innerHTML = `<article class="page">${html}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
      wrote.push(next);
    },
    shown: (now) => shownAt.push(now),
    redraw: (now) => redraws.push(now),
    near: (sent, from, to) => {
      asked.push(sent);
      return core(sent, from, to);
    },
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  localStorage.removeItem(SWITCH);
};

// The core answers with the one block the key reached, drawn as it draws plain words.
function blockwise(sent: Sent): PlacedNear {
  const now = blocksOf(sent.body);
  const was = blocksOf(BODY);
  const at = now.findIndex((one, nth) => one !== was[nth]);
  const first = Math.max(at, 0);
  return drawnAs(sent.body, first, first + 1);
}

const page = () => document.querySelector(".page") as Element;
const paragraphs = () => Array.from(document.querySelectorAll(".page > p"));
const wordsOf = (block: Element) => block.querySelector("span[data-src]")?.firstChild as Text;

const caretAt = (node: Node, offset: number) => document.getSelection()?.collapse(node, offset);

const asking = (inputType: string, data: string | null = null, more: InputEventInit = {}) => {
  const event = new InputEvent("beforeinput", {
    inputType,
    data,
    bubbles: true,
    cancelable: true,
    ...more,
  });
  const taken = page().dispatchEvent(event);
  return { event, taken };
};

const written = (inputType = "insertText") =>
  page().dispatchEvent(new InputEvent("input", { bubbles: true, inputType }));

// jsdom does not write for the browser: the node is changed here as a browser would, then the page is told.
const typedNatively = (node: Text, at: number, data: string) => {
  caretAt(node, at);
  const { taken } = asking("insertText", data);
  if (!taken) return false;
  node.insertData(at, data);
  caretAt(node, at + data.length);
  written();
  return true;
};

const rangeOf = (node: Node, from: number, to: number) => {
  const range = document.createRange();
  range.setStart(node, from);
  range.setEnd(node, to);
  return range;
};

const deleting = (inputType: string, node: Text, from: number, to: number, aimed = true) => {
  const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
  if (aimed)
    Object.defineProperty(event, "getTargetRanges", { value: () => [rangeOf(node, from, to)] });
  return { event, taken: page().dispatchEvent(event) };
};

const key = (name: string, more: KeyboardEventInit = {}) =>
  page().dispatchEvent(
    new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true, ...more }),
  );

const rest = (ms = 250) => vi.advanceTimersByTimeAsync(ms);

describe("the switch", () => {
  it("leaves the key to the slow way when the switch is off, as before", () => {
    opened(PLACED, BODY, null);
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    const { taken } = asking("insertText", "x");
    expect(taken).toBe(false);
    expect(text).toBe(BODY.replace("Uno ", "Unox "));
  });

  it("is read once, when the writer is made", () => {
    opened();
    localStorage.setItem(SWITCH, "off");
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
  });

  it("is off when the switch says anything but on or everywhere", () => {
    opened(PLACED, BODY, "yes");
    caretAt(wordsOf(paragraphs()[0]), 3);
    expect(asking("insertText", "x").taken).toBe(false);
  });

  it("is on with everywhere", () => {
    opened(PLACED, BODY, "everywhere");
    caretAt(wordsOf(paragraphs()[0]), 3);
    expect(asking("insertText", "x").taken).toBe(true);
  });
});

describe("a letter the browser writes in a plain paragraph", () => {
  it("is not held back, and reaches the source where it was typed once the browser has written it", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    expect(typedNatively(words, 4, "s")).toBe(true);
    expect(text).toBe(BODY.replace("Uno ñandú", "Uno sñandú"));
    expect(wrote).toEqual([text]);
    expect(redraws).toEqual([]);
    expect(wordsOf(paragraphs()[0])).toBe(words);
    expect(words.data).toBe("Uno sñandú 漢字 fin");
  });

  it("is taken back with Ctrl and Z", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "x");
    expect(text).not.toBe(BODY);
    key("z", { ctrlKey: true });
    expect(text).toBe(BODY);
    key("y", { ctrlKey: true });
    expect(text).toBe(BODY.replace("Uno ", "Unox "));
  });

  it("moves the place of its span and its block by the bytes typed, a letter of two or three bytes counting so", () => {
    opened();
    const [first, second] = paragraphs();
    const words = wordsOf(first);
    typedNatively(words, 3, "a");
    expect(placeOf(first)).toEqual([0, 23]);
    expect(placeOf(first.querySelector("span"), "data-src")).toEqual([0, 23]);
    typedNatively(words, 4, "ñ");
    expect(placeOf(first.querySelector("span"), "data-src")).toEqual([0, 25]);
    typedNatively(words, 5, "漢");
    expect(placeOf(first)).toEqual([0, 28]);
    expect(placeOf(first.querySelector("span"), "data-src")).toEqual([0, 28]);
    expect(BYTES.encode(text).length).toBe(BYTES.encode(BODY).length + 6);
    expect(second).toBeDefined();
  });

  it("reads the places of the later blocks moved, though they are written only when the keys rest", () => {
    opened();
    const [first, second] = paragraphs();
    typedNatively(wordsOf(first), 3, "漢");
    expect(second.getAttribute("data-at")).toBe("24-31");
    expect(placeOf(second)).toEqual([27, 34]);
    expect(placeOf(second.querySelector("span"), "data-src")).toEqual([27, 34]);
  });

  it("keeps the caret where the browser put it", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "x");
    const selection = document.getSelection();
    expect(selection?.anchorNode).toBe(words);
    expect(selection?.anchorOffset).toBe(4);
  });

  it("writes into a block after the first at the right place of the source", () => {
    opened();
    const words = wordsOf(paragraphs()[1]);
    expect(typedNatively(words, 7, "s")).toBe(true);
    expect(text).toBe(BODY.replace("Tercero", "Terceros"));
    expect(placeOf(paragraphs()[1])).toEqual([24, 32]);
  });
});

describe("the core told after the keys rest", () => {
  it("hears of three letters once, 250 ms after the last, as one edit", async () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "a");
    await rest(100);
    typedNatively(words, 4, "b");
    await rest(100);
    typedNatively(words, 5, "c");
    await rest(249);
    expect(asked).toHaveLength(0);
    await rest(1);
    expect(asked).toHaveLength(1);
    const [sent] = asked;
    expect(sent.body).toBe(text);
    expect(sent.edit).toEqual({ from: 3, to: 3, insert: "abc" });
  });

  it("counts the rest from the last native key, not the first", async () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "a");
    await rest(200);
    typedNatively(words, 4, "b");
    await rest(200);
    expect(asked).toHaveLength(0);
    await rest(50);
    expect(asked).toHaveLength(1);
  });

  it("only takes the core's places when it draws the block alike, leaving the nodes and the caret", async () => {
    opened();
    const [first, second] = paragraphs();
    const words = wordsOf(first);
    const other = wordsOf(second);
    typedNatively(words, 3, "a");
    typedNatively(words, 4, "b");
    typedNatively(words, 5, "c");
    await rest(250);
    await rest(0);
    expect(asked).toHaveLength(1);
    const now = paragraphs();
    expect(now[0]).toBe(first);
    expect(now[1]).toBe(second);
    expect(wordsOf(first)).toBe(words);
    expect(wordsOf(second)).toBe(other);
    expect(words.isConnected).toBe(true);
    expect(words.data).toBe("Unoabc ñandú 漢字 fin");
    expect(document.getSelection()?.anchorNode).toBe(words);
    expect(document.getSelection()?.anchorOffset).toBe(6);
    expect(first.getAttribute("data-at")).toBe("0-25");
    expect(first.querySelector("span")?.getAttribute("data-src")).toBe("0-25");
    expect(second.getAttribute("data-at")).toBe("27-34");
    expect(redraws).toEqual([]);
  });

  it("takes a block over when the core draws it differently", async () => {
    opened();
    const [first] = paragraphs();
    const words = wordsOf(first);
    core = (sent) => {
      const spans = spansOf(blocksOf(sent.body));
      const [from, to] = spans[0];
      return Promise.resolve(
        nearAnswer({
          spans,
          from: 0,
          to: 1,
          html:
            `<p data-at="${from}-${to}"><span data-src="${from}-${from + 4}">Unoa</span>` +
            `<strong><span data-src="${from + 6}-${to - 2}">xyz</span></strong></p>`,
        }),
      );
    };
    typedNatively(words, 3, "a");
    await rest(250);
    await rest(0);
    expect(asked).toHaveLength(1);
    const now = paragraphs()[0];
    expect(now.querySelector("strong")).not.toBeNull();
    expect(words.isConnected).toBe(false);
    expect(redraws).toEqual([]);
  });

  it("ignores an answer drawn for text that moved on", async () => {
    opened();
    const [first] = paragraphs();
    const words = wordsOf(first);
    let answer: ((one: PlacedNear) => void) | undefined;
    let waiting: Sent | undefined;
    core = (sent) => {
      waiting = sent;
      return new Promise((resolve) => {
        answer = resolve;
      });
    };
    typedNatively(words, 3, "a");
    await rest(250);
    expect(waiting?.body).toBe(text);
    typedNatively(words, 4, "b");
    answer?.(blockwise(waiting as Sent));
    await rest(0);
    expect(redraws).toEqual([]);
    expect(wordsOf(first)).toBe(words);
    expect(words.data).toBe("Unoab ñandú 漢字 fin");
    await rest(250);
    expect(asked).toHaveLength(2);
    expect(asked[1].body).toBe(text);
  });
});

describe("a key that is not the browser's, after letters it wrote", () => {
  it("lets the core hear of the letters before the key, and ends with the right text", async () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "a");
    typedNatively(words, 4, "b");
    expect(asked).toHaveLength(0);
    const { taken } = asking("insertText", "*");
    expect(taken).toBe(false);
    await rest(0);
    await vi.waitFor(() => expect(asked.length).toBeGreaterThanOrEqual(2));
    expect(asked[0].body).toBe(BODY.replace("Uno ", "Unoab "));
    expect(asked[0].edit).toEqual({ from: 3, to: 3, insert: "ab" });
    expect(asked[1].body).toBe(text);
    expect(text).toContain("*");
    expect(text.replace(/\*/g, "")).toBe(BODY.replace("Uno ", "Unoab "));
    await rest(500);
    expect(asked.map((one) => one.body)[0]).not.toContain("*");
  });

  it("does so for Enter too", async () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "a");
    typedNatively(words, 4, "b");
    caretAt(words, 5);
    const { taken } = asking("insertParagraph");
    expect(taken).toBe(false);
    await rest(0);
    await vi.waitFor(() => expect(asked.length).toBeGreaterThanOrEqual(2));
    expect(asked[0].body).toBe(BODY.replace("Uno ", "Unoab "));
    expect(asked[1].body).toBe(text);
    expect(text).toBe("Unoab\n\n ñandú 漢字 fin\n\nTercero\n");
  });
});

describe("a key the browser does not write", () => {
  const withWords = (html = PLACED, body = BODY) => {
    opened(html, body);
    return wordsOf(paragraphs()[0]);
  };

  it.each(["*", "#", "[", ":", "/", "-", "_", "\\", ">", "`", "|", "!", "$", "&", "<", "@", "="])(
    "keeps the mark letter %s for the slow way",
    (letter) => {
      const words = withWords();
      caretAt(words, 5);
      expect(asking("insertText", letter).taken).toBe(false);
      expect(text).not.toBe(BODY);
    },
  );

  it("keeps a space at the start of its span", () => {
    const words = withWords();
    caretAt(words, 0);
    expect(asking("insertText", " ").taken).toBe(false);
  });

  it("writes a space inside plain words itself", () => {
    const words = withWords();
    expect(typedNatively(words, 5, " ")).toBe(true);
    expect(text).toBe(BODY.replace("Uno ñ", "Uno ñ "));
  });

  it("keeps a space at the edge of bold", () => {
    const html =
      '<p data-at="0-16"><span data-src="0-4">Uno </span><strong><span data-src="6-11">negra</span></strong><span data-src="13-16"> fi</span></p>';
    const body = "Uno **negra** fi\n";
    opened(html, body);
    const bold = () => document.querySelector("strong span")?.firstChild as Text;
    caretAt(bold(), 5);
    expect(asking("insertText", " ").taken).toBe(false);
    opened(html, body);
    caretAt(bold(), 0);
    expect(asking("insertText", " ").taken).toBe(false);
    opened(html, body);
    caretAt(bold(), 2);
    expect(asking("insertText", " ").taken).toBe(true);
  });

  it("keeps a space after a number and a dot at a line's start", () => {
    opened('<p data-at="0-2"><span data-src="0-2">1.</span></p>', "1.\n");
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 2);
    expect(asking("insertText", " ").taken).toBe(false);
    opened('<p data-at="0-2"><span data-src="0-2">1)</span></p>', "1)\n");
    caretAt(wordsOf(paragraphs()[0]), 2);
    expect(asking("insertText", " ").taken).toBe(false);
    opened('<p data-at="0-2"><span data-src="0-2">a.</span></p>', "a.\n");
    caretAt(wordsOf(paragraphs()[0]), 2);
    expect(asking("insertText", " ").taken).toBe(true);
  });

  it("keeps a selection that spans two spans", () => {
    const html =
      '<p data-at="0-16"><span data-src="0-4">Uno </span><strong><span data-src="6-11">negra</span></strong><span data-src="13-16"> fi</span></p>';
    opened(html, "Uno **negra** fi\n");
    const one = document.querySelector("p > span")?.firstChild as Node;
    const two = document.querySelector("strong span")?.firstChild as Node;
    document.getSelection()?.setBaseAndExtent(one, 1, two, 2);
    expect(asking("insertText", "x").taken).toBe(false);
  });

  it("keeps the key of a composition to the browser, and what follows one just begun to the slow way", () => {
    const words = withWords();
    caretAt(words, 3);
    const composed = asking("insertCompositionText", "x", { isComposing: true });
    expect(composed.taken).toBe(true);
    expect(text).toBe(BODY);
    page().dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true }));
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(false);
  });

  it("keeps every key on a task page", () => {
    const words = withWords();
    document.documentElement.classList.add("untitled");
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(false);
  });

  it("keeps a letter after a format asked for with Ctrl and B", () => {
    const words = withWords();
    caretAt(words, 3);
    key("b", { ctrlKey: true });
    expect(asking("insertText", "x").taken).toBe(false);
    expect(text).toContain("**");
  });

  it("keeps words whose source is written otherwise than shown, as an escaped mark", () => {
    opened('<p data-at="0-6"><span data-src="0-6">a * b</span></p>', "a \\* b\n");
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 1);
    expect(asking("insertText", "x").taken).toBe(false);
    expect(text).toBe("ax \\* b\n");
  });

  it("keeps a span with another attribute, more than one child or none of the paragraphs' kind", () => {
    opened('<p data-at="0-3"><span data-src="0-3" class="x">abc</span></p>', "abc\n");
    caretAt(wordsOf(paragraphs()[0]), 1);
    expect(asking("insertText", "x").taken).toBe(false);
    opened('<div data-at="0-3"><span data-src="0-3">abc</span></div>', "abc\n");
    caretAt(document.querySelector("span")?.firstChild as Text, 1);
    expect(asking("insertText", "x").taken).toBe(false);
    opened('<p data-at="0-3"><code><span data-src="0-3">abc</span></code></p>', "abc\n");
    caretAt(document.querySelector("span")?.firstChild as Text, 1);
    expect(asking("insertText", "x").taken).toBe(false);
  });

  it("keeps a key while the slow way still draws", async () => {
    const words = withWords();
    core = () => new Promise(() => {});
    caretAt(words, 3);
    expect(asking("insertText", "*").taken).toBe(false);
    await rest(0);
    expect(asked).toHaveLength(1);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(false);
  });

  it("keeps a kind of input it does not know", () => {
    const words = withWords();
    caretAt(words, 3);
    expect(asking("insertFromPaste", "xy").taken).toBe(false);
  });
});

describe("a space the browser writes as a non-breaking one", () => {
  const nbsp = " ";

  it("still writes natively over words the page shows with it", () => {
    opened(
      '<p data-at="0-7"><span data-src="0-7">Uno dos</span></p>'.replace(
        "Uno dos",
        `Uno${nbsp}dos`,
      ),
      "Uno dos\n",
    );
    const words = wordsOf(paragraphs()[0]);
    expect(typedNatively(words, 7, "x")).toBe(true);
    expect(text).toBe("Uno dosx\n");
  });

  it("stays a no-break space in the source when that is what was typed", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    expect(typedNatively(words, 5, nbsp)).toBe(true);
    expect(text).toBe(BODY.replace("Uno ñ", `Uno ñ${nbsp}`));
  });

  it("lands as a plain space in the source when a plain space was typed and the browser shows it as one", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 5);
    expect(asking("insertText", " ").taken).toBe(true);
    words.insertData(5, nbsp);
    caretAt(words, 6);
    written();
    expect(text).toBe(BODY.replace("Uno ñ", "Uno ñ "));
  });
});

describe("blanks the browser tidies", () => {
  it("sets the node back to what the key meant and writes that", () => {
    opened('<p data-at="0-4"><span data-src="0-4">x  y</span></p>', "x  y\n");
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 2);
    expect(asking("insertText", "n").taken).toBe(true);
    words.data = "x ny";
    caretAt(words, 3);
    written();
    expect(words.data).toBe("x n y");
    expect(text).toBe("x n y\n");
    expect(document.getSelection()?.anchorNode).toBe(words);
    expect(document.getSelection()?.anchorOffset).toBe(3);
    expect(redraws).toEqual([]);
  });
});

describe("what the browser did that cannot be read back", () => {
  it("draws the block again, the source untouched, when a line break comes into the span", () => {
    opened();
    const [first] = paragraphs();
    const words = wordsOf(first);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    words.insertData(3, "x");
    first.querySelector("span")?.appendChild(document.createElement("br"));
    written();
    expect(wrote).toEqual([]);
    expect(text).toBe(BODY);
    expect(redraws).toEqual([BODY]);
  });

  it("draws it again when a second text node comes into the span", () => {
    opened();
    const [first] = paragraphs();
    const words = wordsOf(first);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    words.insertData(3, "x");
    first.querySelector("span")?.appendChild(document.createTextNode("z"));
    written();
    expect(wrote).toEqual([]);
    expect(redraws).toEqual([BODY]);
  });

  it("draws it again when the words changed meanwhile", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    words.insertData(3, "x");
    text = `${text}more\n`;
    written();
    expect(wrote).toEqual([]);
    expect(redraws).toEqual([text]);
  });

  it("draws it again when the node left the page", () => {
    opened();
    const [first] = paragraphs();
    const words = wordsOf(first);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    first.querySelector("span")?.replaceChildren(document.createTextNode("Unox ñandú 漢字 fin"));
    written();
    expect(wrote).toEqual([]);
    expect(redraws).toEqual([BODY]);
  });

  it("does nothing for an input it did not let through", () => {
    opened();
    written();
    expect(wrote).toEqual([]);
    expect(redraws).toEqual([]);
  });

  it("sets the words to what the key meant when the browser left them as they were", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    written();
    expect(words.data).toBe("Unox ñandú 漢字 fin");
    expect(text).toBe(BODY.replace("Uno ", "Unox "));
  });
});

describe("deleting with the browser", () => {
  it("takes a letter from the middle of a word with Backspace, by its target range", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 6);
    const { taken } = deleting("deleteContentBackward", words, 5, 6);
    expect(taken).toBe(true);
    words.deleteData(5, 1);
    caretAt(words, 5);
    written("deleteContentBackward");
    expect(text).toBe(BODY.replace("ñandú", "ñndú"));
    expect(placeOf(paragraphs()[0])).toEqual([0, 21]);
  });

  it("counts the bytes of a letter of more than one taken away", () => {
    opened();
    const [first, second] = paragraphs();
    const words = wordsOf(first);
    caretAt(words, 5);
    expect(deleting("deleteContentBackward", words, 4, 5).taken).toBe(true);
    words.deleteData(4, 1);
    caretAt(words, 4);
    written("deleteContentBackward");
    expect(text).toBe(BODY.replace("ñ", ""));
    expect(placeOf(first)).toEqual([0, 20]);
    expect(placeOf(second)).toEqual([22, 29]);
  });

  it("takes the letter after the caret with Delete", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 6);
    expect(deleting("deleteContentForward", words, 6, 7).taken).toBe(true);
    words.deleteData(6, 1);
    written("deleteContentForward");
    expect(text).toBe(BODY.replace("ñandú", "ñadú"));
  });

  it("takes a chosen piece with the selection when no target range is named", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    document.getSelection()?.setBaseAndExtent(words, 4, words, 9);
    expect(deleting("deleteContentBackward", words, 4, 9, false).taken).toBe(true);
    words.deleteData(4, 5);
    caretAt(words, 4);
    written("deleteContentBackward");
    expect(text).toBe(BODY.replace("ñandú", ""));
  });

  it("takes a word with Ctrl and Backspace", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    expect(deleting("deleteWordBackward", words, 0, 3).taken).toBe(true);
    words.deleteData(0, 3);
    written("deleteWordBackward");
    expect(text).toBe(BODY.replace("Uno", ""));
  });

  it("leaves a delete that would empty the span to the slow way", () => {
    opened();
    const words = wordsOf(paragraphs()[1]);
    caretAt(words, 7);
    expect(deleting("deleteContentBackward", words, 0, 7).taken).toBe(false);
  });

  it("leaves a delete that reaches into another node to the slow way", () => {
    opened();
    const [first, second] = paragraphs();
    const one = wordsOf(first);
    const two = wordsOf(second);
    const event = new InputEvent("beforeinput", {
      inputType: "deleteContentBackward",
      bubbles: true,
      cancelable: true,
    });
    const reaching = document.createRange();
    reaching.setStart(one, 5);
    reaching.setEnd(two, 0);
    Object.defineProperty(event, "getTargetRanges", { value: () => [reaching] });
    caretAt(two, 0);
    expect(page().dispatchEvent(event)).toBe(false);
  });

  it("leaves a collapsed delete with no target range to the slow way", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 6);
    expect(deleting("deleteContentBackward", words, 5, 6, false).taken).toBe(false);
  });
});

describe("the undo of keys the browser wrote", () => {
  const undone = (said: string | null) => {
    opened(PLACED, BODY, said);
    const words = wordsOf(paragraphs()[0]);
    for (const [at, letter] of ["a", "b"].entries()) {
      caretAt(words, 3 + at);
      if (asking("insertText", letter).taken) {
        words.insertData(3 + at, letter);
        caretAt(words, 4 + at);
        written("deleteContentBackward");
      }
    }
    key("z", { ctrlKey: true });
    return text;
  };

  it("takes quick letters back in one turn, as the slow way does", () => {
    expect(undone(null)).toBe(BODY);
    expect(undone("on")).toBe(BODY);
  });

  it("takes letters typed apart back one turn at a time", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "a");
    vi.setSystemTime(Date.now() + 5000);
    typedNatively(words, 4, "b");
    key("z", { ctrlKey: true });
    expect(text).toBe(BODY.replace("Uno ", "Unoa "));
    key("z", { ctrlKey: true });
    expect(text).toBe(BODY);
  });
});

describe("the input read back after a key the browser writes", () => {
  it("is only that of the key taken, so another kind of input changes nothing", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    written("insertCompositionText");
    expect(wrote).toEqual([]);
    expect(redraws).toEqual([]);
    expect(text).toBe(BODY);
    expect(typedNatively(words, 3, "y")).toBe(true);
    expect(text).toBe(BODY.replace("Uno ", "Unoy "));
  });

  it("is not read when the browser says it is composing", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    page().dispatchEvent(
      new InputEvent("input", { bubbles: true, inputType: "insertText", isComposing: true }),
    );
    expect(wrote).toEqual([]);
    expect(redraws).toEqual([]);
    expect(text).toBe(BODY);
    expect(typedNatively(words, 3, "y")).toBe(true);
    expect(text).toBe(BODY.replace("Uno ", "Unoy "));
  });

  it("is read again for the next real key after a stray one", () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 3);
    expect(asking("insertText", "x").taken).toBe(true);
    written("deleteContentBackward");
    expect(wrote).toEqual([]);
    written();
    expect(wrote).toEqual([]);
    expect(typedNatively(words, 3, "z")).toBe(true);
    expect(wrote).toEqual([BODY.replace("Uno ", "Unoz ")]);
  });
});

describe("the check the core holds, after a slow key that follows native letters", () => {
  const checkHeld = async (slow: () => void) => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "a");
    typedNatively(words, 4, "b");
    expect(asked).toHaveLength(0);
    slow();
    await rest(0);
    await vi.waitFor(() => expect(asked.length).toBeGreaterThanOrEqual(2));
    await rest(500);
    return asked;
  };

  it("is the one the core answered for the letters, when the key is *", async () => {
    const sent = await checkHeld(() => {
      expect(asking("insertText", "*").taken).toBe(false);
    });
    expect(sent).toHaveLength(2);
    expect(sent[1].was).toEqual(sent[0].now);
    expect(sent[1].was.bytes).toBe(BYTES.encode(sent[0].body).length);
  });

  it("is the one the core answered for the letters, when the key is Enter", async () => {
    const sent = await checkHeld(() => {
      caretAt(wordsOf(paragraphs()[0]), 5);
      expect(asking("insertParagraph").taken).toBe(false);
    });
    expect(sent).toHaveLength(2);
    expect(sent[1].was).toEqual(sent[0].now);
    expect(sent[1].was.bytes).toBe(BYTES.encode(sent[0].body).length);
  });
});

describe("the places written once native keys rest", () => {
  const rawPlaces = () =>
    paragraphs().map((one) => [
      one.getAttribute("data-at"),
      one.firstElementChild?.getAttribute("data-src"),
    ]);
  const drawnPlaces = (said: string) =>
    spansOf(blocksOf(said)).map(([from, to]) => [`${from}-${to}`, `${from}-${to}`]);

  it("are written for a later block when two native keys cancel out", async () => {
    opened();
    const [first, second] = paragraphs();
    const words = wordsOf(first);
    typedNatively(words, 3, "f");
    expect(placeOf(second)).toEqual([25, 32]);
    caretAt(words, 4);
    expect(deleting("deleteContentBackward", words, 3, 4).taken).toBe(true);
    words.deleteData(3, 1);
    caretAt(words, 3);
    written("deleteContentBackward");
    expect(text).toBe(BODY);
    await rest(250);
    expect(shownAt).toEqual([BODY]);
    expect(second.getAttribute("data-at")).toBe("24-31");
    expect(second.getAttribute("data-at")).toBe(placeOf(second)?.join("-"));
    expect(second.firstElementChild?.getAttribute("data-src")).toBe("24-31");
    expect(asked).toHaveLength(0);
  });

  it("are written for the later blocks after a slow key and native keys that do not cancel", async () => {
    opened();
    caretAt(wordsOf(paragraphs()[0]), 3);
    expect(asking("insertText", "*").taken).toBe(false);
    await rest(300);
    const words = wordsOf(paragraphs()[0]);
    expect(typedNatively(words, 1, "ñ")).toBe(true);
    expect(typedNatively(words, 2, "漢")).toBe(true);
    await rest(250);
    await rest(0);
    expect(shownAt[shownAt.length - 1]).toBe(text);
    expect(rawPlaces()).toEqual(drawnPlaces(text));
    for (const one of paragraphs()) expect(one.getAttribute("data-at")).toBe(placeSaid(one));
  });
});

describe("a composition opened before the core's answer", () => {
  it("keeps the answer from taking the block until it ends, and drops it then as stale", async () => {
    opened();
    let calls = 0;
    core = (sent) => {
      calls += 1;
      if (calls > 1) return Promise.resolve(blockwise(sent));
      const spans = spansOf(blocksOf(sent.body));
      const [from, to] = spans[0];
      return Promise.resolve(
        nearAnswer({
          spans,
          from: 0,
          to: 1,
          html:
            `<p data-at="${from}-${to}"><span data-src="${from}-${from + 2}">Un</span>` +
            `<strong><span data-src="${from + 2}-${to}">oab ñandú 漢字 fin</span></strong></p>`,
        }),
      );
    };
    const first = paragraphs()[0];
    const words = wordsOf(first);
    typedNatively(words, 3, "a");
    typedNatively(words, 4, "b");
    caretAt(words, 5);
    page().dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true }));
    await rest(250);
    await rest(500);
    expect(asked).toHaveLength(1);
    expect(page().querySelector("strong")).toBeNull();
    expect(words.isConnected).toBe(true);
    expect(redraws).toEqual([]);
    page().dispatchEvent(new CompositionEvent("compositionend", { bubbles: true, data: "é" }));
    await rest(0);
    await rest(500);
    expect(text).toContain("é");
    expect(page().querySelector("strong")).toBeNull();
    expect(redraws).toEqual([]);
  });
});

describe("the selection after a block the core drew differently is taken over", () => {
  it("is laid again at the same places of the source, not at the block's start", async () => {
    opened();
    core = (sent) => {
      const spans = spansOf(blocksOf(sent.body));
      const [from, to] = spans[0];
      return Promise.resolve(
        nearAnswer({
          spans,
          from: 0,
          to: 1,
          html:
            `<p data-at="${from}-${to}"><span data-src="${from}-${from + 2}">Un</span>` +
            `<strong><span data-src="${from + 2}-${from + 5}">oab</span></strong>` +
            `<span data-src="${from + 5}-${to}"> ñandú 漢字 fin</span></p>`,
        }),
      );
    };
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "a");
    typedNatively(words, 4, "b");
    key("ArrowLeft", { shiftKey: true });
    document.getSelection()?.setBaseAndExtent(words, 5, words, 3);
    expect(document.getSelection()?.toString()).toBe("ab");
    await rest(250);
    await rest(0);
    expect(asked).toHaveLength(1);
    expect(page().querySelector("strong")).not.toBeNull();
    expect(words.isConnected).toBe(false);
    const selection = document.getSelection();
    expect(selection?.toString()).toBe("ab");
    expect(page().contains(selection?.anchorNode ?? null)).toBe(true);
  });
});

describe("keys that cancel out, then a slow key", () => {
  it("sends the small edit against the check the core holds", async () => {
    opened();
    const words = wordsOf(paragraphs()[0]);
    typedNatively(words, 3, "x");
    caretAt(words, 4);
    expect(deleting("deleteContentBackward", words, 3, 4).taken).toBe(true);
    words.deleteData(3, 1);
    caretAt(words, 3);
    written("deleteContentBackward");
    expect(text).toBe(BODY);
    await rest(250);
    expect(asked).toHaveLength(0);
    caretAt(words, 3);
    expect(asking("insertText", "*").taken).toBe(false);
    await rest(0);
    await vi.waitFor(() => expect(asked).toHaveLength(1));
    const [sent] = asked;
    expect(sent.body).toBe(text);
    expect(sent.was).toEqual(checkOf(BODY));
    expect(sent.edit).toEqual({ from: 3, to: 3, insert: "*" });
  });
});

describe("matching what the core drew against what the browser wrote", () => {
  const sheetOf = (html: string) => {
    document.body.innerHTML = `<article class="page">${html}</article>`;
    return page();
  };

  const nearOf = (html: string, spans: Span[]) =>
    nearAnswer({ spans, from: 0, to: spans.length, html });

  it("keeps the nodes of a block alike but for dressing, and takes the core's places", () => {
    const sheet = sheetOf(
      '<p data-at="0-9"><a data-src="0-9" href="https://old" style="color:red" class="dressed">link</a></p>',
    );
    const [block] = paragraphs();
    const link = block.firstElementChild;
    const taken = matched(
      sheet,
      nearOf('<p data-at="2-11"><a data-src="2-11" href="https://new">link</a></p>', [[2, 11]]),
    );
    expect(taken).toEqual([]);
    expect(paragraphs()[0]).toBe(block);
    expect(block.firstElementChild).toBe(link);
    expect(block.getAttribute("data-at")).toBe("2-11");
    expect(link?.getAttribute("data-src")).toBe("2-11");
    expect(link?.getAttribute("href")).toBe("https://old");
  });

  it("keeps a picture whose source was rewritten", () => {
    const sheet = sheetOf('<p data-at="0-8"><img data-src="0-8" src="asset://a.png"></p>');
    const img = sheet.querySelector("img");
    const taken = matched(
      sheet,
      nearOf('<p data-at="1-9"><img data-src="1-9" src="a.png"></p>', [[1, 9]]),
    );
    expect(taken).toEqual([]);
    expect(sheet.querySelector("img")).toBe(img);
    expect(img?.getAttribute("src")).toBe("asset://a.png");
    expect(img?.getAttribute("data-src")).toBe("1-9");
  });

  it("copies a heading's new id when its words changed", () => {
    const sheet = sheetOf('<h2 data-at="0-8" id="viejo"><span data-src="3-8">Nuevo</span></h2>');
    const heading = sheet.querySelector("h2");
    const taken = matched(
      sheet,
      nearOf('<h2 data-at="0-8" id="nuevo"><span data-src="3-8">Nuevo</span></h2>', [[0, 8]]),
    );
    expect(taken).toEqual([]);
    expect(sheet.querySelector("h2")).toBe(heading);
    expect(heading?.id).toBe("nuevo");
  });

  it("takes a last span typed with a trailing space for alike", () => {
    const sheet = sheetOf('<p data-at="0-5"><span data-src="0-5">Hola </span></p>');
    const block = paragraphs()[0];
    const taken = matched(
      sheet,
      nearOf('<p data-at="0-4"><span data-src="0-4">Hola</span></p>', [[0, 4]]),
    );
    expect(taken).toEqual([]);
    expect(paragraphs()[0]).toBe(block);
    expect(block.getAttribute("data-at")).toBe("0-4");
  });

  it("takes a block over when the core draws a real change in it", () => {
    const sheet = sheetOf('<p data-at="0-8"><span data-src="0-8">Hola mundo</span></p>');
    const block = paragraphs()[0];
    const taken = matched(
      sheet,
      nearOf(
        '<p data-at="0-12"><span data-src="0-5">Hola </span><strong><span data-src="7-12">mundo</span></strong></p>',
        [[0, 12]],
      ),
    );
    expect(taken).toEqual([block]);
    expect(block.querySelector("strong")).not.toBeNull();
  });

  it("keeps a fold open that is in a block taken over", () => {
    const sheet = sheetOf(
      '<details data-at="0-30" open><summary>Tit</summary><p data-at="10-20"><span data-src="10-20">Hola mundo</span></p></details>',
    );
    const fold = sheet.querySelector("details") as HTMLDetailsElement;
    expect(fold.open).toBe(true);
    const taken = matched(
      sheet,
      nearOf(
        '<details data-at="0-34"><summary>Tit</summary><p data-at="10-24"><span data-src="10-14">Hola</span><strong><span data-src="16-24">mundo</span></strong></p></details>',
        [[0, 34]],
      ),
    );
    expect(taken).toHaveLength(1);
    const now = sheet.querySelector("details") as HTMLDetailsElement;
    expect(now.querySelector("strong")).not.toBeNull();
    expect(now.open).toBe(true);
  });
});

describe("the edit read from the key itself, not from a diff of the words", () => {
  const nbsp = " ";

  it("keeps a no-break space and the letter after it on their sides of the blank", () => {
    opened(
      '<p data-at="0-15"><span data-src="0-15">Segundo parrafo</span></p>',
      "Segundo parrafo\n",
    );
    const words = wordsOf(paragraphs()[0]);
    expect(typedNatively(words, 7, nbsp)).toBe(true);
    expect(text).toBe(`Segundo${nbsp} parrafo\n`);
    expect(words.data).toBe(`Segundo${nbsp} parrafo`);
    expect(typedNatively(words, 8, "x")).toBe(true);
    expect(text).toBe(`Segundo${nbsp}x parrafo\n`);
  });

  it("writes a plain space at the place typed when the browser shows it as a no-break one", () => {
    opened(
      '<p data-at="0-15"><span data-src="0-15">Segundo parrafo</span></p>',
      "Segundo parrafo\n",
    );
    const words = wordsOf(paragraphs()[0]);
    caretAt(words, 7);
    expect(asking("insertText", " ").taken).toBe(true);
    words.insertData(7, nbsp);
    caretAt(words, 8);
    written();
    expect(text).toBe("Segundo  parrafo\n");
    expect(typedNatively(words, 8, "x")).toBe(true);
    expect(text).toBe("Segundo x parrafo\n");
  });

  it("lands a letter typed between two spaces at the caret", () => {
    opened('<p data-at="0-4"><span data-src="0-4">a  b</span></p>', "a  b\n");
    const words = wordsOf(paragraphs()[0]);
    expect(typedNatively(words, 2, "a")).toBe(true);
    expect(text).toBe("a a b\n");
  });
});
