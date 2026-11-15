import { charsAt, spanned } from "tisty-editor/read";

export type Gone = { readonly from: number; readonly to: number };

export const PICTURE = "img[data-src]";

export const aloneOnLine = (block: Element) =>
  block.matches("p[data-at]") &&
  block.textContent?.trim() === "" &&
  block.querySelectorAll("[data-src]").length === 1 &&
  Boolean(block.querySelector(PICTURE));

// A picture alone on its line takes the line with it; one among words leaves a single space between them.
export const pictureGone = (text: string, picture: Element): Gone | null => {
  const own = spanned(picture.getAttribute("data-src"));
  if (!own) return null;
  const block = picture.closest(".page > [data-at]");
  const line = block && aloneOnLine(block) ? spanned(block.getAttribute("data-at")) : null;
  if (block && line) {
    const next = spanned(block.nextElementSibling?.getAttribute("data-at") ?? null)?.[0];
    let from = charsAt(text, line[0]);
    if (next === undefined) while (text[from - 1] === "\n" && text[from - 2] === "\n") from--;
    return { from, to: charsAt(text, next ?? line[1]) };
  }
  const from = charsAt(text, own[0]);
  const to = charsAt(text, own[1]);
  return { from, to: text[from - 1] === " " && text[to] === " " ? to + 1 : to };
};
