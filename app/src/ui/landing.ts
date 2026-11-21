import { useEffect } from "react";
import type { RowId } from "../settingsRows";

const LIT_FOR = 1600;
const WAITS_FOR = 2500;

// A page may still be reading what it draws, so the row is looked for until it shows up.
export function useLanding(row: RowId | undefined, at: number | undefined) {
  useEffect(() => {
    if (!row || !at) return;
    const began = Date.now();
    let frame = 0;
    let lit: HTMLElement | null = null;
    let dims: ReturnType<typeof setTimeout> | undefined;
    const look = () => {
      const there = document.querySelector<HTMLElement>(`[data-setting="${row}"]`);
      if (!there) {
        if (Date.now() - began < WAITS_FOR) frame = requestAnimationFrame(look);
        return;
      }
      lit = there;
      there.scrollIntoView?.({ block: "center" });
      there.setAttribute("data-landed", "");
      dims = setTimeout(() => there.removeAttribute("data-landed"), LIT_FOR);
    };
    frame = requestAnimationFrame(look);
    return () => {
      cancelAnimationFrame(frame);
      clearTimeout(dims);
      lit?.removeAttribute("data-landed");
    };
  }, [row, at]);
}
