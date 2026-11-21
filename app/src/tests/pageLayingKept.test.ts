import { afterEach, describe, expect, it, vi } from "vitest";
import type { PlacedNear, Sent } from "../placing";
import { type Hands, type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nUno dos tres\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-21"><span data-src="8-21">Uno dos tres</span></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (near: Hands["near"], redraw: Hands["redraw"] = () => {}) => {
  let text = BODY;
  document.body.innerHTML = `<article class="page">${PLACED}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw,
    near,
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
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

const caretAfter = (said: string) => {
  const words = document.querySelector('[data-src="8-21"]')?.firstChild as Text;
  document.getSelection()?.collapse(words, (words.textContent ?? "").indexOf(said) + said.length);
};

describe("the keys after one whose drawing went wrong", () => {
  it("still reach the core when drawing the page again threw", async () => {
    const asked: Sent[] = [];
    let redraws = 0;
    const reported = vi.fn();
    window.addEventListener("error", reported);
    opened(
      (one) => {
        asked.push(one);
        return Promise.reject(new Error("no core here"));
      },
      () => {
        redraws += 1;
        if (redraws === 1) throw new Error("the page could not be drawn");
        writing?.drawn();
      },
    );
    caretAfter("dos");
    typed("a");
    await vi.waitFor(() => expect(redraws).toBeGreaterThan(0));
    typed("b");
    await vi.waitFor(() => expect(asked).toHaveLength(2));
    expect(reported).toHaveBeenCalled();
    window.removeEventListener("error", reported);
  });

  it("go on when the core never answers the one before", async () => {
    const asked: Sent[] = [];
    let redraws = 0;
    opened(
      (one) => {
        asked.push(one);
        return asked.length === 1
          ? new Promise<PlacedNear>(() => {})
          : Promise.reject(new Error("no core here"));
      },
      () => {
        redraws += 1;
        writing?.drawn();
      },
    );
    caretAfter("dos");
    typed("a");
    typed("b");
    await vi.waitFor(() => expect(asked).toHaveLength(1));
    await vi.advanceTimersByTimeAsync(10_000);
    await vi.waitFor(() => expect(asked).toHaveLength(2));
    expect(redraws).toBeGreaterThan(0);
  });
});
