import { namesSum, type PlacedNear, summed } from "../placing";

export type Span = readonly [number, number];

export const spansSum = (spans: readonly Span[]) => summed(spans.flat());

// The core answers how many blocks there are and a sum of those it did not draw again.
export const near = (drawn: {
  spans: readonly Span[];
  from: number;
  to: number;
  html: string;
  heads?: readonly string[];
  whole?: boolean;
}): PlacedNear => ({
  count: drawn.spans.length,
  sum: spansSum([...drawn.spans.slice(0, drawn.from), ...drawn.spans.slice(drawn.to)]),
  from: drawn.from,
  to: drawn.to,
  html: drawn.html,
  heads: drawn.heads ? namesSum(drawn.heads) : null,
  whole: drawn.whole ?? false,
});
