import { fill, t } from "../locales";

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
  | "unfilter";

export type Sorting = { readonly column: number; readonly down: boolean; readonly order: number[] };

export type Filtering = {
  readonly column: number;
  readonly kind: "has" | "is" | "yes" | "no" | "soon" | "past";
  readonly said: string;
  readonly label: string;
};

/** How a table is shown on screen only: its order and what it hides, never written unless asked. */
export type Viewing = { readonly sort?: Sorting; readonly filters: Filtering[] };

const BOXES = ".page > .table[data-at]";

export const tablesIn = (page: Document): HTMLElement[] =>
  Array.from(page.querySelectorAll<HTMLElement>(BOXES));

const asking = (
  page: Document,
  act: TableAct,
  label: string,
  said: string,
  place: { row?: number; column?: number } = {},
): HTMLButtonElement => {
  const one = page.createElement("button");
  one.type = "button";
  one.dataset.asked = "table";
  one.dataset.act = act;
  if (place.row !== undefined) one.dataset.row = String(place.row);
  if (place.column !== undefined) one.dataset.column = String(place.column);
  one.setAttribute("aria-label", label);
  one.textContent = said;
  return one;
};

const headsOf = (box: Element): HTMLTableCellElement[] =>
  Array.from(box.querySelectorAll<HTMLTableCellElement>(":scope > table > thead th"));

const rowsOf = (box: Element): HTMLTableRowElement[] =>
  Array.from(box.querySelectorAll<HTMLTableRowElement>(":scope > table > tbody > tr"));

export const rowsCount = (page: Document, nth: number): number => {
  const box = tablesIn(page)[nth];
  return box ? rowsOf(box).length : 0;
};

const headed = (head: Element): string => head.textContent?.replace("⋯", "").trim() ?? "";

// The page arrives inert; the window adds what lets each table be written in place.
export const tabled = (page: Document) => {
  for (const box of tablesIn(page)) {
    if (box.classList.contains("dressed")) continue;
    box.classList.add("dressed");
    const bar = page.createElement("div");
    bar.className = "table-bar";
    bar.hidden = true;
    box.prepend(bar);
    headsOf(box).forEach((head, column) => {
      head.setAttribute("aria-sort", "none");
      const name = headed(head) || String(column + 1);
      head.append(asking(page, "column", fill("tableColumnMenu", name), "⋯", { column }));
    });
    rowsOf(box).forEach((row, at) => {
      row.dataset.row = String(at);
      const first = row.cells[0];
      if (!first) return;
      const grip = asking(page, "row", fill("tableRowMenu", String(at + 1)), "", { row: at });
      grip.classList.add("row-grip");
      first.prepend(grip);
    });
    const adding = asking(page, "rowAdd", t("tableRowAdd"), `+ ${t("tableRowAdd")}`);
    adding.classList.add("row-add");
    box.append(adding);
  }
};

// A click inside a dressed table: a cell to write, a head to sort by, or nothing of the table's.
export const tableClicked = (
  target: Element,
): { act: TableAct; row: number; column: number } | null => {
  if (!target.closest(`${BOXES}.dressed`) || target.closest(".table-input, .col-grip")) return null;
  const cell = target.closest<HTMLTableCellElement>("td, th");
  if (!cell) return null;
  if (cell.tagName === "TH") return { act: "sort", row: -1, column: cell.cellIndex };
  const row = cell.closest<HTMLTableRowElement>("tbody > tr")?.dataset.row;
  if (row === undefined) return null;
  return { act: "cell", row: Number(row), column: cell.cellIndex };
};

export const nthOf = (page: Document, at: [number, number]): number =>
  tablesIn(page).findIndex((one) => one.dataset.at === `${at[0]}-${at[1]}`);

export const placeOf = (page: Document, nth: number): [number, number] | null => {
  const said = tablesIn(page)[nth]?.dataset.at?.split("-").map(Number);
  return said?.length === 2 ? [said[0], said[1]] : null;
};

export const cellOf = (
  page: Document,
  nth: number,
  row: number,
  column: number,
): HTMLTableCellElement | null => {
  const box = tablesIn(page)[nth];
  if (!box) return null;
  if (row < 0) return headsOf(box)[column] ?? null;
  const line = rowsOf(box).find((one) => one.dataset.row === String(row));
  return line?.cells[column] ?? null;
};

export const headsNamed = (page: Document, nth: number): string[] =>
  headsOf(tablesIn(page)[nth] ?? page.createElement("div")).map(headed);

/** The rows shown, top to bottom, by their place in the file. */
export const rowsShown = (page: Document, nth: number): number[] =>
  rowsOf(tablesIn(page)[nth] ?? page.createElement("div"))
    .filter((one) => !one.hidden)
    .map((one) => Number(one.dataset.row));

const keyOf = (cell: HTMLTableCellElement | undefined): string =>
  cell?.dataset.key ?? cell?.textContent?.trim().toLowerCase() ?? "";

// A date or a yes is known only by the key the core gave it; a value written amiss has none and is kept out.
const passes = (cell: HTMLTableCellElement | undefined, one: Filtering, today: string): boolean => {
  const read = cell?.dataset.key;
  if (one.kind === "has")
    return (cell?.textContent ?? "").toLowerCase().includes(one.said.toLowerCase());
  if (one.kind === "is")
    return keyOf(cell)
      .split(",")
      .some((part) => part.trim() === one.said);
  if (one.kind === "yes") return read === "1";
  if (one.kind === "no") return read === "0" || (read === undefined && !cell?.textContent?.trim());
  if (one.kind === "soon") return read !== undefined && read >= today;
  return read !== undefined && read < today;
};

