/** What a click on a table asks for; `row` is the row's place in the file, under the head, from 0. */
export type TableAct =
  | "cell"
  | "head"
  | "row"
  | "column"
  | "sort"
  | "rowAdd"
  | "unsort"
  | "keep"
  | "unfilter"
  | "star"
  | "toggle";

export const BOXES = ".page > .table[data-at]";

export const tablesIn = (page: Document): HTMLElement[] =>
  Array.from(page.querySelectorAll<HTMLElement>(BOXES));

export const asking = (
  page: Document,
  act: TableAct,
  label: string,
  said: string,
  place: { row?: number; column?: number; to?: number } = {},
): HTMLButtonElement => {
  const one = page.createElement("button");
  one.type = "button";
  one.dataset.asked = "table";
  one.dataset.act = act;
  if (place.row !== undefined) one.dataset.row = String(place.row);
  if (place.column !== undefined) one.dataset.column = String(place.column);
  if (place.to !== undefined) one.dataset.to = String(place.to);
  one.setAttribute("aria-label", label);
  one.textContent = said;
  return one;
};
