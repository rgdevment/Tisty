import { namesSum, type PlacedNear, summed } from "../placing";
import { keyMoved, placeOf } from "./pagePlaces";

// What the page lends or links in a block is laid by its own dressing, which only a whole drawing runs.
const WHOLE_ONLY = "code.language-widget, figure.widget, .doc-card";

const folds = (blocks: readonly Element[]) =>
  blocks.flatMap((one) => [
    ...(one.matches("details") ? [one as HTMLDetailsElement] : []),
    ...one.querySelectorAll("details"),
  ]);

// A fold drawn anew comes closed, so one being written in stays open, found again by where it begins.
const openFolds = (going: readonly Element[], by: number, edited?: readonly [number, number]) => {
  const [from, to] = edited ?? [Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY];
  return new Set(
    folds(going)
      .filter((one) => one.open)
      .flatMap((one) => {
        const at = placeOf(one)?.[0];
        if (at === undefined || (at >= from && at < to)) return [];
        return [at < from ? at : at + by];
      }),
  );
};

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

const samePlace = (old: Element | undefined, fresh: Element | undefined, moved: number) => {
  const was = placeOf(old);
  const now = placeOf(fresh);
  if (!old || !fresh || !was || !now || old.tagName !== fresh.tagName) return null;
  return now[0] === was[0] + moved && now[1] === was[1] + moved ? was : null;
};

// Only the same heading in the same place keeps its shown name; one that came or went renames those after it.
const namesCarried = (was: readonly Element[], laid: readonly Element[], by: number) => {
  const before = headsIn(was, HEADINGS);
  const after = headsIn(laid, HEADINGS);
  const same = (one: Element, at: number) =>
    Boolean(samePlace(before[at], one, 0) ?? samePlace(before[at], one, by));
  if (before.length !== after.length || !after.every((one, at) => same(one, at))) return false;
  after.forEach((one, at) => {
    if (before[at].id) one.id = before[at].id;
  });
  return true;
};

// Adding or taking out a child of the page restyles every paragraph of a long page, so words keep their block.
const WORDS = "p, h1, h2, h3, h4, h5, h6";

export const keptInPlace = (one: Element) => one.matches(WORDS);

const sameKind = (old: Element | undefined, fresh: Element): old is Element =>
  old !== undefined && old.tagName === fresh.tagName && keptInPlace(old);

export const takenOver = (old: Element, fresh: Element) => {
  for (const name of old.getAttributeNames())
    if (!fresh.hasAttribute(name)) old.removeAttribute(name);
  for (const name of fresh.getAttributeNames()) {
    const value = fresh.getAttribute(name) ?? "";
    if (old.getAttribute(name) !== value) old.setAttribute(name, value);
  }
  old.replaceChildren(...fresh.childNodes);
};

// The core draws a block on each side of the key too; one whose place held and the key never reached is as it was.
const besideKept = (
  redrawn: readonly Element[],
  drawn: readonly Element[],
  by: number,
  [from, to]: readonly [number, number],
  renamed: boolean,
) => {
  const held = (old: Element | undefined, fresh: Element | undefined, moved: number) => {
    const was = samePlace(old, fresh, moved);
    return was && renamed && old && fresh && headsIn([old, fresh], HEADINGS).length > 0
      ? null
      : was;
  };
  const first = held(redrawn[0], drawn[0], 0);
  const lead = first && first[1] < from ? 1 : 0;
  const last = held(redrawn[redrawn.length - 1], drawn[drawn.length - 1], by);
  const room = Math.min(redrawn.length, drawn.length) - lead;
  const trail = room > 0 && last && last[0] > to ? 1 : 0;
  return [lead, trail] as const;
};

// The blocks the core drew again take the place of the old ones, and those after only move by
// what the edit added; when the page does not line up with the core, it is drawn whole instead.
export const patched = (
  sheet: Element,
  near: PlacedNear,
  by: number,
  edited?: readonly [number, number],
): Element[] | null => {
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
  const drawn = Array.from(fresh.content.children);
  const redrawn = old.slice(from, kept);
  if (drawn.length !== to - from || (!near.heads && !namesCarried(redrawn, drawn, by))) return null;
  const [lead, trail] = edited
    ? besideKept(redrawn, drawn, by, edited, Boolean(near.heads))
    : [0, 0];
  const going = redrawn.slice(lead, redrawn.length - trail);
  const coming = drawn.slice(lead, drawn.length - trail);
  if (coming.some((one) => one.matches(WHOLE_ONLY) || one.querySelector(WHOLE_ONLY))) return null;
  const opened = openFolds(going, by, edited);
  const laid = coming.map((one, at) => (sameKind(going[at], one) ? going[at] : one));
  keyMoved(sheet, old, from + lead, kept - trail, laid, by);
  coming.forEach((one, at) => {
    if (laid[at] !== one) takenOver(laid[at], one);
  });
  for (const one of folds(laid)) if (opened.has(placeOf(one)?.[0] ?? -1)) one.open = true;
  for (const one of going) if (!laid.includes(one)) one.remove();
  for (const one of sheet.querySelectorAll(":scope > [data-ghost]")) one.remove();
  const next = old[kept - trail] ?? null;
  const before = from + lead > 0 ? old[from + lead - 1] : null;
  laid.forEach((one, at) => {
    if (one.parentElement === sheet) return;
    const prior = laid[at - 1];
    const later = laid.find((block) => block.parentElement === sheet) ?? next;
    if (prior) prior.after(one);
    else if (later) later.before(one);
    else if (before) before.after(one);
    else sheet.prepend(one);
  });
  return laid;
};
