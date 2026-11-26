import { placeOf } from "./pagePlaces";

export type WordAt = { readonly said: string; readonly at: [number, number] };

const LETTER = /[\p{L}\p{M}'’]/u;
const bytesOf = (said: string) => new TextEncoder().encode(said).length;

type Spot = { readonly node: Node; readonly offset: number };

const spotAt = (page: Document, x: number, y: number): Spot | null => {
  const spot = page.caretPositionFromPoint?.(x, y);
  if (spot) return { node: spot.offsetNode, offset: spot.offset };
  const range = page.caretRangeFromPoint?.(x, y);
  return range ? { node: range.startContainer, offset: range.startOffset } : null;
};

// Only words laid letter for letter from their Markdown can be put back by their bytes; code is never spelled.
const plainSpan = (node: Node): Element | null => {
  const span = node.parentElement;
  if (!span?.matches(".page [data-src]") || span.closest("code, pre, a, .math")) return null;
  const place = placeOf(span, "data-src");
  if (!place || place[1] - place[0] !== bytesOf(span.textContent ?? "")) return null;
  return span.firstChild === node && span.childNodes.length === 1 ? span : null;
};

export const wordIn = (node: Node, offset: number): WordAt | null => {
  if (node.nodeType !== Node.TEXT_NODE) return null;
  const span = plainSpan(node);
  const place = span && placeOf(span, "data-src");
  if (!place) return null;
  const data = (node as Text).data;
  let from = offset;
  let to = offset;
  while (from > 0 && LETTER.test(data[from - 1])) from--;
  while (to < data.length && LETTER.test(data[to])) to++;
  const said = data.slice(from, to).replace(/^['’]+|['’]+$/g, "");
  if (!said || !/\p{L}/u.test(said)) return null;
  const start = place[0] + bytesOf(data.slice(0, from + data.slice(from, to).indexOf(said)));
  return { said, at: [start, start + bytesOf(said)] };
};

export const wordAt = (page: Document, x: number, y: number): WordAt | null => {
  const spot = spotAt(page, x, y);
  return spot ? wordIn(spot.node, spot.offset) : null;
};
