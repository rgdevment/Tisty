import { isHue } from "./shell";

// Each icon the text names is drawn from the glyphs carried here, so no page ever reaches out for one; the whole
// set outweighs the rest of the sheet, so it comes only for a page that names an icon.
export const iconed = async (page: Document | HTMLElement): Promise<void> => {
  const named = [...page.querySelectorAll<HTMLElement>("span.ico[data-ico]")];
  if (named.length === 0) return;
  const { markup } = await import("./glyphs");
  const made = "createElementNS" in page ? page : page.ownerDocument;
  for (const one of named) {
    const drawn = markup(one.dataset.ico ?? "");
    if (!drawn) continue;
    const hue = one.dataset.hue ?? "";
    if (isHue(hue)) one.classList.add(`ico-${hue}`);
    const glyph = made.createElementNS("http://www.w3.org/2000/svg", "svg");
    glyph.setAttribute("class", "glyph");
    glyph.setAttribute("viewBox", "0 0 24 24");
    glyph.setAttribute("aria-hidden", "true");
    glyph.innerHTML = drawn;
    one.replaceChildren(glyph);
  }
};
