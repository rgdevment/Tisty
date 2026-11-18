import type { Drawing } from "./drawing";
import { stepped, wayOf } from "./moving";

export interface Walk {
  picked: () => number | null;
  hidden: () => readonly boolean[];
  pick: (node: number) => void;
  open: (node: number) => void;
  find: () => void;
}

const typedIn = (target: EventTarget | null): boolean =>
  target instanceof Element &&
  target.closest("input, select, textarea, button, a, [contenteditable]") !== null;

export const walker =
  (drawing: Drawing, count: number, near: number | undefined, walk: Walk) =>
  (event: KeyboardEvent): void => {
    if (event.defaultPrevented || typedIn(event.target)) return;
    if (event.key === "/") {
      event.preventDefault();
      walk.find();
      return;
    }
    const picked = walk.picked();
    if (event.key === "Enter" && picked !== null) {
      event.preventDefault();
      walk.open(picked);
      return;
    }
    const way = wayOf(event.key);
    if (!way) return;
    event.preventDefault();
    const hidden = walk.hidden();
    if (picked === null) {
      const start = near !== undefined && !hidden[near] ? near : hidden.indexOf(false);
      if (start >= 0) walk.pick(start);
      return;
    }
    const next = stepped(
      picked,
      way,
      (node) => (hidden[node] ? undefined : drawing.nodeOnScreen(node)),
      drawing.beside(picked),
      count,
    );
    if (next !== undefined) walk.pick(next);
  };
