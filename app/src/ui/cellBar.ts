import { docked, FORMATS, formatButtons, PENS } from "tisty-editor/edit";
import { t } from "../locales";
import { modKey } from "./macKeys";
import { holds, type Mark } from "./peeled";

/** The bar over a table while one of its cells is written: each press puts a mark on the whole cell or takes it off. */
export const cellBar = (
  box: HTMLElement,
  input: HTMLInputElement,
  marks: () => readonly Mark[],
  press: (open: string, close: string) => void,
): (() => void) => {
  const page = box.ownerDocument;
  const bar = page.createElement("div");
  bar.className = "cm-bar cell-bar";
  bar.setAttribute("role", "toolbar");
  bar.setAttribute("aria-label", t("formatting"));
  const shown: [HTMLElement, string, string][] = [];
  const lit = () => {
    for (const [one, open, close] of shown)
      one.setAttribute("aria-pressed", String(holds(marks(), open, close)));
  };
  const pressed = (open: string, close: string) => {
    press(open, close);
    lit();
  };
  const { formats, pens } = formatButtons(page, t, pressed);
  // A link is not a mark a whole cell takes from a button; it is written in the cell's Markdown.
  FORMATS.forEach(([, , open, close], at) => {
    if (open !== "[") shown.push([formats[at], open, close]);
  });
  const marked = shown.map(([one]) => one);
  PENS.forEach(([, , close], at) => {
    shown.push([pens[at], "==", close]);
  });
  const gap = page.createElement("span");
  gap.className = "cm-bar-sep";
  bar.append(...marked, gap, ...pens);
  lit();
  // The field keeps the editor's own keys for the marks the bar shows.
  input.addEventListener("keydown", (event) => {
    if (!modKey(event)) return;
    const said = `${event.shiftKey ? "Shift " : ""}${event.key.toUpperCase()}`;
    const found = FORMATS.find(([, keys, open]) => keys === said && open !== "[");
    if (!found) return;
    event.preventDefault();
    pressed(found[2], found[3]);
  });
  // Like the page's own bar it waits for words chosen in the cell, leaving the lines around the table free.
  const chosen = () => {
    bar.hidden = input.selectionStart === input.selectionEnd;
  };
  for (const kind of ["select", "keyup", "mouseup", "input", "focus"])
    input.addEventListener(kind, chosen);
  chosen();
  const hint = page.createElement("span");
  hint.hidden = true;
  // Over the cell being written, ending at its right edge, as the page's bar sits over the words chosen.
  const cell = input.parentElement ?? box;
  const ending = () =>
    Math.min(cell.getBoundingClientRect().right, box.getBoundingClientRect().right);
  return docked(bar, hint, () => cell.getBoundingClientRect().top, undefined, false, ending).stop;
};
