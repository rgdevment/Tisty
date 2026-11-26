import { useEffect, useRef } from "react";
import { letterOf, modKey } from "./macKeys";

/// Ctrl+N (⌘N) makes a document wherever the library stands, from anywhere in the window.
export default function NewDocKey({ onNew }: Readonly<{ onNew: () => void }>) {
  const making = useRef(onNew);
  making.current = onNew;
  useEffect(() => {
    const pressed = (e: KeyboardEvent) => {
      if (!modKey(e) || e.shiftKey || e.altKey || letterOf(e) !== "n") return;
      if (document.querySelector('[aria-modal="true"]')) return;
      e.preventDefault();
      making.current();
    };
    window.addEventListener("keydown", pressed);
    return () => window.removeEventListener("keydown", pressed);
  }, []);
  return null;
}
