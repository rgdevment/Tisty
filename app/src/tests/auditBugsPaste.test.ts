import { afterEach, describe, expect, it } from "vitest";
import { arrival, TISTY_MARKDOWN } from "../ui/clipboarding";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# T\n\nHola mundo bonito\n";
const PLACED =
  '<h1 id="t" data-at="0-4"><span data-src="2-3">T</span></h1><p data-at="5-22"><span data-src="5-22">Hola mundo bonito</span></p>';
const BOLD = "Hola **mundo**\n";
const BOLD_PLACED =
  '<p data-at="0-14"><span data-src="0-5">Hola </span><strong><span data-src="7-12">mundo</span></strong></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const settled = () => new Promise((done) => setTimeout(done, 0));

const onPage = (body = BODY, placed = PLACED) => {
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
  const words = Array.from(document.querySelectorAll("[data-src]")).pop()?.firstChild as Text;
  document.getSelection()?.collapse(words, words.length);
  return () => text;
};

const lineBroken = () =>
  document.querySelector(".page")?.dispatchEvent(
    new KeyboardEvent("keydown", {
      key: "Enter",
      shiftKey: true,
      bubbles: true,
      cancelable: true,
    }),
  );

const wordsPasted = () => {
  const event = new InputEvent("beforeinput", {
    inputType: "insertFromPaste",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "getTargetRanges", { value: () => [] });
  Object.defineProperty(event, "dataTransfer", {
    value: {
      getData: (kind: string) =>
        kind === "text/html" ? "<b>y</b>" : kind === "text/plain" ? "**y**" : "",
    },
  });
  document.querySelector(".page")?.dispatchEvent(event);
};

describe("the paste harness", () => {
  it("writes the words pasted at the end of the paragraph", async () => {
    const text = onPage();
    wordsPasted();
    await settled();
    expect(text()).toContain("**y**");
  });
});

describe("words pasted on a line just begun", () => {
  it("are written after a line break and not dropped", async () => {
    const text = onPage();
    lineBroken();
    wordsPasted();
    await settled();
    expect(text()).toContain("**y**");
  });
});

const copied = () => {
  const set: Record<string, string> = {};
  const event = new Event("copy", { bubbles: true, cancelable: true });
  Object.defineProperty(event, "clipboardData", {
    value: {
      setData: (kind: string, value: string) => {
        set[kind] = value;
      },
    },
  });
  document.querySelector(".page")?.dispatchEvent(event);
  return set;
};

describe("words copied from the page", () => {
  it("carry their formatting as text/html for other apps", () => {
    onPage(BOLD, BOLD_PLACED);
    const first = document.querySelector('[data-src="0-5"]')?.firstChild as Text;
    const last = document.querySelector('[data-src="7-12"]')?.firstChild as Text;
    const range = document.createRange();
    range.setStart(first, 0);
    range.setEnd(last, last.length);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    const set = copied();
    expect(Object.keys(set)).toContain("text/html");
    expect(set["text/html"]).toMatch(/<(strong|b)>mundo<\/(strong|b)>/);
  });
});

describe("words copied from one page and pasted on another", () => {
  it("come back as the same Markdown, the Tisty type outranking the other faces", () => {
    onPage(BOLD, BOLD_PLACED);
    const first = document.querySelector('[data-src="0-5"]')?.firstChild as Text;
    const last = document.querySelector('[data-src="7-12"]')?.firstChild as Text;
    const range = document.createRange();
    range.setStart(first, 0);
    range.setEnd(last, last.length);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    const set = copied();
    expect(set[TISTY_MARKDOWN]).toBe("Hola **mundo**");
    const clipboard = { getData: (kind: string) => set[kind] ?? "" };
    expect(arrival(clipboard)).toEqual({ kind: "own", text: "Hola **mundo**" });
  });
});
