import { describe, expect, it, vi } from "vitest";
import { picked, quickly } from "../ui/quickly";

vi.mock("../glyphs", () => ({ markup: () => '<path d="M1 1"/>' }));

// The page's own document has no selection in the test window, so the window's document stands in.
const page = (html: string): Document => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
  return document;
};

const choose = (drawn: Document, node: Node, from: number, to: number) => {
  const range = drawn.createRange();
  range.setStart(node, from);
  range.setEnd(node, to);
  const chosen = drawn.getSelection();
  chosen?.removeAllRanges();
  chosen?.addRange(range);
};

describe("words picked on the drawn page", () => {
  it("names the block, the words before them and the words themselves", () => {
    const drawn = page('<p data-at="0-20">Rina likes her walks</p>');
    const words = drawn.querySelector("p")?.firstChild as Node;
    choose(drawn, words, 11, 14);
    expect(picked(drawn)).toMatchObject({ at: [0, 20], read: "Rina likes ", said: "her" });
  });

  it("offers nothing for a picking that spans two blocks or none at all", () => {
    const drawn = page('<p data-at="0-4">Rina</p><p data-at="6-10">Toto</p>');
    const [one, two] = Array.from(drawn.querySelectorAll("p"));
    const range = drawn.createRange();
    range.setStart(one.firstChild as Node, 1);
    range.setEnd(two.firstChild as Node, 2);
    drawn.getSelection()?.addRange(range);
    expect(picked(drawn)).toBeNull();
    drawn.getSelection()?.removeAllRanges();
    expect(picked(drawn)).toBeNull();
  });

  it("writes over the words picked with the key pressed, and keeps them picked on Enter", () => {
    const drawn = page('<p data-at="0-20">Rina likes her walks</p>');
    const onChoose = vi.fn();
    quickly(drawn, () => ({ onChoose }));
    const words = drawn.querySelector("p")?.firstChild as Node;
    choose(drawn, words, 11, 14);
    drawn.dispatchEvent(new KeyboardEvent("keydown", { key: "h", cancelable: true }));
    expect(onChoose).toHaveBeenLastCalledWith(expect.objectContaining({ said: "her", typed: "h" }));
    drawn.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", cancelable: true }));
    expect(onChoose).toHaveBeenLastCalledWith(expect.objectContaining({ typed: null }));
    drawn.dispatchEvent(new KeyboardEvent("keydown", { key: "Backspace", cancelable: true }));
    expect(onChoose).toHaveBeenLastCalledWith(expect.objectContaining({ typed: "" }));
    drawn.dispatchEvent(
      new KeyboardEvent("keydown", { key: "c", ctrlKey: true, cancelable: true }),
    );
    drawn.getSelection()?.removeAllRanges();
    drawn.dispatchEvent(new KeyboardEvent("keydown", { key: "x", cancelable: true }));
    expect(onChoose).toHaveBeenCalledTimes(3);
  });

  it("asks for a format with what was picked, and leaves the page alone when nobody can write it", async () => {
    const drawn = page('<p data-at="0-20">Rina likes her walks</p>');
    const onFormat = vi.fn();
    let writable = true;
    quickly(drawn, () => (writable ? { onFormat } : {}));
    const bar = drawn.querySelector<HTMLElement>(".quick-bar");
    expect(bar?.hidden).toBe(true);
    choose(drawn, drawn.querySelector("p")?.firstChild as Node, 11, 14);
    drawn.dispatchEvent(new MouseEvent("mouseup"));
    await new Promise((done) => setTimeout(done, 0));
    expect(bar?.hidden).toBe(false);
    bar?.querySelector<HTMLButtonElement>("button")?.click();
    expect(onFormat).toHaveBeenCalledWith(
      expect.objectContaining({ said: "her", read: "Rina likes ", open: "**", close: "**" }),
    );
    expect(bar?.hidden).toBe(true);
    writable = false;
    choose(drawn, drawn.querySelector("p")?.firstChild as Node, 0, 4);
    drawn.dispatchEvent(new MouseEvent("mouseup"));
    await new Promise((done) => setTimeout(done, 0));
    expect(bar?.hidden).toBe(true);
  });

  it("hands a right click to the window, saying whether it fell on a block being written", () => {
    const drawn = page('<p data-at="0-4">Rina</p><div class="editing"><span>x</span></div>');
    const onMenu = vi.fn();
    quickly(drawn, () => ({ onMenu }));
    const asked = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
      clientX: 5,
      clientY: 6,
    });
    drawn.querySelector("p")?.dispatchEvent(asked);
    expect(asked.defaultPrevented).toBe(true);
    expect(onMenu).toHaveBeenLastCalledWith(
      expect.objectContaining({
        x: 5,
        y: 6,
        writing: false,
        block: expect.objectContaining({ at: [0, 4] }),
      }),
    );
    drawn
      .querySelector(".editing span")
      ?.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true }));
    expect(onMenu).toHaveBeenLastCalledWith(
      expect.objectContaining({ writing: true, block: null }),
    );
  });
});
