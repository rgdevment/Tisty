import { spanned } from "tisty-editor/read";
import { namesSum, type PlacedNear, summed } from "../placing";
import { placeOf } from "./pageSpots";

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

const linedUp = (old: readonly Element[], near: PlacedNear, kept: number, by: number) => {
  const places: number[] = [];
  for (let at = 0; at < old.length; at++) {
    if (at >= near.from && at < kept) continue;
    const place = placeOf(old[at]);
    if (!place) return false;
    const moved = at < near.from ? 0 : by;
    places.push(place[0] + moved, place[1] + moved);
  }
  const [fnv, mixed] = summed(places);
  return fnv === near.sum[0] && mixed === near.sum[1];
};

const LEVELS = [1, 2, 3, 4, 5, 6];
const HEADINGS = LEVELS.map((level) => `h${level}[data-at]`).join(", ");
const HEADS = LEVELS.map((level) => `h${level}[data-at][id]`).join(", ");

const headsIn = (blocks: readonly Element[], which = HEADS) =>
  blocks.flatMap((one) => [...(one.matches(which) ? [one] : []), ...one.querySelectorAll(which)]);

const namedAlike = (sheet: Element, old: readonly Element[], near: PlacedNear, kept: number) => {
  if (!near.heads) return true;
  const redrawn = old.slice(near.from, kept);
  const standing = Array.from(sheet.querySelectorAll(HEADS)).filter(
    (one) => !redrawn.some((block) => block.contains(one)),
  );
  const [fnv, mixed] = namesSum(standing.map((one) => one.id));
  return fnv === near.heads[0] && mixed === near.heads[1];
};

// Only the same heading in the same place keeps its shown name; one that came or went renames those after it.
const namesCarried = (was: readonly Element[], laid: readonly Element[], by: number) => {
  const before = headsIn(was, HEADINGS);
  const after = headsIn(laid, HEADINGS);
  const same = (one: Element, at: number) => {
    const old = placeOf(before[at]);
    const now = placeOf(one);
    if (!old || !now || one.tagName !== before[at].tagName) return false;
    const moved = now[0] - old[0];
    return now[1] - moved === old[1] && (moved === 0 || moved === by);
  };
  if (before.length !== after.length || !after.every((one, at) => same(one, at))) return false;
  after.forEach((one, at) => {
    if (before[at].id) one.id = before[at].id;
  });
  return true;
};

// The blocks the core drew again take the place of the old ones, and those after only move by
// what the edit added; when the page does not line up with the core, it is drawn whole instead.
export const patched = (sheet: Element, near: PlacedNear, by: number): Element[] | null => {
  if (near.whole) return null;
  const old = Array.from(sheet.querySelectorAll(":scope > [data-at]"));
  const { count, from, to } = near;
  const after = count - to;
  const kept = old.length - after;
  if (after < 0 || kept < from) return null;
  if (!linedUp(old, near, kept, by) || !namedAlike(sheet, old, near, kept)) return null;
  const page = sheet.ownerDocument;
  const fresh = page.createElement("template");
  fresh.innerHTML = near.html;
  const laid = Array.from(fresh.content.children);
  if (
    laid.length !== to - from ||
    laid.some((one) => one.matches(WHOLE_ONLY) || one.querySelector(WHOLE_ONLY)) ||
    (!near.heads && !namesCarried(old.slice(from, kept), laid, by))
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
