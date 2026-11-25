import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const OLD = "# Título\n\nTexto\n";
const OLD_PLACED =
  '<h1 data-at="0-9"><span data-src="2-9">Título</span></h1><p data-at="11-17"><span data-src="11-16">Texto</span></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (
  body: string,
  placed: string,
  near: () => Promise<never> = () => Promise.reject(new Error("no core here")),
) => {
  const here = { text: body };
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  writing = writtenOn(document, {
    text: () => here.text,
    write: (next) => {
      here.text = next;
    },
    shown: () => {},
    redraw: () => {},
    near,
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  return { here, writing };
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

const wordsOf = (selector: string) => document.querySelector(selector)?.firstChild as Text;

describe("the typing harness", () => {
  it("writes a key at the caret when nothing changed from outside", () => {
    const { here } = opened(OLD, OLD_PLACED);
    document.querySelector(".page")?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    document.getSelection()?.collapse(wordsOf('[data-src="11-16"]'), 0);
    typed("x");
    expect(here.text).toContain("\n\nxTexto");
  });
});

describe("keys typed after the words changed from outside", () => {
  it.fails("land at the caret's place in the new words, not inside the heading above it", () => {
    const { here } = opened(OLD, OLD_PLACED);
    document.getSelection()?.collapse(wordsOf('[data-src="11-16"]'), 0);
    const next = `Añadí ñandú\n\n${OLD}`;
    here.text = next;
    writing?.outside(OLD, next);
    typed("x");
    expect(here.text).toContain("\n\nxTexto");
    expect(here.text).toContain("\n\n# Título\n");
  });
});

describe("keys typed while the core has not answered", () => {
  const HOLA = "Hola\n\nUno\n";
  const HOLA_PLACED =
    '<p data-at="0-5"><span data-src="0-4">Hola</span></p><p data-at="6-10"><span data-src="6-9">Uno</span></p>';

  it.fails("land where the caret was put by a click, not where the last key left it", () => {
    const { here } = opened(HOLA, HOLA_PLACED, () => new Promise(() => {}));
    document.getSelection()?.collapse(wordsOf('[data-src="0-4"]'), 4);
    typed("a");
    typed("b");
    typed("c");
    document.querySelector(".page")?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    document.getSelection()?.collapse(wordsOf('[data-src="6-9"]'), 0);
    typed("X");
    expect(here.text).toBe("Holaabc\n\nXUno\n");
  });
});

describe("a page written on while every key asks for a whole drawing", () => {
  it.fails("asks for no more than one whole drawing at a time while keys keep coming", async () => {
    let requests = 0;
    const text = { value: "Hola\n" };
    document.body.innerHTML =
      '<article class="page"><p data-at="0-5"><span data-src="0-4">Hola</span></p></article>';
    writing = writtenOn(document, {
      text: () => text.value,
      write: (next) => {
        text.value = next;
      },
      shown: () => {},
      redraw: () => {
        requests += 1;
      },
      near: () => Promise.resolve(null as never),
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    document.getSelection()?.collapse(wordsOf('[data-src="0-4"]'), 4);
    for (let key = 0; key < 8; key += 1) {
      typed("a");
      await new Promise((done) => setTimeout(done, 30));
    }
    expect(requests).toBeLessThanOrEqual(1);
  });
});
