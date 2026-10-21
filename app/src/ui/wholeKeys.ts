import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

// Counts a sheet reads as asks: Ctrl+E opens or leaves the whole document, Ctrl+A chooses every word.
export function useWholeKeys(shown: boolean, wholeable: boolean) {
  const [wholeAsked, setWholeAsked] = useState(0);
  const [allAsked, setAllAsked] = useState(0);
  useEffect(() => {
    if (!shown) return;
    // The page hears every Ctrl key the block left alone, so the keys work from inside a block too.
    const pressed = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.shiftKey || event.altKey || event.repeat)
        return;
      const key = event.key.toLowerCase();
      // A field of the window's own, the search or a title, keeps its keys.
      if ((event.target as Element | null)?.closest?.("input, textarea, [contenteditable]")) return;
      if (key === "e" && wholeable) {
        event.preventDefault();
        setWholeAsked((many) => many + 1);
      }
      // A page just opened has the window's focus, not its own, so select-all is handed to it.
      if (key === "a" && event.target !== window) {
        event.preventDefault();
        setAllAsked((many) => many + 1);
      }
    };
    window.addEventListener("keydown", pressed);
    // On a Mac the menu holds the keys, so the switch arrives as its item.
    let drop: (() => void) | undefined;
    let gone = false;
    listen("whole", () => {
      if (wholeable) setWholeAsked((many) => many + 1);
    })
      .then((off) => {
        if (gone) off();
        else drop = off;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      drop?.();
      window.removeEventListener("keydown", pressed);
    };
  }, [shown, wholeable]);
  return { wholeAsked, allAsked };
}