// The order and the hiding are laid on the drawn rows; the file and the totals stay as they are.
export const viewed = (page: Document, nth: number, view: Viewing | undefined, today: string) => {
  const box = tablesIn(page)[nth];
  const body = box?.querySelector<HTMLElement>(":scope > table > tbody");
  if (!box || !body) return;
  const rows = rowsOf(box);
  const byPlace = new Map(rows.map((one) => [Number(one.dataset.row), one]));
  const order =
    view?.sort?.order ?? rows.map((one) => Number(one.dataset.row)).sort((a, b) => a - b);
  for (const at of order) {
    const row = byPlace.get(at);
    if (row) body.append(row);
  }
  const filters = view?.filters ?? [];
  for (const row of rows) {
    row.hidden = !filters.every((one) => passes(row.cells[one.column], one, today));
  }
  headsOf(box).forEach((head, column) => {
    const sorted = view?.sort?.column === column;
    head.setAttribute(
      "aria-sort",
      sorted ? (view?.sort?.down ? "descending" : "ascending") : "none",
    );
  });
  barred(page, box, view);
};

const barred = (page: Document, box: HTMLElement, view: Viewing | undefined) => {
  const bar = box.querySelector<HTMLElement>(":scope > .table-bar");
  if (!bar) return;
  bar.replaceChildren();
  const sort = view?.sort;
  if (sort) {
    const said = page.createElement("span");
    said.textContent = fill(
      "tableSortedBy",
      headsNamed(page, tablesIn(page).indexOf(box))[sort.column] ?? "",
    );
    bar.append(
      said,
      asking(page, "keep", t("tableKeepOrder"), t("tableKeepOrder")),
      asking(page, "unsort", t("tableUnsort"), t("tableUnsort")),
    );
  }
  (view?.filters ?? []).forEach((one, at) => {
    const chip = page.createElement("span");
    chip.className = "table-chip";
    chip.textContent = one.label;
    chip.append(asking(page, "unfilter", fill("tableFilterOff", one.label), "×", { column: at }));
    bar.append(chip);
  });
  bar.hidden = bar.childElementCount === 0;
};

/** How the writing in a cell ended: where the caret goes next, or nowhere. */
export type Leaving = "enter" | "tab" | "back" | "up" | "down" | "blur" | "cancel";

// A plain field over the cell: what it holds is the cell's Markdown, and the core writes it back.
export const typing = (
  holder: HTMLElement,
  said: string,
  label: string,
  done: (said: string, how: Leaving) => void,
): HTMLInputElement => {
  const input = holder.ownerDocument.createElement("input");
  input.className = "table-input";
  input.value = said;
  input.setAttribute("aria-label", label);
  let ended = false;
  const end = (how: Leaving) => {
    if (ended) return;
    ended = true;
    input.remove();
    done(input.value, how);
  };
  // What is typed here is the cell's, never a shortcut of the page around it.
  input.addEventListener("keydown", (event) => {
    event.stopPropagation();
    const ways: Record<string, Leaving | undefined> = {
      Enter: "enter",
      Escape: "cancel",
      Tab: event.shiftKey ? "back" : "tab",
      ArrowUp: event.altKey ? "up" : undefined,
      ArrowDown: event.altKey ? "down" : undefined,
    };
    const how = ways[event.key];
    if (!how) return;
    event.preventDefault();
    end(how);
  });
  input.addEventListener("blur", () => end("blur"));
  holder.append(input);
  input.focus();
  input.select();
  return input;
};

/** Where the caret goes after a cell, among the rows shown: a cell, a new row past the last, or nowhere. */
export const beside = (
  shown: number[],
  heads: number,
  row: number,
  column: number,
  how: Leaving,
): { row: number; column: number } | "new" | null => {
  const at = shown.indexOf(row);
  if (how === "tab") {
    if (column + 1 < heads) return { row, column: column + 1 };
    return at + 1 < shown.length ? { row: shown[at + 1], column: 0 } : "new";
  }
  if (how === "back") {
    if (column > 0) return { row, column: column - 1 };
    return at > 0 ? { row: shown[at - 1], column: heads - 1 } : null;
  }
  if (how === "enter")
    return at >= 0 && at + 1 < shown.length ? { row: shown[at + 1], column } : null;
  return null;
};

// A row's grip is dragged up or down the rows shown; it lands on the row under the pointer.
export const dragging = (
  page: Document,
  moved: (box: HTMLElement, from: number, to: number) => void,
) => {
  page.addEventListener("pointerdown", (event) => {
    const grip = (event.target as Element | null)?.closest?.<HTMLElement>(".row-grip");
    const box = grip?.closest<HTMLElement>(BOXES);
    if (!grip || !box || event.button !== 0) return;
    const from = Number(grip.dataset.row);
    const startY = event.clientY;
    let to = from;
    let far = false;
    const over = (next: PointerEvent) => {
      far ||= Math.abs(next.clientY - startY) > 4;
      if (!far) return;
      const under = rowsOf(box).find((one) => {
        const edge = one.getBoundingClientRect();
        return !one.hidden && next.clientY >= edge.top && next.clientY < edge.bottom;
      });
      if (under) to = Number(under.dataset.row);
      box.classList.add("row-moving");
    };
    const up = () => {
      page.removeEventListener("pointermove", over);
      page.removeEventListener("pointerup", up);
      box.classList.remove("row-moving");
      if (!far) return;
      // The click that ends a drag is not a click on the grip, and no later click is lost to it.
      const swallowed = (one: Event) => one.stopPropagation();
      page.addEventListener("click", swallowed, { capture: true, once: true });
      setTimeout(() => page.removeEventListener("click", swallowed, { capture: true }), 0);
      if (to !== from) moved(box, from, to);
    };
    page.addEventListener("pointermove", over);
    page.addEventListener("pointerup", up);
  });
};
