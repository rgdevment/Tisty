import { SHEET_GLYPHS, type SheetGlyph } from "./sheetGlyphs";

export const piece = (page: Document, kind: string, said = "") => {
  const one = page.createElement("span");
  one.className = kind;
  one.textContent = said;
  return one;
};

export const drawnGlyph = (page: Document, paths: string) => {
  const drawn = page.createElementNS("http://www.w3.org/2000/svg", "svg");
  drawn.setAttribute("class", "glyph");
  drawn.setAttribute("viewBox", "0 0 24 24");
  drawn.setAttribute("aria-hidden", "true");
  drawn.innerHTML = paths;
  return drawn;
};

export const glyph = (page: Document, name: SheetGlyph) => drawnGlyph(page, SHEET_GLYPHS[name]);
