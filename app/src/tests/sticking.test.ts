import { describe, expect, it, vi } from "vitest";
import { movedBlock, sticking } from "../ui/sticking";

const NOTE = "> [!IDEA] 2026-11-12 · pink · right\n> Llamar al vet";
const TEXT = `# Casa\n\nUno.\n\n${NOTE}\n\nDos.\n\nTres.\n`;
const spanOf = (text: string, said: string): [number, number] => {
  const from = text.indexOf(said);
  return [from, from + said.length];
};

describe("a sticky note carried to another place in the text", () => {
  it("lands below a block further down, with one blank line on each side", () => {
    const [, end] = spanOf(TEXT, "Tres.");
    expect(movedBlock(TEXT, spanOf(TEXT, NOTE), end)).toBe(
      `# Casa\n\nUno.\n\nDos.\n\nTres.\n\n${NOTE}\n`,
    );
  });

  it("lands above a block further up and leaves no gap where it was", () => {
    const [start] = spanOf(TEXT, "Uno.");
    expect(movedBlock(TEXT, spanOf(TEXT, NOTE), start)).toBe(
      `# Casa\n\n${NOTE}\n\nUno.\n\nDos.\n\nTres.\n`,
    );
  });

  it("stays put when it lands on itself, and keeps a file's Windows line endings", () => {
    const span = spanOf(TEXT, NOTE);
    expect(movedBlock(TEXT, span, span[0])).toBe(TEXT);
    const windows = TEXT.replace(/\n/g, "\r\n");
    const note = NOTE.replace(/\n/g, "\r\n");
    const [start] = spanOf(windows, "Uno.");
    expect(movedBlock(windows, spanOf(windows, note), start)).toBe(
      `# Casa\r\n\r\n${note}\r\n\r\nUno.\r\n\r\nDos.\r\n\r\nTres.\r\n`,
    );
  });
});

const page = () => {
  const sheet = document.implementation.createHTMLDocument();
  sheet.body.innerHTML = `<article class="page">
    <p data-at="8-12">Uno.</p>
    <blockquote class="callout" data-callout="idea" data-at="14-60">Idea</blockquote>
    <p data-at="62-66">Dos.</p>
  </article>`;
  const blocks = Array.from(sheet.querySelectorAll<HTMLElement>("[data-at]"));
  blocks.forEach((one, at) => {
    one.getBoundingClientRect = () =>
      ({ top: at * 50, bottom: at * 50 + 40, height: 40 }) as DOMRect;
  });
  return { sheet, blocks };
};

const press = (target: EventTarget, type: string, y: number) =>
  target.dispatchEvent(new MouseEvent(type, { bubbles: true, button: 0, clientY: y }));

describe("carrying a sticky note on the page", () => {
  it("lands it below the block it is let go over, and only once it went far enough", () => {
    const { sheet, blocks } = page();
    const [, note, below] = blocks;
    const moved = vi.fn();
    sticking(sheet, () => moved);
    press(note, "pointerdown", 60);
    press(sheet, "pointermove", 62);
    press(sheet, "pointerup", 62);
    expect(moved).not.toHaveBeenCalled();
    press(note, "pointerdown", 60);
    press(sheet, "pointermove", 125);
    expect(below.classList.contains("idea-landing-below")).toBe(true);
    press(sheet, "pointerup", 125);
    expect(moved).toHaveBeenCalledWith([14, 60], 66);
    expect(below.classList.contains("idea-landing-below")).toBe(false);
  });

  it("does nothing on a page that cannot be written", () => {
    const { sheet, blocks } = page();
    const [first, note] = blocks;
    sticking(sheet, () => undefined);
    press(note, "pointerdown", 60);
    press(sheet, "pointermove", 5);
    expect(first.classList.contains("idea-landing-above")).toBe(false);
  });
});
