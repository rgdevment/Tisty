import { namesSum, type PlacedNear, summed } from "../placing";
import {
  keyMoved,
  placeCopied,
  placeNamed,
  placeOf,
  placesTaken,
  TITLE_OF_SHEET,
} from "./pagePlaces";

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

// A player drawn again would stop what it plays, so the one already drawn for the same file and name is kept.
const playersKept = (old: Element) => {
  const players = Array.from(old.querySelectorAll<HTMLElement>(".media[data-file]"));
  return () => {
    for (const img of old.querySelectorAll("img[src]")) {
      const at = players.findIndex(
        (one) =>
          one.dataset.file === img.getAttribute("src") &&
          one.dataset.name === (img.getAttribute("alt") ?? ""),
      );
      if (at < 0) continue;
      const [player] = players.splice(at, 1);
      placeCopied(img, player, "data-src", "data-placed");
      img.replaceWith(player);
    }
  };
};

const attributesTaken = (old: Element, fresh: Element) => {
  for (const name of old.getAttributeNames())
    if (!fresh.hasAttribute(name)) old.removeAttribute(name);
  for (const name of fresh.getAttributeNames()) {
    const value = fresh.getAttribute(name) ?? "";
    if (old.getAttribute(name) !== value) old.setAttribute(name, value);
  }
};

const shapedAlike = (old: Node, fresh: Node): boolean => {
  if (old.nodeType !== fresh.nodeType) return false;
  if (old.nodeType === Node.TEXT_NODE) return true;
  if (old.nodeType !== Node.ELEMENT_NODE) return false;
  if ((old as Element).tagName !== (fresh as Element).tagName) return false;
  return childrenAlike(old, fresh);
};

const childrenAlike = (old: Node, fresh: Node) =>
  old.childNodes.length === fresh.childNodes.length &&
  Array.from(old.childNodes).every((one, at) => shapedAlike(one, fresh.childNodes[at]));

const lettersTaken = (old: Text, said: string) => {
  const was = old.data;
  if (was === said) return;
  let from = 0;
  while (from < was.length && from < said.length && was[from] === said[from]) from++;
  let back = 0;
  const room = Math.min(was.length, said.length) - from;
  while (back < room && was[was.length - 1 - back] === said[said.length - 1 - back]) back++;
  old.replaceData(from, was.length - from - back, said.slice(from, said.length - back));
};

const shapeTaken = (old: Node, fresh: Node) => {
  if (old.nodeType === Node.TEXT_NODE) return lettersTaken(old as Text, (fresh as Text).data);
  if (old.nodeType === Node.ELEMENT_NODE) attributesTaken(old as Element, fresh as Element);
  for (const [at, one] of Array.from(old.childNodes).entries())
    shapeTaken(one, fresh.childNodes[at]);
};

// The system's speller marks the text nodes themselves, so a block that keeps its shape keeps them and only its letters change.
export const takenOver = (old: Element, fresh: Element) => {
  attributesTaken(old, fresh);
  if (childrenAlike(old, fresh)) return shapeTaken(old, fresh);
  const kept = playersKept(old);
  old.replaceChildren(...fresh.childNodes);
  kept();
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
    // The title's mark hangs on the blocks before, so a block that gains or loses it is drawn anew.
    if (old?.classList.contains("title") !== fresh?.classList.contains("title")) return null;
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
  // The title's slot waits on a page with nothing written, and leaves with the first block.
  const title = sheet.querySelector(TITLE_OF_SHEET);
  for (const one of sheet.querySelectorAll(":scope > [data-ghost]"))
    if (one !== title) one.remove();
  if (count > 0) title?.remove();
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

const PLACED = "[data-at], [data-src], [data-placed]";
// Dressing rewrites where a picture or a link points and adds looks of its own; a heading's name follows its words.
const DRESSED = new Set(["contenteditable", "spellcheck", "src", "href", "style", "id"]);
const dressing = (name: string) => placeNamed(name) || DRESSED.has(name);

const evenWords = (said: string | null) =>
  (said ?? "").replace(/\u00A0/g, " ").replace(/\u200B/g, "");

const placedIn = (block: Element) => [block, ...block.querySelectorAll(PLACED)];

// The classes the core gives are all still there; dressing may add its own.
const classesKept = (old: Element, fresh: Element) =>
  Array.from(fresh.classList).every((one) => old.classList.contains(one));

// Dressing adds to what the core drew and never takes away, so the core's own names and words are what must agree.
const drawnAlike = (old: Element, fresh: Element, nested: boolean, last: boolean) => {
  if (old.tagName !== fresh.tagName || !classesKept(old, fresh)) return false;
  if (nested && old.parentElement?.tagName !== fresh.parentElement?.tagName) return false;
  for (const name of fresh.getAttributeNames())
    if (name !== "class" && !dressing(name) && old.getAttribute(name) !== fresh.getAttribute(name))
      return false;
  if (fresh.tagName !== "SPAN" || fresh.attributes.length !== 1) return true;
  // A blank typed at a line's end is written, but the core draws the line without it.
  const words = (one: Element) => {
    const said = evenWords(one.textContent);
    return last ? said.replace(/\s+$/, "") : said;
  };
  return words(old) === words(fresh);
};

const blockAlike = (old: Element, fresh: Element) => {
  const olds = placedIn(old);
  const freshes = placedIn(fresh);
  const end = freshes.length - 1;
  return (
    olds.length === freshes.length &&
    freshes.every((one, at) => drawnAlike(olds[at], one, at > 0, at === end))
  );
};

const namesTaken = (olds: readonly Element[], freshes: readonly Element[]) => {
  freshes.forEach((one, at) => {
    if (one.id && olds[at] && olds[at].id !== one.id) olds[at].id = one.id;
  });
};

// A fold taken over comes back closed, so the ones open before stay open, found again in their order.
const foldsKept = (going: Element) => {
  const open = folds([going]).map((one) => one.open);
  return (laid: Element) =>
    folds([laid]).forEach((one, at) => {
      if (open[at]) one.open = true;
    });
};

/** The browser wrote the words already: blocks the core draws alike only take its places, and the rest are drawn anew. */
export const matched = (sheet: Element, near: PlacedNear): Element[] | null => {
  if (near.whole) return null;
  const old = Array.from(sheet.querySelectorAll(":scope > [data-at]"));
  const { count, from, to } = near;
  const kept = old.length - (count - to);
  if (count < to || kept < from) return null;
  if (!linedUp(old, near, kept, 0) || !namedAlike(sheet, old, near, kept)) return null;
  const fresh = sheet.ownerDocument.createElement("template");
  fresh.innerHTML = near.html;
  const drawn = Array.from(fresh.content.children);
  const redrawn = old.slice(from, kept);
  if (drawn.length !== redrawn.length) return null;
  const alike = drawn.map((one, at) => blockAlike(redrawn[at], one));
  const changed = drawn.filter((_, at) => !alike[at]);
  if (changed.some((one) => one.matches(WHOLE_ONLY) || one.querySelector(WHOLE_ONLY))) return null;
  const taken: Element[] = [];
  for (const [at, one] of drawn.entries()) {
    const going = redrawn[at];
    if (alike[at]) {
      placesTaken(placedIn(going), placedIn(one));
      namesTaken(placedIn(going), placedIn(one));
      continue;
    }
    if (going.id && !one.hasAttribute("id")) one.id = going.id;
    const reopened = foldsKept(going);
    const laid = going.tagName === one.tagName ? going : one;
    if (laid === one) going.replaceWith(one);
    else takenOver(going, one);
    reopened(laid);
    taken.push(laid);
  }
  return taken;
};
