import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { holdingOn } from "../ui/pageHolding";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BYTES = new TextEncoder();
const LINE = 40;

let writing: PageWriting | null = null;
let text = "";
let redrawn: string | null = null;
const tall = new Map<number, number>();
let settling: (() => void) | null = null;

const drawnOf = (words: string) => {
  let at = 0;
  let html = "";
  for (const one of words.split("\n\n")) {
    const end = at + BYTES.encode(one).length;
    const said = one.replace(/\n$/, "");
    html += one.startsWith("<!--")
      ? `<div class="hint" data-at="${at}-${end}" contenteditable="false"></div>`
      : `<p data-at="${at}-${end}"><span data-src="${at}-${at + BYTES.encode(said).length}">${said}</span></p>`;
    at = end + 2;
  }
  return html;
};

const startOf = (one: Element) => Number(one.getAttribute("data-at")?.split("-")[0]);
const heightOf = (one: Element) => (one.matches(".hint") ? 0 : (tall.get(startOf(one)) ?? LINE));
const scrolled = () => document.documentElement.scrollTop;

const topOn = (said: string) => {
  const block = Array.from(document.querySelectorAll(".page > p")).find(
    (one) => one.textContent === said,
  );
  return block?.getBoundingClientRect().top;
};

const pageOf = (words: string, scrollTop: number) => {
  text = words;
  document.body.innerHTML = `<article class="page">${drawnOf(words)}</article>`;
  document.documentElement.scrollTop = scrollTop;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: (now) => {
      redrawn = now;
    },
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
};

const drawnAnew = async (next: string, heights: [number, number][]) => {
  writing?.rewrite(next);
  await vi.waitFor(() => expect(redrawn).toBe(next));
  for (const [start, height] of heights) tall.set(start, height);
  const sheet = document.querySelector(".page") as HTMLElement;
  sheet.innerHTML = drawnOf(next);
  writing?.drawn();
};

beforeEach(() => {
  Object.defineProperty(document, "scrollingElement", {
    value: document.documentElement,
    configurable: true,
  });
  Object.defineProperty(document.documentElement, "scrollHeight", {
    get: () => 5000,
    configurable: true,
  });
  Object.defineProperty(document.documentElement, "scrollTop", {
    value: 0,
    writable: true,
    configurable: true,
  });
  vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function (this: Element) {
    const blocks = Array.from(document.querySelectorAll(".page > [data-at]"));
    if (!blocks.includes(this) || !heightOf(this)) return new DOMRect(0, 0, 0, 0);
    let top = 0;
    for (const one of blocks.slice(0, blocks.indexOf(this))) top += heightOf(one);
    return new DOMRect(0, top - scrolled(), 600, heightOf(this));
  });
  window.ResizeObserver = class {
    constructor(seen: ResizeObserverCallback) {
      settling = () => seen([], this);
    }
    observe() {}
    unobserve() {}
    disconnect() {
      settling = null;
    }
  };
});

afterEach(() => {
  writing?.stop();
  writing = null;
  redrawn = null;
  settling = null;
  tall.clear();
  document.documentElement.classList.remove("tall");
  vi.restoreAllMocks();
  document.body.innerHTML = "";
});

const PAGE = "Arriba\n\nUn diagrama\n\nUno dos tres\n\nFin\n";
const DIAGRAM = PAGE.indexOf("Un diagrama");

describe("a page drawn whole again while it is written", () => {
  it("puts the block in sight back where it stood, though a diagram above it is drawn shorter", async () => {
    tall.set(DIAGRAM, 400);
    pageOf(PAGE, LINE + 400);
    expect(topOn("Uno dos tres")).toBe(0);
    await drawnAnew(PAGE.replace("dos ", ""), [[DIAGRAM, 100]]);
    expect(topOn("Uno tres")).toBe(0);
  });

  it("keeps it there as the diagram takes its height again, until a key is pressed", async () => {
    tall.set(DIAGRAM, 400);
    pageOf(PAGE, LINE + 400);
    await drawnAnew(PAGE.replace("dos ", ""), [[DIAGRAM, 100]]);
    expect(topOn("Uno tres")).toBe(0);
    expect(settling).not.toBeNull();
    tall.set(DIAGRAM, 400);
    settling?.();
    expect(topOn("Uno tres")).toBe(0);
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));
    expect(settling).toBeNull();
  });

  it("finds the block in sight again when words above it, out of sight, changed", async () => {
    tall.set(DIAGRAM, 400);
    pageOf(PAGE, LINE + 400);
    await drawnAnew(PAGE.replace("Arriba", "Arriba del todo"), [[DIAGRAM + 9, 100]]);
    expect(topOn("Uno dos tres")).toBe(0);
  });

  it("holds the block in sight, not one after a hint the page hides", async () => {
    const page = "Un diagrama\n\nUno dos tres\n\n<!-- Una pista -->\n\nOtro\n\nFin\n";
    const first = page.indexOf("Uno");
    tall.set(0, 400);
    tall.set(first, 2 * LINE);
    pageOf(page, 400);
    await drawnAnew(page.replace("dos ", ""), [
      [0, 100],
      [first, LINE],
    ]);
    expect(topOn("Uno tres")).toBe(0);
  });

  it("carries the block in sight through keys written before the drawing arrives", async () => {
    tall.set(DIAGRAM, 400);
    pageOf(PAGE, LINE + 400);
    const longer = PAGE.replace("Arriba", "Arriba del todo");
    writing?.rewrite(longer);
    await vi.waitFor(() => expect(redrawn).toBe(longer));
    await drawnAnew(longer.replace("dos ", ""), [[DIAGRAM + 9, 100]]);
    expect(topOn("Uno tres")).toBe(0);
  });

  it("keeps the block in sight when the words change from beside the page", () => {
    tall.set(DIAGRAM, 400);
    pageOf(PAGE, LINE + 400);
    const next = PAGE.replace("Arriba", "Arriba del todo");
    writing?.heldAcross(PAGE, next);
    tall.set(DIAGRAM + 9, 100);
    (document.querySelector(".page") as HTMLElement).innerHTML = drawnOf(next);
    writing?.drawn();
    expect(topOn("Uno dos tres")).toBe(0);
  });

  it("never scrolls a task's page inside its own frame, which the window around it scrolls", async () => {
    document.documentElement.classList.add("tall");
    tall.set(DIAGRAM, 400);
    pageOf(PAGE, LINE + 400);
    await drawnAnew(PAGE.replace("dos ", ""), [[DIAGRAM, 100]]);
    expect(scrolled()).toBe(LINE + 400);
  });
});

describe("the view held across a page drawn anew", () => {
  it("keeps the block where it stood, not where a caret laid on guessed heights moved the view", () => {
    tall.set(DIAGRAM, 400);
    document.body.innerHTML = `<article class="page">${drawnOf(PAGE)}</article>`;
    document.documentElement.scrollTop = LINE + 400;
    const holding = holdingOn(document);
    holding.before({ from: PAGE.indexOf("dos"), removed: 4, by: -4 });
    tall.set(DIAGRAM, 100);
    (document.querySelector(".page") as HTMLElement).innerHTML = drawnOf(PAGE.replace("dos ", ""));
    const caretInSight = vi.fn();
    holding.landed(() => {
      document.documentElement.scrollTop += 151;
    }, caretInSight);
    tall.set(DIAGRAM, 400);
    settling?.();
    expect(topOn("Uno tres")).toBe(0);
    expect(caretInSight).toHaveBeenCalled();
    holding.stop();
  });
});
