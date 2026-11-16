import { spanned } from "tisty-editor/read";
import type { PlacedNear } from "../placing";

const NAMES = ["data-at", "data-src", "data-placed"];
// What the page lends or links in a block is laid by its own dressing, which only a whole drawing runs.
const WHOLE_ONLY = "code.language-widget, figure.widget, .doc-card";

const shifted = (block: Element, by: number) => {
  for (const one of [block, ...block.querySelectorAll("[data-at], [data-src], [data-placed]")]) {
    for (const name of NAMES) {
      const at = spanned(one.getAttribute(name));
      if (at) one.setAttribute(name, `${at[0] + by}-${at[1] + by}`);
    }
  }
};

const folds = (blocks: readonly Element[]) =>
  blocks.flatMap((one) => [
    ...(one.matches("details") ? [one as HTMLDetailsElement] : []),
    ...one.querySelectorAll("details"),
  ]);

export const spansSum = (spans: Iterable<readonly [number, number]>): [number, number] => {
  let fnv = 0x811c9dc5;
  let mixed = 0x9747b28c;
  for (const span of spans) {
    for (const at of span) {
      fnv = Math.imul(fnv ^ at, 0x01000193);
      mixed = Math.imul(mixed ^ at, 0x5bd1e995);
      mixed ^= mixed >>> 13;
    }
  }
  return [fnv >>> 0, mixed >>> 0];
};

const placeOf = (one: Element, by: number): [number, number] | null => {
  const at = spanned(one.getAttribute("data-at"));
  return at && [at[0] + by, at[1] + by];
};

const linedUp = (standing: readonly Element[], near: PlacedNear, by: number) => {
  const places = standing.map((one, at) => placeOf(one, at < near.from ? 0 : by));
  if (places.some((one) => !one)) return false;
  const [fnv, mixed] = spansSum(places as [number, number][]);
  return fnv === near.sum[0] && mixed === near.sum[1];
};

// The blocks the core drew again take the place of the old ones, and those after only move by
// what the edit added; when the page does not line up with the core, it is drawn whole instead.
export const patched = (sheet: Element, near: PlacedNear, by: number): Element[] | null => {
  const old = Array.from(sheet.querySelectorAll(":scope > [data-at]"));
  const { count, from, to } = near;
  const after = count - to;
  const kept = old.length - after;
  if (after < 0 || kept < from) return null;
  if (!linedUp([...old.slice(0, from), ...old.slice(kept)], near, by)) return null;
  const page = sheet.ownerDocument;
  const fresh = page.createElement("template");
  fresh.innerHTML = near.html;
  const laid = Array.from(fresh.content.children);
  if (
    laid.length !== to - from ||
    laid.some((one) => one.matches(WHOLE_ONLY) || one.querySelector(WHOLE_ONLY))
  )
    return null;
  // A fold drawn anew comes closed, so one being written in stays open as it was.
  const opened = folds(old.slice(from, kept)).map((one) => one.open);
  folds(laid).forEach((one, at) => {
    if (opened[at]) one.open = true;
  });
  for (const one of old.slice(from, kept)) one.remove();
  for (const one of sheet.querySelectorAll(":scope > [data-ghost]")) one.remove();
  const next = old[kept] ?? null;
  const before = from > 0 ? old[from - 1] : null;
  if (next) next.before(fresh.content);
  else if (before) before.after(fresh.content);
  else sheet.prepend(fresh.content);
  if (by !== 0) for (const one of old.slice(kept)) shifted(one, by);
  return laid;
};
