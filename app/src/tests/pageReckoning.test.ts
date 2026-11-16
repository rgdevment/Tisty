import { afterEach, describe, expect, it, vi } from "vitest";
import { type Check, checkOf, type Sent } from "../placing";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const patching = vi.hoisted(() => ({ refused: 0 }));

vi.mock("../ui/pagePatching", async (original) => ({
  ...(await original<typeof import("../ui/pagePatching")>()),
  patched: () => (patching.refused-- > 0 ? null : []),
}));

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
  document.documentElement.classList.remove("untitled");
});

const same = (one: Check, other: Check) =>
  one.bytes === other.bytes && one.hash[0] === other.hash[0] && one.hash[1] === other.hash[1];

const typed = (data: string, inputType = "insertText") =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

// The core keeps the text and its check as the app does: a key it can place, or the whole body.
const coreOf = (body: string) => {
  const core = { bytes: BYTES.encode(body), check: checkOf(body), told: [] as string[], drawn: 0 };
  const near = (sent: Sent) => {
    if (same(core.check, sent.was)) {
      const { from, to, insert } = sent.edit;
      core.bytes = Uint8Array.from([
        ...core.bytes.slice(0, from),
        ...BYTES.encode(insert),
        ...core.bytes.slice(to),
      ]);
      core.told.push("key");
    } else {
      core.bytes = BYTES.encode(sent.body);
      core.told.push("body");
    }
    core.check = sent.now;
    expect(new TextDecoder().decode(core.bytes)).toBe(sent.body);
    expect(sent.now.bytes).toBe(core.bytes.length);
    return Promise.resolve({ count: 0, sum: [0, 0] as const, from: 0, to: 0, html: "" });
  };
  const seeded = (now: string) => {
    core.bytes = BYTES.encode(now);
    core.check = checkOf(now);
    core.drawn += 1;
  };
  return { core, near, seeded };
};

describe("the core told each key alone, whatever the page goes through", () => {
  it("never needs the whole body across keys, undo, an edit before them and a whole drawing", async () => {
    const { core, near, seeded } = coreOf(BODY);
    let text = BODY;
    document.body.innerHTML = `<article class="page">${PLACED}</article>`;
    const page = writtenOn(document, {
      text: () => text,
      write: (next) => {
        text = next;
      },
      shown: () => {},
      redraw: seeded,
      near,
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    writing = page;
    const words = document.querySelector('[data-src="8-28"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, (words.textContent ?? "").indexOf(" dos"));
    const told = (count: number) => vi.waitFor(() => expect(core.told).toHaveLength(count));

    typed("a");
    await told(1);
    typed("b");
    await told(2);
    typed("", "historyUndo");
    await told(3);
    page.rewrite(text.replace("Hola", "Hola 😁"));
    await told(4);
    document.documentElement.classList.add("untitled");
    typed("c");
    await vi.waitFor(() => expect(core.drawn).toBe(1));
    document.documentElement.classList.remove("untitled");
    page.drawn();
    page.rewrite(`${text}fin\n`);
    await told(5);
    page.rewrite(text.replace("primero", "primer"));
    await told(6);
    patching.refused = 1;
    page.rewrite(text.replace("Uno", "Unos"));
    await told(7);
    await vi.waitFor(() => expect(core.drawn).toBe(2));
    page.drawn();
    page.rewrite(text.replace("Unos", "Uno"));
    await told(8);

    expect(core.told).toEqual(Array(8).fill("key"));
    expect(new TextDecoder().decode(core.bytes)).toBe(text);
  });
});
