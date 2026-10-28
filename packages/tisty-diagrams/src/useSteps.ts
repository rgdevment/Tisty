import { useState } from "react";

// What the window changed, kept to be undone; text typed by hand in between ends the trail.
interface Step {
  readonly before: string;
  readonly after: string;
}

const KEPT_STEPS = 100;

export interface Steps {
  // Writes the window's change as a step that can be undone.
  readonly stepped: (written: string) => void;
  readonly undo: (() => void) | null;
  readonly redo: (() => void) | null;
}

/** The window's changes to a diagram's text, one by one, to undo and redo while nobody typed over them. */
export const useSteps = (text: string, setText: (text: string) => void): Steps => {
  const [steps, setSteps] = useState<{ done: readonly Step[]; undone: readonly Step[] }>({
    done: [],
    undone: [],
  });
  const last = steps.done[steps.done.length - 1];
  const next = steps.undone[steps.undone.length - 1];
  return {
    stepped: (written) => {
      if (written === text) return;
      setSteps({
        done: [...steps.done.slice(1 - KEPT_STEPS), { before: text, after: written }],
        undone: [],
      });
      setText(written);
    },
    undo:
      last?.after === text
        ? () => {
            setSteps({ done: steps.done.slice(0, -1), undone: [...steps.undone, last] });
            setText(last.before);
          }
        : null,
    redo:
      next?.before === text
        ? () => {
            setSteps({ done: [...steps.done, next], undone: steps.undone.slice(0, -1) });
            setText(next.after);
          }
        : null,
  };
};
