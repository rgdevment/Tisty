import { useEffect } from "react";
import { fold } from "../breadth";
import { onMac } from "./WindowChrome";

export const FINDER = "side-finder";
export const FIND_KEY = onMac ? "⌘K" : "Ctrl+K";

export function useFindKey(shown: boolean, onFind: () => void) {
  useEffect(() => {
    if (!shown) return;
    const pressed = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.repeat || event.altKey || event.shiftKey) return;
      if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== "k") return;
      if (document.querySelector('[aria-modal="true"]')) return;
      event.preventDefault();
      onFind();
    };
    window.addEventListener("keydown", pressed);
    return () => window.removeEventListener("keydown", pressed);
  }, [shown, onFind]);
}

export const focusFinder = () => document.getElementById(FINDER)?.focus();

// A folded sidebar has no finder to reach, so it unfolds first.
export const findNow = (folded: boolean) => {
  if (folded) fold(false);
  requestAnimationFrame(focusFinder);
};
