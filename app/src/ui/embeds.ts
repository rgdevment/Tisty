import { charsAt } from "tisty-editor/read";
import { placeOf } from "./pagePlaces";

export type Gone = { readonly from: number; readonly to: number };

// A file drawn in place of its picture, as a card or a player, keeps the picture's place in `data-placed`.
export const EMBED = "img[data-src], [data-placed]";

const DRESSING = "button, .picture-away";

export const embedPlace = (shown: Element) =>
  placeOf(shown, "data-src") ?? placeOf(shown, "data-placed");

export const aloneOnLine = (block: Element) => {
  if (!block.matches("p[data-at]")) return false;
  const shown = Array.from(block.children).filter((one) => !one.matches(DRESSING));
  return (
    shown.length === 1 &&
    shown[0].matches(EMBED) &&
    Array.from(block.childNodes).every((one) => one.nodeType !== 3 || !one.textContent?.trim())
  );
};

// A file alone on its line takes the line with it; one among words leaves a single space between them.
export const embedGone = (text: string, shown: Element): Gone | null => {
  const own = embedPlace(shown);
  if (!own) return null;
  const block = shown.closest(".page > [data-at]");
  const line = block && aloneOnLine(block) ? placeOf(block) : null;
  if (block && line) {
    const next = placeOf(block.nextElementSibling)?.[0];
    let from = charsAt(text, line[0]);
    if (next === undefined) while (text[from - 1] === "\n" && text[from - 2] === "\n") from--;
    return { from, to: charsAt(text, next ?? line[1]) };
  }
  return spacedOut(text, charsAt(text, own[0]), charsAt(text, own[1]));
};

export const spacedOut = (text: string, from: number, to: number): Gone => ({
  from,
  to: text[from - 1] === " " && text[to] === " " ? to + 1 : to,
});
