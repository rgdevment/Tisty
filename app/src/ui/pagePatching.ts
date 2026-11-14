import { spanned } from "tisty-editor/read";
import type { PlacedNear } from "../placing";

const NAMES = ["data-at", "data-src"];
// What the page lends or links in a block is laid by its own dressing, which only a whole drawing runs.
const WHOLE_ONLY = "code.language-widget, figure.widget, .doc-card";

const shifted = (block: Element, by: number) => {
  for (const one of [block, ...block.querySelectorAll("[data-at], [data-src]")]) {
    for (const name of NAMES) {
      const at = spanned(one.getAttribute(name));
      if (at) one.setAttribute(name, `${at[0] + by}-${at[1] + by}`);
    }
  }
};

const same = (one: Element, want: readonly [number, number] | undefined, by = 0) => {
  const at = spanned(one.getAttribute("data-at"));
  return Boolean(at && want && at[0] + by === want[0] && at[1] + by === want[1]);
};

// The blocks the core drew again take the place of the old ones, and those after only move by
// what the edit added; when the page does not line up with the core, it is drawn whole instead.
export const patched = (sheet: Element, near: PlacedNear, by: number): Element[] | null => {
  const old = Array.from(sheet.querySelectorAll(":scope > [data-at]"));
  const { spans, from, to } = near;
  const after = spans.length - to;
  const kept = old.length - after;
  if (after < 0 || kept < from) return null;
  for (let at = 0; at < from; at++) if (!same(old[at], spans[at])) return null;
  for (let at = 0; at < after; at++) if (!same(old[kept + at], spans[to + at], by)) return null;
  const page = sheet.ownerDocument;
  const fresh = page.createElement("template");
  fresh.innerHTML = near.html;
  const laid = Array.from(fresh.content.children);
  if (
    laid.length !== to - from ||
    laid.some((one) => one.matches(WHOLE_ONLY) || one.querySelector(WHOLE_ONLY))
  )
    return null;
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
