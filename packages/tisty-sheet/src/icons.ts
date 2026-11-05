import { isHue } from "./hues";

// The whole set outweighs the rest of the sheet, so it comes only for a page that names an icon.
export const iconed = async (page: Document | HTMLElement): Promise<void> => {
  const named = [...page.querySelectorAll<HTMLElement>("span.ico[data-ico]")];
  if (named.length === 0) return;
  // A page whose icons could not be fetched still reads, without them.
  const glyphs = await import("./glyphs").catch(() => null);
  if (!glyphs) return;
  const { markup } = glyphs;
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
