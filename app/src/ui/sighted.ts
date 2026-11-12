import { bytesAt, charsAt } from "tisty-editor/read";
import { reanchored } from "./anchoring";

const shownAt = (one: HTMLElement) => one.getBoundingClientRect();

// The blocks come in the order they are read, so the first one reaching below `top` is found by halves.
export const firstInSight = (blocks: readonly HTMLElement[], top: number): HTMLElement | null => {
  let low = 0;
  let high = blocks.length;
  while (low < high) {
    const mid = (low + high) >> 1;
    if (shownAt(blocks[mid]).bottom > top) high = mid;
    else low = mid + 1;
  }
  for (let at = low; at < blocks.length; at++)
    if (shownAt(blocks[at]).height > 0) return blocks[at];
  return null;
};

export const carried = (before: string, next: string, bytes: number): number => {
  const from = charsAt(before, bytes);
  return bytesAt(next, reanchored(before, next, from, from)?.[0] ?? from);
};
