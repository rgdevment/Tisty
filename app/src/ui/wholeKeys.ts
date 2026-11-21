import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { modKey, onMac } from "./macKeys";

export const WHOLE_ASKED = "tisty:whole";

// Counts a sheet reads as asks: Ctrl+E opens or leaves the whole document, Ctrl+A chooses every word.
export function useWholeKeys(shown: boolean, wholeable: boolean) {
  const [wholeAsked, setWholeAsked] = useState(0);
  const [allAsked, setAllAsked] = useState(0);
  useEffect(() => {
    if (!shown) return;
    // The page hears every Ctrl key the block left alone, so the keys work from inside a block too.
    const pressed = (event: KeyboardEvent) => {
      if (!modKey(event) || event.shiftKey || event.altKey) return;
      const key = event.key.toLowerCase();
      // A field of the window's own, the search or a title, keeps its keys.
      const field = (event.target as Element | null)?.closest?.(
        "input, textarea, [contenteditable]",
      );
      if (event.repeat || field) {
        if (onMac && key === "e") event.preventDefault();
        return;
      }
      if (key === "e" && wholeable) {
        event.preventDefault();
        setWholeAsked((many) => many + 1);
      }
      // A page just opened has the window's focus, not its own, so select-all is handed to it;
      // the page's own keys come back aimed at the window itself, which is no element.
      if (key === "a" && event.target instanceof Node) {
        event.preventDefault();
        setAllAsked((many) => many + 1);
      }
    };
    const asked = () => {
      if (wholeable) setWholeAsked((many) => many + 1);
    };
    window.addEventListener("keydown", pressed);
    window.addEventListener(WHOLE_ASKED, asked);
    // On a Mac the menu holds the keys, so the switch arrives as its item.
    let drop: (() => void) | undefined;
    let gone = false;
    listen("whole", asked)
      .then((off) => {
        if (gone) off();
        else drop = off;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      drop?.();
      window.removeEventListener("keydown", pressed);
      window.removeEventListener(WHOLE_ASKED, asked);
    };
  }, [shown, wholeable]);
  return { wholeAsked, allAsked };
}
