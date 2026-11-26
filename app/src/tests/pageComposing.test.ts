import { afterEach, describe, expect, it, vi } from "vitest";
import { type PlacedNear, summed } from "../placing";
import { type Hands, type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nQu tal\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-14"><span data-src="8-14">Qu tal</span></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (near: Hands["near"] = () => Promise.reject(new Error("no core here"))) => {
  let text = BODY;
  document.body.innerHTML = `<article class="page">${PLACED}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: () => {},
    near,
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  return () => text;
};

const sheet = () => document.querySelector(".page") as HTMLElement;
const words = () => document.querySelector('[data-src="8-14"]')?.firstChild as Text;

const caretAfter = (said: string) =>
  document.getSelection()?.collapse(words(), words().data.indexOf(said) + said.length);

const asked = (inputType: string, data: string | null, isComposing = false) =>
  sheet().dispatchEvent(
    new InputEvent("beforeinput", {
      inputType,
      data,
      isComposing,
      bubbles: true,
      cancelable: true,
    }),
  );

const composing = (kind: string, data: string) =>
  sheet().dispatchEvent(new CompositionEvent(kind, { data, bubbles: true }));

const deadKey = (mark: string, letter: string) => {
  const held = document.getSelection();
  composing("compositionstart", "");
  asked("insertCompositionText", mark, true);
  const node = held?.anchorNode as Text;
  const at = held?.anchorOffset ?? 0;
  node.insertData(at, mark);
  held?.collapse(node, at + mark.length);
  asked("deleteCompositionText", null, true);
  node.deleteData(at, mark.length);
  asked("insertFromComposition", letter, true);
  node.insertData(at, letter);
  held?.collapse(node, at + letter.length);
  composing("compositionend", letter);
};

describe("a letter written with a dead key", () => {
  it("lands where the caret is", () => {
    const text = opened();
    caretAfter("Qu");
    deadKey("´", "é");
    expect(text()).toBe("# Hola\n\nQué tal\n");
  });

  it("lands after keys the page does not show yet", () => {
    const text = opened();
    caretAfter("Qu");
    asked("insertText", "e");
    asked("insertText", "s");
    deadKey("´", "é");
    expect(text()).toBe("# Hola\n\nQuesé tal\n");
  });

  it("opens a fence written with dead backquotes as typed backquotes would", () => {
    const text = opened();
    caretAfter("tal");
    asked("insertParagraph", null);
    deadKey("`", "`");
    deadKey("`", "`");
    deadKey("`", "`");
    expect(text()).toContain("```\n");
    expect(text().match(/```/g)?.length).toBe(2);
  });

  it("keeps its mark until the composition ends, even when a drawing comes back meanwhile", async () => {
    let answer: (one: PlacedNear | null) => void = () => {};
    const near = vi.fn(
      () =>
        new Promise<PlacedNear>((done, failed) => {
          answer = (one) => (one ? done(one) : failed(new Error("no core here")));
        }),
    );
    const text = opened(near as unknown as Hands["near"]);
    caretAfter("Qu");
    asked("insertText", "e");
    await vi.waitFor(() => expect(near).toHaveBeenCalledTimes(1));
    composing("compositionstart", "");
    asked("insertCompositionText", "´", true);
    const node = words();
    const at = document.getSelection()?.anchorOffset ?? 0;
    node.insertData(at, "´");
    const marked = node;
    answer({
      count: 2,
      sum: summed([0, 6]),
      from: 1,
      to: 2,
      html: '<p data-at="8-15"><span data-src="8-15">Que tal</span></p>',
      heads: null,
      whole: false,
    });
    await vi.advanceTimersByTimeAsync(50);
    expect(marked.isConnected).toBe(true);
    expect(marked.data).toContain("´");
    composing("compositionend", "é");
    expect(text()).toContain("Queé");
    await vi.waitFor(() => expect(marked.isConnected && marked.data.includes("´")).toBe(false));
  });
});

describe("a composition that ends away from the page", () => {
  it("never holds the drawings of the keys after it", async () => {
    let answer: (one: PlacedNear) => void = () => {};
    const near = vi.fn(
      () =>
        new Promise<PlacedNear>((done) => {
          answer = done;
        }),
    );
    opened(near as unknown as Hands["near"]);
    caretAfter("Qu");
    composing("compositionstart", "");
    document.body.dispatchEvent(
      new CompositionEvent("compositionend", { data: "", bubbles: true }),
    );
    asked("insertText", "e");
    await vi.waitFor(() => expect(near).toHaveBeenCalledTimes(1));
    const before = words();
    answer({
      count: 2,
      sum: summed([0, 6]),
      from: 1,
      to: 2,
      html: '<p data-at="8-15"><span data-src="8-15">Que tal</span></p>',
      heads: null,
      whole: false,
    });
    await vi.advanceTimersByTimeAsync(50);
    expect(before.isConnected ? before.data : null).toBe("Que tal");
  });
});

describe("a composition whose end never comes", () => {
  it("holds the drawing of the key after it once, never every key after", async () => {
    const answers: ((one: PlacedNear) => void)[] = [];
    const near = vi.fn(
      () =>
        new Promise<PlacedNear>((done) => {
          answers.push(done);
        }),
    );
    opened(near as unknown as Hands["near"]);
    caretAfter("Qu");
    composing("compositionstart", "");
    const drawn = (words: string, end: number): PlacedNear => ({
      count: 2,
      sum: summed([0, 6]),
      from: 1,
      to: 2,
      html: `<p data-at="8-${end}"><span data-src="8-${end}">${words}</span></p>`,
      heads: null,
      whole: false,
    });
    const line = () => document.querySelector(".page > p span")?.firstChild as Text;
    asked("insertText", "e");
    await vi.waitFor(() => expect(near).toHaveBeenCalledTimes(1));
    answers[0](drawn("Que tal", 15));
    await vi.advanceTimersByTimeAsync(10_000);
    await vi.waitFor(() => expect(line().data).toBe("Que tal"));
    document.getSelection()?.collapse(line(), 3);
    asked("insertText", "s");
    await vi.waitFor(() => expect(near).toHaveBeenCalledTimes(2));
    answers[1](drawn("Ques tal", 16));
    await vi.advanceTimersByTimeAsync(50);
    expect(line().data).toBe("Ques tal");
  });
});

describe("a composition given up before it writes anything", () => {
  it("takes away the words it was begun over, as the page no longer shows them", () => {
    const text = opened();
    const range = document.createRange();
    range.setStart(words(), words().data.indexOf("tal"));
    range.setEnd(words(), words().data.length);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    composing("compositionstart", "");
    composing("compositionend", "");
    expect(text()).toBe("# Hola\n\nQu \n");
  });

  it("leaves the text alone when it changed while the composition was open", () => {
    const text = opened();
    const range = document.createRange();
    range.setStart(words(), words().data.indexOf("tal"));
    range.setEnd(words(), words().data.length);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    composing("compositionstart", "");
    asked("insertText", "Z");
    composing("compositionend", "");
    expect(text()).toBe("# Hola\n\nQu Z\n");
  });

  it("changes nothing when it was begun at a caret", () => {
    const text = opened();
    caretAfter("Qu");
    composing("compositionstart", "");
    composing("compositionend", "");
    expect(text()).toBe("# Hola\n\nQu tal\n");
  });
});
