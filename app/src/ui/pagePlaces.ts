import { spanned } from "tisty-editor/read";

export type Placed = "data-at" | "data-src" | "data-placed";
export type Place = [number, number];

const NAMES: readonly Placed[] = ["data-at", "data-src", "data-placed"];
const ANY = "[data-at], [data-src], [data-placed]";

// Writing every later place on each key costs more than the key itself on a long page, so they wait for the keys to rest.
type Moving = {
  readonly sheet: Element;
  readonly first: Element;
  readonly from: number;
  readonly by: number;
  readonly laid: ReadonlySet<Element>;
};

const moving = new WeakMap<Document, Moving>();

const pending = (page: Document): Moving | null => {
  const one = moving.get(page);
  if (one && one.first.parentElement === one.sheet) return one;
  moving.delete(page);
  return null;
};

const shifted = (block: Element, by: number) => {
  for (const one of [block, ...block.querySelectorAll(ANY)]) {
    for (const name of NAMES) {
      const at = spanned(one.getAttribute(name));
      if (at) one.setAttribute(name, `${at[0] + by}-${at[1] + by}`);
    }
  }
};

export const placesSettled = (page: Document) => {
  const one = pending(page);
  moving.delete(page);
  if (!one) return;
  for (let block: Element | null = one.first; block; block = block.nextElementSibling)
    if (block.hasAttribute("data-at")) shifted(block, one.by);
};

const topIn = (sheet: Element, one: Element) => {
  let top = one;
  while (top.parentElement && top.parentElement !== sheet) top = top.parentElement;
  return top.parentElement === sheet ? top : null;
};

export const placeOf = (
  one: Element | null | undefined,
  name: Placed = "data-at",
): Place | null => {
  const said = spanned(one?.getAttribute(name) ?? null);
  if (!said || !one) return said;
  const moved = pending(one.ownerDocument);
  if (!moved || said[0] < moved.from) return said;
  const top = topIn(moved.sheet, one);
  if (!top || moved.laid.has(top)) return said;
  return [said[0] + moved.by, said[1] + moved.by];
};

export const placeSaid = (one: Element | null | undefined, name: Placed = "data-at") => {
  const at = placeOf(one, name);
  return at ? `${at[0]}-${at[1]}` : null;
};

export const samePlace = (one: Element | null | undefined, at: readonly [number, number]) => {
  const own = placeOf(one);
  return own !== null && own[0] === at[0] && own[1] === at[1];
};

export const placeWritten = (
  one: Element,
  at: readonly [number, number],
  name: Placed = "data-at",
) => {
  placesSettled(one.ownerDocument);
  one.setAttribute(name, `${at[0]}-${at[1]}`);
};

export const placeCopied = (
  from: Element,
  to: Element,
  name: Placed = "data-at",
  as: Placed = name,
) => {
  placesSettled(from.ownerDocument);
  const said = from.getAttribute(name);
  if (said) to.setAttribute(as, said);
};

export const placedAt = <E extends Element = Element>(
  page: Document,
  selector: string,
  at: readonly [number, number] | number,
): E | null => {
  placesSettled(page);
  const said = typeof at === "number" ? `^="${at}-"` : `="${at[0]}-${at[1]}"`;
  return page.querySelector<E>(`${selector}[data-at${said}]`);
};

// Only a key that reaches the first block still unmoved can add to what waits; any other writes it first.
export const keyMoved = (
  sheet: Element,
  old: readonly Element[],
  from: number,
  kept: number,
  laid: readonly Element[],
  by: number,
) => {
  const page = sheet.ownerDocument;
  const was = pending(page);
  const first = old[kept] ?? null;
  const going = old.slice(from, kept);
  const joins = was !== null && (was.first === first || going.includes(was.first));
  if (was && !joins) placesSettled(page);
  const total = (joins && was ? was.by : 0) + by;
  const start = first ? spanned(first.getAttribute("data-at"))?.[0] : undefined;
  if (!first || total === 0 || start === undefined) {
    moving.delete(page);
    return;
  }
  const still = joins && was ? [...was.laid].filter((one) => !going.includes(one)) : [];
  moving.set(page, { sheet, first, from: start, by: total, laid: new Set([...still, ...laid]) });
};
