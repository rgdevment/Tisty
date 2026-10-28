import { useReactFlow } from "@xyflow/react";
import { type KeyboardEvent, useRef } from "react";

type Point = { readonly x: number; readonly y: number };
type Act = (() => void) | null | undefined;

// Keys typed into a field, or pressed on the canvas's own buttons, are not for what is chosen.
const elsewhere = (e: KeyboardEvent) =>
  (e.target as Element).closest(
    "input, textarea, select, [contenteditable], [role=toolbar], .react-flow__panel",
  ) !== null;

/** Where a canvas puts what its palette brings, and the keys it answers for what is chosen. */
export const useCanvasHands = (acts: {
  readonly remove: Act;
  readonly undo: Act;
  readonly redo: Act;
  readonly duplicate?: Act;
}) => {
  const { screenToFlowPosition } = useReactFlow();
  const pane = useRef<HTMLDivElement>(null);
  const middleOfPane = () => {
    const box = pane.current?.getBoundingClientRect();
    return screenToFlowPosition({
      x: (box?.left ?? 0) + (box?.width ?? 0) / 2,
      y: (box?.top ?? 0) + (box?.height ?? 0) / 2,
    });
  };
  // A piece let go outside the canvas, over the palette say, is put nowhere.
  const droppedAt = (at: Point): Point | null =>
    pane.current?.contains(document.elementFromPoint(at.x, at.y)) ? screenToFlowPosition(at) : null;
  const keyed = (e: KeyboardEvent) => {
    if (elsewhere(e)) return;
    const mod = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    const act = (did: Act) => {
      e.preventDefault();
      did?.();
    };
    if ((key === "delete" || key === "backspace") && acts.remove) act(acts.remove);
    else if (mod && key === "z") act(e.shiftKey ? acts.redo : acts.undo);
    else if (mod && key === "y") act(acts.redo);
    else if (mod && key === "d" && acts.duplicate !== undefined) act(acts.duplicate);
  };
  return { pane, middleOfPane, droppedAt, keyed };
};
