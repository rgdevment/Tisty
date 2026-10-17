import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { askedHere, HELP } from "./HelpHere";

export default function HelpKey({ onHelp }: { readonly onHelp: (topic?: string) => void }) {
  const now = useRef(onHelp);
  now.current = onHelp;
  useEffect(() => {
    const keyed = (event: KeyboardEvent) => {
      if (event.key !== "F1" || event.repeat) return;
      event.preventDefault();
      now.current(askedHere());
    };
    const asked = (event: Event) => now.current((event as CustomEvent<string | undefined>).detail);
    window.addEventListener("keydown", keyed);
    window.addEventListener(HELP, asked);
    let drop: (() => void) | undefined;
    let gone = false;
    listen("help", () => now.current())
      .then((off) => {
        if (gone) off();
        else drop = off;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      drop?.();
      window.removeEventListener("keydown", keyed);
      window.removeEventListener(HELP, asked);
    };
  }, []);
  return null;
}
