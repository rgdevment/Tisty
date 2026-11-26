import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

/** The item of a Mac's Edit menu that pastes without formatting, as the window hears it. */
export const PASTE_PLAINLY = "paste-plainly";

// On a Mac the menu holds the keys, so the page that has the focus asks for the paste, and the window pastes for it.
export const usePlainPasting = (focused: () => { plainNext: () => void } | null) => {
  const asking = useRef(focused);
  asking.current = focused;
  useEffect(() => {
    let drop: (() => void) | undefined;
    let gone = false;
    listen(PASTE_PLAINLY, () => {
      const page = asking.current();
      if (!page) return;
      page.plainNext();
      invoke("pasted_plainly").catch(() => undefined);
    })
      .then((off) => {
        if (gone) off();
        else drop = off;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      drop?.();
    };
  }, []);
};
