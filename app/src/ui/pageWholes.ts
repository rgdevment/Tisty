import { type Reckoned, unitFrom } from "../placing";
import type { Span } from "./pageEdits";
import { fenceAt, type Mend } from "./pageMends";
import { elementOf, sideOf, WHOLE } from "./pageNodes";
import { placeOf } from "./pagePlaces";

const CODE = ".page figure.code";
const OPENING = `${WHOLE}, .page > .front`;

type WholeReach = {
  readonly from: Element | null;
  readonly to: Element | null;
  readonly first: Element | null;
  readonly last: Element | null;
};

export type Choice = { readonly span: Span; readonly mend: Mend | null };

const nextOnSheet = (range: AbstractRange) => {
  const sheet = elementOf(range.startContainer);
  const onSheet = sheet === range.startContainer && sheet?.matches(".page");
  return onSheet ? elementOf(sheet.childNodes[range.startOffset]) : null;
};

const opensPage = (range: AbstractRange): Element | null => {
  const block = nextOnSheet(range);
  if (!block?.matches(WHOLE)) return null;
  for (let one = block.previousElementSibling; one; one = one.previousElementSibling)
    if (one.matches("[data-at]") && !one.matches(OPENING)) return null;
  return block;
};

const codeOf = (node: Node) => elementOf(node)?.closest(CODE) ?? null;

const codeBeside = (block: Element, node: Node, offset: number, before: boolean) =>
  sideOf(block.querySelector("code") ?? block, node, offset, before).toString();

const codeOpened = (range: AbstractRange): Element | null => {
  const next = nextOnSheet(range);
  const block = codeOf(range.startContainer) ?? (next?.matches(CODE) ? next : null);
  if (!block || block.contains(range.endContainer)) return null;
  return codeBeside(block, range.startContainer, range.startOffset, true) === "" ? block : null;
};

const codeClosed = (range: AbstractRange): Element | null => {
  const block = codeOf(range.endContainer);
  if (!block || block.contains(range.startContainer)) return null;
  const after = codeBeside(block, range.endContainer, range.endOffset, false);
  return /^\r?\n?$/.test(after) ? block : null;
};

const codeAcross = (inside: Node, outside: Node) => {
  const block = codeOf(inside);
  return block && !block.contains(outside) ? block : null;
};

export const wholeReach = (range: AbstractRange): WholeReach | null => {
  if (range.collapsed) return null;
  const opened = codeOpened(range);
  const to = codeClosed(range);
  const first = opened ? null : codeAcross(range.startContainer, range.endContainer);
  const last = to ? null : codeAcross(range.endContainer, range.startContainer);
  const from = opensPage(range) ?? opened;
  return from || to || first || last ? { from, to, first, last } : null;
};

export const spanReached = (
  range: AbstractRange,
  span: Span | null,
  text: string,
  known: Reckoned | null,
  reach = wholeReach(range),
): Span | null => {
  if (!span || !reach) return span;
  const start = placeOf(reach.from)?.[0];
  const end = placeOf(reach.to)?.[1];
  const from = start === undefined ? span.from : unitFrom(known, text, start);
  let to = end === undefined ? span.to : unitFrom(known, text, end);
  while (end !== undefined && to > from && /[\r\n]/.test(text[to - 1] ?? "")) to--;
  return { from, to };
};

export const mendOf = (
  range: AbstractRange,
  text: string,
  known: Reckoned | null,
  reach = wholeReach(range),
): Mend | null => {
  const fenceOf = (block: Element | null) => {
    const place = placeOf(block);
    return place ? fenceAt(text, unitFrom(known, text, place[0])) : null;
  };
  const first = fenceOf(reach?.first ?? null);
  const last = fenceOf(reach?.last ?? null);
  return first || last ? { first, last } : null;
};
