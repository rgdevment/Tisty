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

const HELD_FOR = 1500;

export const holding = (
  room: HTMLElement,
  grows: HTMLElement,
  aim: () => number | null,
): (() => void) => {
  let mine = Number.NaN;
  let tall = -1;
  let over = false;
  const reach = () => {
    const target = over ? null : aim();
    if (target === null) return;
    room.scrollTop = target;
    mine = room.scrollTop;
    tall = room.scrollHeight;
  };
  const nudged = () => {
    if (room.scrollTop === mine) return;
    // A page that shrank takes the view with it; only a scroll on a page the same height is the person's.
    if (room.scrollHeight !== tall) reach();
    else stop();
  };
  const seeing = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(reach);
  const frame = requestAnimationFrame(reach);
  const timer = window.setTimeout(() => stop(), HELD_FOR);
  function stop() {
    if (over) return;
    over = true;
    seeing?.disconnect();
    room.removeEventListener("scroll", nudged);
    cancelAnimationFrame(frame);
    window.clearTimeout(timer);
  }
  seeing?.observe(grows);
  room.addEventListener("scroll", nudged);
  return stop;
};

export const carried = (before: string, next: string, bytes: number): number => {
  const from = charsAt(before, bytes);
  return bytesAt(next, reanchored(before, next, from, from)?.[0] ?? from);
};
