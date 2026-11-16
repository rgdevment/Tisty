import { afterEach, describe, expect, it, vi } from "vitest";
import { checkAfter, checkOf, type PlacedNear, type Sent, summed, type Typed } from "../placing";
import { changeOf, inWholeChars } from "../ui/pageEdits";
import { placeOf } from "../ui/pagePlaces";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BYTES = new TextEncoder();
const BODY = "# Hola\n\nUno 😀 dos ñandú\r\n\n- primero\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-28"><span data-src="8-28">Uno 😀 dos ñandú</span></p>' +
  '<ul data-at="31-41"><li data-at="31-40"><span data-src="33-40">primero</span></li></ul>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const sentOn = async (act: (writing: PageWriting) => void) => {
  const sent: Sent[] = [];
  let text = BODY;
  document.body.innerHTML = `<article class="page">${PLACED}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: () => {},
    near: (one) => {
      sent.push(one);
      return Promise.reject(new Error("no core here"));
    },
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  act(writing);
  await vi.waitFor(() => expect(sent).toHaveLength(1));
  return { sent: sent[0], now: text };
};

const appliedInBytes = (was: string, edit: Typed) => {
  const bytes = BYTES.encode(was);
  return [...bytes.slice(0, edit.from), ...BYTES.encode(edit.insert), ...bytes.slice(edit.to)];
};

const caretAfter = (said: string) => {
  const words = document.querySelector('[data-src="8-28"]')?.firstChild as Text;
  document.getSelection()?.collapse(words, (words.textContent ?? "").indexOf(said) + said.length);
};

describe("a key sent to the core alone", () => {
  it("names the text it was typed on and lands, in its bytes, on the text it made", async () => {
    const { sent, now } = await sentOn(() => {
      caretAfter("ñandú");
      document.querySelector(".page")?.dispatchEvent(
        new InputEvent("beforeinput", {
          inputType: "insertText",
          data: "ñ",
          bubbles: true,
          cancelable: true,
        }),
      );
    });
    expect(now).toBe(BODY.replace("ñandú", "ñandúñ"));
    expect(sent.body).toBe(now);
    expect(sent.was).toEqual(checkOf(BODY));
    expect(sent.now).toEqual(checkAfter(checkOf(BODY), sent.edit));
    expect(sent.now.bytes).toBe(BYTES.encode(now).length);
    expect(appliedInBytes(BODY, sent.edit)).toEqual([...BYTES.encode(now)]);
  });

  it("takes in a whole character when the change cuts one in two", async () => {
    const next = BODY.replace("😀", "😁");
    const { sent } = await sentOn((writing) => writing.rewrite(next));
    expect(appliedInBytes(BODY, sent.edit)).toEqual([...BYTES.encode(next)]);
    expect(sent.edit.insert).toBe("😁");
    expect(sent.now).toEqual(checkAfter(checkOf(BODY), sent.edit));
    expect(sent.now.bytes).toBe(BYTES.encode(next).length);
  });
});

describe("an edit made whole for the core", () => {
  it("is left as it is when no character is cut", () => {
    const one = changeOf("Uno dos", "Uno tres");
    expect(inWholeChars("Uno dos", one)).toBe(one);
  });

  it("widens to the character on either side, at the edges of the text too", () => {
    for (const [was, now] of [
      ["😀", "😁"],
      ["a😀b", "a😁b"],
      ["😀b", "😀😀b"],
      ["x😀", "x"],
    ]) {
      const one = inWholeChars(was, changeOf(was, now));
      expect(`${was.slice(0, one.from)}${one.insert}${was.slice(one.to)}`).toBe(now);
      expect(new TextDecoder().decode(BYTES.encode(one.insert))).toBe(one.insert);
    }
  });
});

describe("a key laid while the next is already written", () => {
  it("still opens the blocks it laid, as a table no caret goes into", async () => {
    let text = BODY;
    const pending: { answer?: (near: PlacedNear) => void } = {};
    let asked = 0;
    document.body.innerHTML = `<article class="page">${PLACED}</article>`;
    writing = writtenOn(document, {
      text: () => text,
      write: (next) => {
        text = next;
      },
      shown: () => {},
      redraw: () => {},
      near: () => {
        asked += 1;
        if (asked > 1) return Promise.reject(new Error("only the first key is answered"));
        return new Promise((resolve) => {
          pending.answer = resolve;
        });
      },
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    const typed = () =>
      document.querySelector(".page")?.dispatchEvent(
        new InputEvent("beforeinput", {
          inputType: "insertText",
          data: "ñ",
          bubbles: true,
          cancelable: true,
        }),
      );
    caretAfter("ñandú");
    typed();
    await vi.waitFor(() => expect(pending.answer).toBeDefined());
    typed();
    pending.answer?.({
      count: 3,
      sum: summed([0, 6, 33, 43]),
      from: 1,
      to: 2,
      html: '<div class="table" data-at="8-30"><table><tr><td>x</td></tr></table></div>',
      heads: null,
      whole: false,
    });
    await vi.waitFor(() =>
      expect(document.querySelector(".table")?.getAttribute("contenteditable")).toBe("false"),
    );
  });
});

describe("the places after a key, written once the keys rest", () => {
  it("leaves the blocks after a key as they say until the keys rest, and reads them moved", async () => {
    let text = BODY;
    const shown: string[] = [];
    document.body.innerHTML = `<article class="page">${PLACED}</article>`;
    const list = document.querySelector("ul") as Element;
    writing = writtenOn(document, {
      text: () => text,
      write: (next) => {
        text = next;
      },
      shown: (now) => shown.push(now),
      redraw: () => {},
      near: () =>
        Promise.resolve({
          count: 3,
          sum: summed([0, 6, 33, 43]),
          from: 1,
          to: 2,
          html: '<p data-at="8-30"><span data-src="8-30">Uno 😀 dos ñandúñ</span></p>',
          heads: null,
          whole: false,
        }),
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    caretAfter("ñandú");
    document.querySelector(".page")?.dispatchEvent(
      new InputEvent("beforeinput", {
        inputType: "insertText",
        data: "ñ",
        bubbles: true,
        cancelable: true,
      }),
    );

    await vi.waitFor(() => expect(document.querySelector("p")?.textContent).toContain("ñandúñ"));
    expect(list.getAttribute("data-at")).toBe("31-41");
    expect(placeOf(list)).toEqual([33, 43]);
    expect(placeOf(list.querySelector("[data-src]"), "data-src")).toEqual([35, 42]);

    await vi.waitFor(() => expect(shown).toEqual([text]));
    expect(list.getAttribute("data-at")).toBe("33-43");
    expect(list.querySelector("li")?.getAttribute("data-at")).toBe("33-42");
  });
});

describe("the blocks the core draws beside a laid key", () => {
  it("stay on the page as they were, and only the block the key reached is dressed", async () => {
    let text = BODY;
    const dressed: Element[][] = [];
    document.body.innerHTML = `<article class="page">${PLACED}</article>`;
    const [heading, words, list] = Array.from(document.querySelectorAll(".page > *"));
    writing = writtenOn(document, {
      text: () => text,
      write: (next) => {
        text = next;
      },
      shown: () => {},
      redraw: () => {},
      near: () =>
        Promise.resolve({
          count: 3,
          sum: summed([]),
          from: 0,
          to: 3,
          html:
            '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
            '<p data-at="8-30"><span data-src="8-30">Uno 😀 dos ñandúñ</span></p>' +
            '<ul data-at="33-43"><li data-at="33-42"><span data-src="35-42">primero</span></li></ul>',
          heads: null,
          whole: false,
        }),
      dressed: (_, laid) => dressed.push([...laid]),
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    caretAfter("ñandú");
    document.querySelector(".page")?.dispatchEvent(
      new InputEvent("beforeinput", {
        inputType: "insertText",
        data: "ñ",
        bubbles: true,
        cancelable: true,
      }),
    );

    await vi.waitFor(() => expect(dressed).toHaveLength(1));
    expect(dressed[0]).toEqual([words]);
    expect(Array.from(document.querySelectorAll(".page > *"))).toEqual([heading, words, list]);
    expect(words?.textContent).toContain("ñandúñ");
    expect(list?.getAttribute("data-at")).toBe("31-41");
    expect(placeOf(list)).toEqual([33, 43]);
  });
});
