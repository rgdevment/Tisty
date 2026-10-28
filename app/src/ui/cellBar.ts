import { t } from "../locales";
import { FORMATS, formatButtons, PENS } from "./barring";
import { docked } from "./docking";
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
  const { formats, pens } = formatButtons(page, pressed);
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
    if (!(event.ctrlKey || event.metaKey)) return;
    const said = `${event.shiftKey ? "Shift " : ""}${event.key.toUpperCase()}`;
    const found = FORMATS.find(([, keys, open]) => keys === said && open !== "[");
    if (!found) return;
    event.preventDefault();
    pressed(found[2], found[3]);
  });
  const hint = page.createElement("span");
  hint.hidden = true;
  // Over the table, never over its rows, and kept in sight while a long table scrolls.
  return docked(bar, hint, () => box.getBoundingClientRect().top).stop;
};
