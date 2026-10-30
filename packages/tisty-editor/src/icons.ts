import { markup } from "./glyphs";
import { isHue } from "./shell";

// Each icon the text names is drawn from the glyphs carried here, so no page ever reaches out for one.
export const iconed = (page: Document | HTMLElement) => {
  const made = "createElementNS" in page ? page : page.ownerDocument;
  for (const one of page.querySelectorAll<HTMLElement>("span.ico[data-ico]")) {
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
