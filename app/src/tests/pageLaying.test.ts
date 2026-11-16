import { afterEach, describe, expect, it, vi } from "vitest";
import { checkAfter, checkOf, type Sent, type Typed } from "../placing";
import { changeOf, inWholeChars } from "../ui/pageEdits";
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
